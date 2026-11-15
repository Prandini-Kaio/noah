//! Revision comparison in an explicitly aligned, arbitrary SfM reference frame.
use noah_geometry::{PointCloud, Transform, cleaning::NearestIndex, registration};
use noah_pipeline::{Result, Status, controlled, inspect, read_bounded};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: PathBuf,
    pub manifest_sha256: String,
    pub run_id: String,
    pub input_images: usize,
    pub registered_images: usize,
    pub points: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Registration {
    pub method: String,
    pub common_images: usize,
    pub common_image_sha256: Vec<String>,
    pub new_to_reference_translation: [f64; 3],
    pub new_to_reference_rotation: [[f64; 3]; 3],
    pub new_to_reference_scale: f64,
    pub camera_rmse: f64,
    pub max_camera_residual: f64,
    pub normalized_camera_rmse: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Distances {
    pub queries: usize,
    pub target_points: usize,
    pub mean: f64,
    pub rms: f64,
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
    pub within_threshold_fraction: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema_version: u32,
    pub reference: Source,
    pub updated: Source,
    pub frame: String,
    pub physical_unit_known: bool,
    pub metrology_validated: bool,
    pub threshold: f64,
    pub registration: Registration,
    pub reference_to_updated: Distances,
    pub updated_to_reference: Distances,
    pub sampling: String,
}
pub struct Comparison {
    pub report: Report,
    pub reference: PointCloud,
    pub updated_aligned: PointCloud,
    pub updated_distances: Vec<f64>,
}
fn load(
    root: &Path,
) -> Result<(
    Source,
    noah_core::reconstruction::Reconstruction,
    BTreeMap<String, [f64; 3]>,
)> {
    let root = fs::canonicalize(root)?;
    let m = inspect(&root)?;
    if matches!(m.status, Status::Prepared | Status::Running) {
        return Err("aguarde o run terminar antes de comparar".into());
    }
    if !m
        .artifacts
        .iter()
        .any(|a| a.path == "exports/reconstruction.json")
    {
        return Err("run sem reconstrução registrada".into());
    }
    let meta: serde_json::Value = serde_json::from_slice(&read_bounded(
        &controlled(&root, "exports/reconstruction.json")?,
        4 * 1024 * 1024,
    )?)?;
    let selected = meta["selected_model"].as_str().ok_or("submodelo ausente")?;
    if selected.is_empty() || selected.len() > 20 || !selected.bytes().all(|b| b.is_ascii_digit()) {
        return Err("submodelo inválido".into());
    }
    for name in ["cameras.txt", "images.txt", "points3D.txt"] {
        if !m
            .artifacts
            .iter()
            .any(|a| a.path == format!("model-{selected}/{name}"))
        {
            return Err("modelo TXT não registrado".into());
        }
    }
    let reconstruction = crate::model::parse(&root.join(format!("model-{selected}")))?;
    let names: HashMap<_, _> = m
        .inputs
        .iter()
        .map(|i| {
            (
                Path::new(&i.snapshot)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                i.sha256.clone(),
            )
        })
        .collect();
    let mut cameras = BTreeMap::new();
    let mut duplicates = std::collections::HashSet::new();
    for view in &reconstruction.views {
        let hash = names
            .get(&view.image_name)
            .ok_or("vista não corresponde ao snapshot")?;
        if cameras.insert(hash.clone(), camera_center(view)).is_some() {
            duplicates.insert(hash.clone());
        }
    }
    for duplicate in duplicates {
        cameras.remove(&duplicate);
    }
    let info = Source {
        path: root.clone(),
        manifest_sha256: noah_pipeline::hash_file(&root.join("run.json"), 16 * 1024 * 1024)?.0,
        run_id: m.run_id,
        input_images: m.inputs.len(),
        registered_images: reconstruction.views.len(),
        points: reconstruction.cloud.positions().len(),
    };
    Ok((info, reconstruction, cameras))
}
fn camera_center(v: &noah_core::reconstruction::CameraView) -> [f64; 3] {
    let [w, x, y, z] = v.rotation_wxyz;
    let r = [
        [
            1. - 2. * (y * y + z * z),
            2. * (x * y - z * w),
            2. * (x * z + y * w),
        ],
        [
            2. * (x * y + z * w),
            1. - 2. * (x * x + z * z),
            2. * (y * z - x * w),
        ],
        [
            2. * (x * z - y * w),
            2. * (y * z + x * w),
            1. - 2. * (x * x + y * y),
        ],
    ];
    std::array::from_fn(|a| -(0..3).map(|i| r[i][a] * v.translation[i]).sum::<f64>())
}
fn sample(cloud: &PointCloud, limit: usize, t: Option<Transform>) -> Result<PointCloud> {
    let stride = cloud.positions().len().div_ceil(limit).max(1);
    let ids: Vec<_> = (0..cloud.positions().len()).step_by(stride).collect();
    let points = ids
        .iter()
        .map(|&i| {
            let p = cloud.positions()[i];
            if let Some(t) = t {
                Ok(t.apply(p)?)
            } else {
                Ok(p)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PointCloud::new(
        points,
        cloud.colors().map(|c| ids.iter().map(|&i| c[i]).collect()),
    )?)
}
fn distances(
    source: &PointCloud,
    target: &PointCloud,
    threshold: f64,
) -> Result<(Distances, Vec<f64>)> {
    let index = NearestIndex::new(target.positions())?;
    let values: Vec<_> = source
        .positions()
        .iter()
        .map(|&p| index.distance(p).map_err(|e| e.into()))
        .collect::<Result<_>>()?;
    let n = values.len();
    if n == 0 {
        return Err("comparação sem pontos".into());
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let rms = (values.iter().map(|d| d * d).sum::<f64>() / n as f64).sqrt();
    let mut sorted = values.clone();
    sorted.sort_by(f64::total_cmp);
    let p = |q: f64| {
        sorted[((n as f64 * q).ceil() as usize)
            .saturating_sub(1)
            .min(n - 1)]
    };
    Ok((
        Distances {
            queries: n,
            target_points: target.positions().len(),
            mean,
            rms,
            p50: p(0.5),
            p95: p(0.95),
            max: sorted[n - 1],
            within_threshold_fraction: values.iter().filter(|&&d| d <= threshold).count() as f64
                / n as f64,
        },
        values,
    ))
}
/// At least four common unique images and normalized camera RMSE <=5% are required.
pub fn compare(reference: &Path, updated: &Path, threshold: f64) -> Result<Comparison> {
    let (a, ra, ca) = load(reference)?;
    let (b, rb, cb) = load(updated)?;
    let hashes: Vec<_> = ca.keys().filter(|h| cb.contains_key(*h)).cloned().collect();
    if hashes.len() < 4 {
        return Err("comparação exige ao menos quatro fotos únicas registradas em comum".into());
    }
    let source: Vec<_> = hashes.iter().map(|h| cb[h]).collect();
    let target: Vec<_> = hashes.iter().map(|h| ca[h]).collect();
    let fit = registration::similarity(&source, &target)
        .map_err(|_| "câmeras comuns degeneradas/colineares; não é possível alinhar as revisões")?;
    let normalized = fit.rmse / fit.reference_radius;
    if !normalized.is_finite() || normalized > 0.05 {
        return Err(format!("alinhamento de câmeras inconsistente ({:.2}% do raio de referência; limite heurístico 5%). Não comparar nuvens neste referencial",normalized*100.).into());
    }
    let threshold = if threshold == 0. {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for p in ra.cloud.positions() {
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
        ((0..3).map(|a| (hi[a] - lo[a]).powi(2)).sum::<f64>().sqrt() * 0.01).max(1e-12)
    } else {
        threshold
    };
    if !threshold.is_finite() || threshold <= 0. || threshold > 1e12 {
        return Err("limiar de comparação deve ser positivo e finito".into());
    }
    let reference_target = sample(&ra.cloud, 200_000, None)?;
    let updated_target = sample(&rb.cloud, 200_000, Some(fit.transform))?;
    let reference = sample(&ra.cloud, 20_000, None)?;
    let updated_aligned = sample(&rb.cloud, 20_000, Some(fit.transform))?;
    let (ref_dist, _) = distances(&reference, &updated_target, threshold)?;
    let (new_dist, values) = distances(&updated_aligned, &reference_target, threshold)?;
    let report=Report{schema_version:1,reference:a,updated:b,frame:"reference_run_arbitrary_right_handed_sfm".into(),physical_unit_known:false,metrology_validated:false,threshold,registration:Registration{method:"positive_similarity_least_squares_common_camera_centers_by_image_sha256".into(),common_images:hashes.len(),common_image_sha256:hashes,new_to_reference_translation:fit.transform.translation(),new_to_reference_rotation:fit.transform.rotation(),new_to_reference_scale:fit.transform.scale(),camera_rmse:fit.rmse,max_camera_residual:fit.max_residual,normalized_camera_rmse:normalized},reference_to_updated:ref_dist,updated_to_reference:new_dist,sampling:"deterministic original-index stride; <=20000 queries/direction, <=200000 target points/direction; point-to-point nearest neighbour, not point-to-surface or uncertainty".into()};
    Ok(Comparison {
        report,
        reference,
        updated_aligned,
        updated_distances: values,
    })
}
pub fn save(comparison: &Comparison, path: &Path) -> Result<()> {
    noah_pipeline::write_new(path, &serde_json::to_vec_pretty(&comparison.report)?)
}
pub fn reopen(path: &Path) -> Result<Comparison> {
    let saved: Report = serde_json::from_slice(&read_bounded(path, 4 * 1024 * 1024)?)?;
    if saved.schema_version != 1
        || !saved.reference.path.is_absolute()
        || !saved.updated.path.is_absolute()
    {
        return Err("relatório de comparação incompatível".into());
    }
    for source in [&saved.reference, &saved.updated] {
        if noah_pipeline::hash_file(&source.path.join("run.json"), 16 * 1024 * 1024)?.0
            != source.manifest_sha256
        {
            return Err("fonte da comparação alterada".into());
        }
    }
    let computed = compare(&saved.reference.path, &saved.updated.path, saved.threshold)?;
    if computed.report != saved {
        return Err("relatório diverge dos dados originais".into());
    }
    Ok(computed)
}
