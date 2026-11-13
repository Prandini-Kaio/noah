//! Surface stages and traceable derivation from an immutable existing NOAH run.
use crate::{args, export_dense_preview, export_sparse, path, verify_engine};
use noah_pipeline::{Job, Result, Status, controlled, hash_file, inspect, read_bounded, write_new};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    DelaunaySparse,
    PoissonDense,
}
impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Self::DelaunaySparse => "delaunay_sparse_visibility",
            Self::PoissonDense => "poisson_dense_oriented",
        }
    }
}
pub struct Config {
    pub source: PathBuf,
    pub output: PathBuf,
    pub engine: PathBuf,
    pub method: Method,
    pub timeout: Duration,
}
pub(crate) fn mesh(job: &mut Job, method: Method, sparse: &Path) -> Result<()> {
    let root = job.root().to_path_buf();
    let raw = path(root.join("exports/mesh.ply"))?;
    match method {
        Method::DelaunaySparse => job.step(
            "meshing",
            args(&[
                "delaunay_mesher",
                "--input_path",
                &path(sparse.to_path_buf())?,
                "--input_type",
                "sparse",
                "--output_path",
                &raw,
                "--DelaunayMeshing.num_threads",
                "2",
            ]),
        )?,
        Method::PoissonDense => job.step(
            "meshing",
            args(&[
                "poisson_mesher",
                "--input_path",
                &path(root.join("exports/dense.ply"))?,
                "--output_path",
                &raw,
                "--PoissonMeshing.depth",
                "6",
                "--PoissonMeshing.trim",
                "0",
                "--PoissonMeshing.num_threads",
                "2",
            ]),
        )?,
    }
    let bytes = read_bounded(
        &controlled(&root, "exports/mesh.ply")?,
        noah_storage::MAX_PLY_BYTES,
    )?;
    let mesh = noah_storage::colmap_ply::parse_mesh(&bytes)?;
    let obj = noah_storage::colmap_ply::encode_obj(&mesh)?;
    // Validate the exact format the desktop will import before declaring success.
    noah_storage::obj::parse(&obj)?;
    write_new(&root.join("exports/mesh.obj"), &obj)?;
    let info = serde_json::json!({"schema_version":1,"source_run":job.manifest().run_id,"parent_run":job.manifest().parameters.get("parent_run"),"method":method.name(),"poisson_depth":if method==Method::PoissonDense{Some(6)}else{None},"poisson_trim":if method==Method::PoissonDense{Some(0)}else{None},"vertex_count":mesh.positions().len(),"face_count":mesh.triangles().len(),"unit":"unknown","frame":"arbitrary-right-handed-sfm","textured":false,"watertight_validated":false,"metrology_validated":false,"normalized_obj":"geometry_only; raw PLY attributes preserved"});
    write_new(
        &root.join("exports/mesh.metadata.json"),
        &serde_json::to_vec_pretty(&info)?,
    )?;
    job.artifact("exports/mesh.ply", "surface_raw_colmap")?;
    job.artifact("exports/mesh.obj", "surface_mesh_obj")?;
    job.artifact("exports/mesh.metadata.json", "surface_provenance")
}
fn required<'a>(m: &'a noah_pipeline::Manifest, p: &str) -> Result<&'a noah_pipeline::Artifact> {
    m.artifacts
        .iter()
        .find(|a| a.path == p)
        .ok_or_else(|| format!("run sem artefato registrado necessário: {p}").into())
}
fn copy_registered(
    source: &Path,
    job: &Job,
    m: &noah_pipeline::Manifest,
    from: &str,
    to: &str,
) -> Result<()> {
    let a = required(m, from)?;
    let b = read_bounded(&controlled(source, from)?, noah_storage::MAX_PLY_BYTES)?;
    write_new(&job.root().join(to), &b)?;
    if hash_file(&job.root().join(to), noah_storage::MAX_PLY_BYTES)?.0 != a.sha256 {
        return Err("origem alterada durante derivação".into());
    }
    Ok(())
}
/// New run snapshots images/cameras and records parent manifest; never mutates parent.
pub fn from_run(config: Config) -> Result<()> {
    derive(config, None)
}
/// Continue a sparse run using a verified cleaning revision, with COLMAP track links intact.
pub fn from_clean_revision(config: Config, revision: &Path) -> Result<()> {
    derive(config, Some(revision))
}
fn derive(config: Config, revision: Option<&Path>) -> Result<()> {
    let m = inspect(&config.source)?;
    if matches!(m.status, Status::Prepared | Status::Running) {
        return Err("aguarde o término do run de origem".into());
    }
    required(&m, "exports/reconstruction.json")?;
    let meta: serde_json::Value = serde_json::from_slice(&read_bounded(
        &controlled(&config.source, "exports/reconstruction.json")?,
        4 * 1024 * 1024,
    )?)?;
    let model = meta["selected_model"]
        .as_str()
        .ok_or("run sem submodelo selecionado")?;
    if model.is_empty() || model.len() > 20 || !model.bytes().all(|b| b.is_ascii_digit()) {
        return Err("submodelo de origem inválido".into());
    }
    for file in ["cameras.txt", "images.txt", "points3D.txt"] {
        required(&m, &format!("model-{model}/{file}"))?;
    }
    if config.method == Method::PoissonDense {
        required(&m, "exports/dense.ply")?;
    }
    let clean = if let Some(root) = revision {
        if config.method != Method::DelaunaySparse {
            return Err("revisão sparse exige Delaunay CPU".into());
        }
        let (mut r, _) = noah_storage::cloud_revision::load(root)?;
        let (source, indices) = noah_storage::cloud_revision::run_selection(root)?;
        r.source = source;
        r.kept_indices = indices;
        if r.source.kind != "run"
            || fs::canonicalize(&r.source.path)? != fs::canonicalize(&config.source)?
            || r.source.sha256 != hash_file(&config.source.join("run.json"), 64 * 1024 * 1024)?.0
            || r.source.item_id != format!("run:{}", m.run_id)
        {
            return Err("revisão não pertence à nuvem esparsa deste run".into());
        }
        let original = crate::model::parse(&config.source.join(format!("model-{model}")))?;
        let (_, clean_cloud) = noah_storage::cloud_revision::load(root)?;
        if noah_geometry::cleaning::subset(&original.cloud, &r.kept_indices)? != clean_cloud {
            return Err("pontos da revisão não correspondem ao modelo COLMAP".into());
        }
        Some((
            r,
            read_bounded(&root.join("cloud-revision.json"), 16 * 1024 * 1024)?,
        ))
    } else {
        None
    };
    let parameters = BTreeMap::from([
        ("mode".into(), "surface_from_run".into()),
        ("surface_method".into(), config.method.name().into()),
        ("parent_run".into(), m.run_id.clone()),
        ("threads".into(), "2".into()),
    ]);
    let mut job = Job::create(
        &config.output,
        &config.source.join("images"),
        &config.engine,
        parameters,
        config.timeout,
    )?;
    let result = (|| -> Result<()> {
        verify_engine(&mut job)?;
        if job.manifest().inputs.len() != m.inputs.len() {
            return Err("quantidade de imagens de origem mudou".into());
        }
        for input in &job.manifest().inputs {
            let old = m
                .inputs
                .iter()
                .find(|i| {
                    Path::new(&i.snapshot).file_name().and_then(|s| s.to_str())
                        == Some(input.original_name.as_str())
                })
                .ok_or("nomes de snapshot incompatíveis com as câmeras")?;
            if input.sha256 != old.sha256
                || Path::new(&input.snapshot).file_name() != Path::new(&old.snapshot).file_name()
            {
                return Err("imagem de origem alterada ou renomeada".into());
            }
        }
        fs::create_dir(job.root().join("model-0"))?;
        for file in ["cameras.txt", "images.txt", "points3D.txt"] {
            copy_registered(
                &config.source,
                &job,
                &m,
                &format!("model-{model}/{file}"),
                &format!("model-0/{file}"),
            )?;
        }
        if let Some((revision, bytes)) = &clean {
            let original = crate::model::parse(&job.root().join("model-0"))?;
            let ids = revision
                .kept_indices
                .iter()
                .map(|&i| original.point_ids[i])
                .collect();
            let selection = serde_json::json!({"schema_version":1,"original_source":revision.source,"root_point_indices":revision.kept_indices,"point_ids":revision.kept_indices.iter().map(|&i|original.point_ids[i]).collect::<Vec<_>>()});
            write_new(
                &job.root().join("cleaning-selected-points.json"),
                &serde_json::to_vec_pretty(&selection)?,
            )?;
            job.artifact("cleaning-selected-points.json", "cleaned_sparse_selection")?;
            crate::model::filter_points(&job.root().join("model-0"), &ids)?;
            write_new(&job.root().join("cleaning.json"), bytes)?;
            job.artifact("cleaning.json", "cloud_cleaning_revision")?;
        }
        let r = crate::model::parse(&job.root().join("model-0"))?;
        export_sparse(&mut job, "0", &r)?;
        write_new(
            &job.root().join("parent-run.json"),
            &serde_json::to_vec_pretty(&m)?,
        )?;
        job.artifact("parent-run.json", "parent_manifest_snapshot")?;
        fs::create_dir(job.root().join("sparse"))?;
        fs::create_dir(job.root().join("sparse/0"))?;
        let sparse = job.root().join("sparse/0");
        job.step(
            "restore_model",
            args(&[
                "model_converter",
                "--input_path",
                &path(job.root().join("model-0"))?,
                "--output_path",
                &path(sparse.clone())?,
                "--output_type",
                "BIN",
            ]),
        )?;
        for file in ["cameras.bin", "images.bin", "points3D.bin"] {
            job.artifact(&format!("sparse/0/{file}"), "restored_colmap_model")?;
        }
        if config.method == Method::PoissonDense {
            copy_registered(
                &config.source,
                &job,
                &m,
                "exports/dense.ply",
                "exports/dense.ply",
            )?;
            export_dense_preview(&mut job)?;
        }
        mesh(&mut job, config.method, &sparse)?;
        job.succeed()
    })();
    if let Err(e) = result {
        job.fail(&e.to_string())?;
        return Err(format!("{e}; derivação preservada em {}", job.root().display()).into());
    }
    Ok(())
}
