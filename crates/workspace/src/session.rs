//! Sessão schema 8 (leitura 1/2/3/4/5/6/7): referências verificadas, câmera e ajustes de instância.
use crate::{Camera, Placement, Result, Scene, Source, source_fingerprint};
use noah_pipeline::{read_bounded, write_new};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entity {
    id: String,
    #[serde(default)]
    name: Option<String>,
    placement: Placement,
    visible: bool,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    explode_direction: [f64; 3],
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
    #[serde(default)]
    assemblies: Vec<crate::Assembly>,
    #[serde(default)]
    explosion: f64,
    #[serde(default = "default_distance")]
    explosion_distance: f64,
    #[serde(default)]
    calibration: Option<crate::measurement::ScaleReference>,
    #[serde(default)]
    measurements: Vec<crate::measurement::Measurement>,
    #[serde(default)]
    view: crate::view::ViewOptions,
    #[serde(default)]
    angles: Vec<crate::inspection::AngularMeasurement>,
    #[serde(default)]
    dimension_measurements: Vec<crate::inspection::DimensionMeasurement>,
}
fn default_distance() -> f64 {
    1.
}
fn validate_options(scene: &Scene) -> Result<()> {
    scene.camera.validate()?;
    scene.view.validate()?;
    scene.validate_world()?;
    scene.validate_measurements()?;
    if !scene.point_size.is_finite() || !(1.0..=8.0).contains(&scene.point_size) {
        return Err("tamanho de ponto inválido".into());
    }
    if scene.selected.as_ref().is_some_and(|id| {
        !scene.items.iter().any(|i| &i.id == id) && !scene.assemblies.iter().any(|g| &g.id == id)
    }) {
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
        schema_version: 8,
        sources: scene.sources.clone(),
        entities: scene
            .items
            .iter()
            .map(|i| Entity {
                id: i.id.clone(),
                name: Some(i.name.clone()),
                placement: i.placement.clone(),
                visible: i.visible,
                parent: i.parent.clone(),
                explode_direction: i.explode_direction,
            })
            .collect(),
        camera: scene.camera.clone(),
        selected: scene.selected.clone(),
        point_size: scene.point_size,
        grid: scene.grid,
        wireframe: scene.wireframe,
        assemblies: scene.assemblies.clone(),
        explosion: scene.explosion,
        explosion_distance: scene.explosion_distance,
        calibration: scene.calibration.clone(),
        measurements: scene.measurements.clone(),
        view: scene.view.clone(),
        angles: scene.angles.clone(),
        dimension_measurements: scene.dimension_measurements.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&session)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("sessão excede 4 MiB".into());
    }
    write_new(path, &bytes)
}
pub fn load(path: &Path) -> Result<Scene> {
    let saved: Session = serde_json::from_slice(&read_bounded(path, 4 * 1024 * 1024)?)?;
    if !matches!(saved.schema_version, 1..=8)
        || saved.sources.len() > 32
        || saved.entities.len() > 1000
    {
        return Err("schema/limites de sessão inválidos".into());
    }
    if saved.schema_version < 6
        && saved
            .sources
            .iter()
            .any(|s| matches!(s.kind, crate::SourceKind::RunSurface))
    {
        return Err("representação de superfície exige sessão 6".into());
    }
    if saved.schema_version < 7
        && saved
            .sources
            .iter()
            .any(|s| matches!(s.kind, crate::SourceKind::CloudRevision))
    {
        return Err("revisão limpa exige sessão 7".into());
    }
    let mut scene = Scene::default();
    for source in &saved.sources {
        if !source.path.is_absolute() {
            return Err("sessão exige caminho absoluto de fonte".into());
        }
        scene.add_source_with_surface(
            &source.path,
            matches!(source.kind, crate::SourceKind::RunSurface),
        )?;
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
        if let Some(name) = entity.name {
            if name.trim().is_empty() || name.len() > 256 {
                return Err("nome de peça inválido".into());
            }
            item.name = name;
        }
        item.placement = entity.placement;
        item.visible = entity.visible;
        item.parent = entity.parent;
        item.explode_direction = entity.explode_direction;
        item.validate()?;
    }
    scene.assemblies = saved.assemblies;
    scene.explosion = saved.explosion;
    scene.explosion_distance = saved.explosion_distance;
    scene.calibration = saved.calibration;
    scene.measurements = saved.measurements;
    scene.view = saved.view;
    scene.angles = saved.angles;
    scene.dimension_measurements = saved.dimension_measurements;
    scene.camera = saved.camera;
    scene.selected = saved.selected;
    scene.point_size = saved.point_size;
    scene.grid = saved.grid;
    scene.wireframe = saved.wireframe;
    validate_options(&scene)?;
    Ok(scene)
}
