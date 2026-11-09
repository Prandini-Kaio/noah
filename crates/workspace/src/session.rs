//! Sessão schema 1: referências verificadas, câmera e ajustes de instância.
use crate::{Camera, Placement, Result, Scene, Source, source_fingerprint};
use noah_pipeline::{read_bounded, write_new};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entity {
    id: String,
    placement: Placement,
    visible: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema_version: u32,
    sources: Vec<Source>,
    entities: Vec<Entity>,
    camera: Camera,
    selected: Option<String>,
    point_size: f32,
    grid: bool,
    wireframe: bool,
}
fn validate_options(scene: &Scene) -> Result<()> {
    scene.camera.validate()?;
    if !scene.point_size.is_finite() || !(1.0..=8.0).contains(&scene.point_size) {
        return Err("tamanho de ponto inválido".into());
    }
    if scene
        .selected
        .as_ref()
        .is_some_and(|id| !scene.items.iter().any(|i| &i.id == id))
    {
        return Err("seleção ausente na cena".into());
    }
    Ok(())
}
pub fn save(scene: &Scene, path: &Path) -> Result<()> {
    validate_options(scene)?;
    for source in &scene.sources {
        if source_fingerprint(&source.path, &source.kind)? != source.sha256 {
            return Err("fonte alterada desde a abertura; sessão não salva".into());
        }
    }
    for item in &scene.items {
        item.validate()?;
    }
    let session = Session {
        schema_version: 1,
        sources: scene.sources.clone(),
        entities: scene
            .items
            .iter()
            .map(|i| Entity {
                id: i.id.clone(),
                placement: i.placement.clone(),
                visible: i.visible,
            })
            .collect(),
        camera: scene.camera.clone(),
        selected: scene.selected.clone(),
        point_size: scene.point_size,
        grid: scene.grid,
        wireframe: scene.wireframe,
    };
    let bytes = serde_json::to_vec_pretty(&session)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("sessão excede 4 MiB".into());
    }
    write_new(path, &bytes)
}
pub fn load(path: &Path) -> Result<Scene> {
    let saved: Session = serde_json::from_slice(&read_bounded(path, 4 * 1024 * 1024)?)?;
    if saved.schema_version != 1 || saved.sources.len() > 32 || saved.entities.len() > 1000 {
        return Err("schema/limites de sessão inválidos".into());
    }
    let mut scene = Scene::default();
    for source in &saved.sources {
        if !source.path.is_absolute() {
            return Err("sessão exige caminho absoluto de fonte".into());
        }
        scene.add_source(&source.path)?;
        if scene.sources.last() != Some(source) {
            return Err("fonte movida, alterada ou tipo incompatível".into());
        }
    }
    if scene.items.len() != saved.entities.len() {
        return Err("objetos da sessão não correspondem às fontes".into());
    }
    let mut seen = HashSet::new();
    for entity in saved.entities {
        if !seen.insert(entity.id.clone()) {
            return Err("ID duplicado em sessão".into());
        }
        let item = scene
            .items
            .iter_mut()
            .find(|i| i.id == entity.id)
            .ok_or("objeto da sessão não encontrado")?;
        item.placement = entity.placement;
        item.visible = entity.visible;
        item.validate()?;
    }
    scene.camera = saved.camera;
    scene.selected = saved.selected;
    scene.point_size = saved.point_size;
    scene.grid = saved.grid;
    scene.wireframe = saved.wireframe;
    validate_options(&scene)?;
    Ok(scene)
}
