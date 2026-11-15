//! New capture revision: validated parent + unique additional photos, full reprocessing.
use crate::{Config, execute};
use noah_pipeline::{Job, Result, Status, controlled, hash_file, inspect, read_bounded, write_new};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};
pub fn refine(parent: &Path, config: Config) -> Result<()> {
    let m = inspect(parent)?;
    if matches!(m.status, Status::Prepared | Status::Running) {
        return Err("aguarde o run de origem terminar".into());
    }
    if !m
        .artifacts
        .iter()
        .any(|a| a.path == "exports/reconstruction.json")
    {
        return Err("run pai sem reconstrução registrada".into());
    }
    let parent = fs::canonicalize(parent)?;
    let parent_bytes = read_bounded(&parent.join("run.json"), 16 * 1024 * 1024)?;
    if serde_json::from_slice::<serde_json::Value>(&parent_bytes)? != serde_json::to_value(&m)? {
        return Err("manifesto pai alterado durante leitura".into());
    }
    let parent_hash = noah_storage::content_sha256(&parent_bytes);
    let prepared = crate::preparation::provenance(&config.images)?;
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    let mut origins = BTreeMap::new();
    for input in &m.inputs {
        if seen.insert(input.sha256.clone()) {
            let path = controlled(&parent, &input.snapshot)?;
            paths.push(path.clone());
            origins.insert(input.sha256.clone(),serde_json::json!({"source":"parent","original_name":input.original_name,"snapshot":input.snapshot}));
        }
    }
    let mut extras = Vec::new();
    noah_pipeline::directory(&config.images)?;
    for entry in fs::read_dir(&config.images)? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg"))
        {
            noah_pipeline::regular(&path)?;
            extras.push(path);
            if extras.len() > 500 {
                return Err("máximo de 500 fotos adicionais".into());
            }
        }
    }
    extras.sort();
    let mut added = 0;
    let mut duplicates = 0;
    for path in extras {
        let hash = hash_file(&path, 32 * 1024 * 1024)?.0;
        if seen.insert(hash.clone()) {
            added += 1;
            origins.insert(
                hash,
                serde_json::json!({"source":"additional","path":fs::canonicalize(&path)?}),
            );
            paths.push(path);
        } else {
            duplicates += 1;
        }
    }
    if added == 0 {
        return Err("nenhuma foto nova única; duplicatas por conteúdo foram ignoradas".into());
    }
    if paths.len() > 500 {
        return Err("pai + novas fotos excedem 500 imagens".into());
    }
    let params = BTreeMap::from([
        ("mode".into(), "refinement_full_reprocess".into()),
        ("dense".into(), config.dense.to_string()),
        ("mesh".into(), config.mesh.to_string()),
        ("max_image_size".into(), "1600".into()),
        ("parent_run".into(), m.run_id.clone()),
        ("parent_manifest_sha256".into(), parent_hash.clone()),
        ("new_unique_images".into(), added.to_string()),
        ("duplicate_images_ignored".into(), duplicates.to_string()),
        ("threads".into(), "2".into()),
        ("feature_gpu".into(), "false".into()),
        ("matching_gpu".into(), "false".into()),
    ]);
    let mut job = Job::create_from_paths(
        &config.output,
        &paths,
        &config.engine,
        params,
        config.timeout,
    )?;
    let result = (|| -> Result<()> {
        let snapshots: Vec<_> = job
            .manifest()
            .inputs
            .iter()
            .map(|i| {
                let origin = origins
                    .get(&i.sha256)
                    .ok_or("foto alterada durante snapshot")?;
                Ok(serde_json::json!({"snapshot":i.snapshot,"sha256":i.sha256,"origin":origin}))
            })
            .collect::<Result<_>>()?;
        if job.manifest().inputs.len() != seen.len()
            || job
                .manifest()
                .inputs
                .iter()
                .map(|i| i.sha256.clone())
                .collect::<HashSet<_>>()
                != seen
        {
            return Err("entradas alteradas durante snapshot".into());
        }
        if let Some(bytes) = prepared {
            write_new(&job.root().join("additional-preparation.json"), &bytes)?;
            job.artifact(
                "additional-preparation.json",
                "additional_image_preparation",
            )?;
        }
        write_new(&job.root().join("parent-run.json"), &parent_bytes)?;
        job.artifact("parent-run.json", "parent_manifest_snapshot")?;
        let info = serde_json::json!({"schema_version":1,"parent_run":m.run_id,"parent_path":parent,"parent_manifest_sha256":parent_hash,"new_run":job.manifest().run_id,"new_unique_images":added,"duplicates_ignored":duplicates,"strategy":"full_sfm_reprocess","inherited_geometry_edits":false,"inherited_physical_calibration":false,"inputs":snapshots});
        write_new(
            &job.root().join("refinement.json"),
            &serde_json::to_vec_pretty(&info)?,
        )?;
        job.artifact("refinement.json", "capture_revision_provenance")?;
        execute(&mut job, config.dense, config.mesh)
    })();
    if let Err(e) = result {
        job.fail(&e.to_string())?;
        return Err(format!("{e}; revisão preservada em {}", job.root().display()).into());
    }
    Ok(())
}
