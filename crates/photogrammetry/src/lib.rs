//! Adaptador de processo COLMAP. Domínio não importa esta crate.
use noah_pipeline::{Job, Result, controlled, directory, read_bounded, write_new};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
pub mod model;

pub struct Config {
    pub engine: PathBuf,
    pub images: PathBuf,
    pub output: PathBuf,
    pub dense: bool,
    pub timeout: Duration,
}
fn path(path: PathBuf) -> Result<String> {
    Ok(path.to_str().ok_or("caminho não é UTF-8")?.into())
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
/// Run independente do schema E1; saída permanece no referencial arbitrário SfM.
pub fn reconstruct(config: Config) -> Result<()> {
    let parameters = BTreeMap::from([
        (
            "mode".into(),
            if config.dense { "dense" } else { "sparse" }.into(),
        ),
        ("feature_gpu".into(), "false".into()),
        ("matching_gpu".into(), "false".into()),
        ("threads".into(), "2".into()),
        ("max_image_size".into(), "1600".into()),
    ]);
    let mut job = Job::create(
        &config.output,
        &config.images,
        &config.engine,
        parameters,
        config.timeout,
    )?;
    let result = execute(&mut job, config.dense);
    if let Err(error) = result {
        job.fail(&error.to_string())?;
        return Err(format!("{error}; run preservado em {}", job.root().display()).into());
    }
    Ok(())
}
fn execute(job: &mut Job, dense: bool) -> Result<()> {
    job.step("version", args(&["--help"]))?;
    let version = String::from_utf8(read_bounded(
        &job.root().join("logs/00-version.stdout.log"),
        8 * 1024 * 1024,
    )?)?;
    let banner = version
        .lines()
        .find(|s| s.contains("COLMAP 3.11"))
        .ok_or("adaptador requer COLMAP 3.11.x; verifique logs da versão")?;
    job.set_engine_version(banner.trim().chars().take(4096).collect())?;
    let root = job.root().to_path_buf();
    let images = path(root.join("images"))?;
    let database = path(root.join("database.db"))?;
    fs::create_dir(root.join("sparse"))?;
    job.step(
        "features",
        args(&[
            "feature_extractor",
            "--database_path",
            &database,
            "--image_path",
            &images,
            "--ImageReader.camera_model",
            "SIMPLE_RADIAL",
            "--SiftExtraction.use_gpu",
            "0",
            "--SiftExtraction.num_threads",
            "2",
            "--SiftExtraction.max_image_size",
            "1600",
        ]),
    )?;
    job.step(
        "matching",
        args(&[
            "exhaustive_matcher",
            "--database_path",
            &database,
            "--SiftMatching.use_gpu",
            "0",
            "--SiftMatching.num_threads",
            "2",
        ]),
    )?;
    job.step(
        "mapping",
        args(&[
            "mapper",
            "--database_path",
            &database,
            "--image_path",
            &images,
            "--output_path",
            &path(root.join("sparse"))?,
            "--Mapper.num_threads",
            "2",
        ]),
    )?;
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root.join("sparse"))? {
        let dir = entry?.path();
        directory(&dir)?;
        let name = dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("nome de submodelo inválido")?
            .to_owned();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_digit()) || candidates.len() >= 32 {
            return Err("submodelos COLMAP inválidos/excedentes".into());
        }
        candidates.push((name, dir));
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    let mut best = None;
    for (name, dir) in candidates {
        let txt = root.join(format!("model-{name}"));
        fs::create_dir(&txt)?;
        job.step(
            &format!("convert_{name}"),
            args(&[
                "model_converter",
                "--input_path",
                &path(dir.clone())?,
                "--output_path",
                &path(txt.clone())?,
                "--output_type",
                "TXT",
            ]),
        )?;
        let reconstruction = model::parse(&txt)?;
        if best.as_ref().is_none_or(
            |(_, _, r): &(PathBuf, String, noah_core::reconstruction::Reconstruction)| {
                reconstruction.views.len() > r.views.len()
            },
        ) {
            best = Some((dir, name, reconstruction));
        }
    }
    let (selected, name, reconstruction) = best.ok_or(
        "COLMAP não produziu modelo esparso; verifique sobreposição/textura e logs do mapper",
    )?;
    let snapshots: std::collections::HashSet<_> = job
        .manifest()
        .inputs
        .iter()
        .filter_map(|i| Path::new(&i.snapshot).file_name().and_then(|s| s.to_str()))
        .collect();
    if reconstruction
        .views
        .iter()
        .any(|v| !snapshots.contains(v.image_name.as_str()))
    {
        return Err("modelo referencia imagem fora do snapshot".into());
    }
    fs::create_dir(root.join("exports"))?;
    write_new(
        &root.join("exports/sparse.ply"),
        &noah_storage::ply::encode(&reconstruction.cloud),
    )?;
    let cameras:Vec<_>=reconstruction.cameras.iter().map(|c|serde_json::json!({"id":c.id,"model":c.model,"width":c.width,"height":c.height,"parameters":c.parameters})).collect();
    let views:Vec<_>=reconstruction.views.iter().map(|v|serde_json::json!({"id":v.id,"camera_id":v.camera_id,"image_name":v.image_name,"world_to_camera":{"rotation_wxyz":v.rotation_wxyz,"translation":v.translation}})).collect();
    let export = serde_json::json!({"schema_version":1,"unit":"unknown","frame":"arbitrary-right-handed-sfm","source_run":job.manifest().run_id,"selected_model":name,"input_images":job.manifest().inputs.len(),"registered_images":views.len(),"point_ids":reconstruction.point_ids,"point_count":reconstruction.cloud.positions().len(),"cameras":cameras,"views":views});
    write_new(
        &root.join("exports/reconstruction.json"),
        &serde_json::to_vec_pretty(&export)?,
    )?;
    job.artifact("exports/sparse.ply", "sparse_point_cloud")?;
    job.artifact("exports/reconstruction.json", "cameras_and_poses")?;
    for file in ["cameras.txt", "images.txt", "points3D.txt"] {
        job.artifact(&format!("model-{name}/{file}"), "colmap_text_model")?;
    }
    if dense {
        fs::create_dir(root.join("dense"))?;
        let workspace = path(root.join("dense"))?;
        job.step(
            "undistort",
            args(&[
                "image_undistorter",
                "--image_path",
                &images,
                "--input_path",
                &path(selected)?,
                "--output_path",
                &workspace,
                "--output_type",
                "COLMAP",
                "--max_image_size",
                "1600",
            ]),
        )?;
        job.step(
            "patch_match",
            args(&[
                "patch_match_stereo",
                "--workspace_path",
                &workspace,
                "--workspace_format",
                "COLMAP",
                "--PatchMatchStereo.geom_consistency",
                "true",
            ]),
        )?;
        let fused = path(root.join("exports/dense.ply"))?;
        job.step(
            "fusion",
            args(&[
                "stereo_fusion",
                "--workspace_path",
                &workspace,
                "--workspace_format",
                "COLMAP",
                "--input_type",
                "geometric",
                "--output_path",
                &fused,
                "--StereoFusion.num_threads",
                "2",
            ]),
        )?;
        validate_dense(&controlled(&root, "exports/dense.ply")?)?;
        job.artifact("exports/dense.ply", "dense_point_cloud_colmap")?;
    }
    job.succeed()
}
/// Verifica envelope do PLY denso binário COLMAP XYZ/normal/RGB (27 bytes/vértice).
fn validate_dense(path: &Path) -> Result<()> {
    use std::io::{BufRead, BufReader};
    let mut reader = BufReader::new(fs::File::open(path)?);
    let mut header = Vec::new();
    let mut total = 0;
    loop {
        let mut line = String::new();
        let n = reader.by_ref().take(4097).read_line(&mut line)?;
        total += n;
        if n == 0 || n > 4096 || total > 65536 {
            return Err("header PLY denso inválido".into());
        }
        header.push(line.trim_end().to_owned());
        if line.trim_end() == "end_header" {
            break;
        }
    }
    let fields: Vec<_> = header
        .iter()
        .filter(|s| !s.starts_with("comment "))
        .map(String::as_str)
        .collect();
    if fields.len() != 13
        || fields[0] != "ply"
        || fields[1] != "format binary_little_endian 1.0"
        || fields[3..12]
            != [
                "property float x",
                "property float y",
                "property float z",
                "property float nx",
                "property float ny",
                "property float nz",
                "property uchar red",
                "property uchar green",
                "property uchar blue",
            ]
    {
        return Err("formato PLY denso COLMAP não suportado".into());
    }
    let count: u64 = fields[2]
        .strip_prefix("element vertex ")
        .ok_or("contagem PLY denso ausente")?
        .parse()?;
    if count == 0 || count > 1_000_000 || fs::metadata(path)?.len() != total as u64 + count * 27 {
        return Err("contagem/tamanho da nuvem densa inválido".into());
    }
    use std::io::Read;
    let mut vertex = [0u8; 27];
    for _ in 0..count {
        reader.read_exact(&mut vertex)?;
        for field in vertex[..24].as_chunks::<4>().0 {
            if !f32::from_le_bytes(*field).is_finite() {
                return Err("coordenada/normal densa não finita".into());
            }
        }
    }
    Ok(())
}
