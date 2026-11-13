//! Cena, câmera, seleção e sessão independentes da interface nativa.
use noah_geometry::{PointCloud, Transform, TriangleMesh};
use noah_pipeline::{hash_file, read_bounded};
use noah_storage::StoredProject;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};
pub mod alignment;
pub mod assembly;
pub mod camera;
pub mod demo;
pub mod inspection;
pub mod measurement;
pub mod view;
pub use assembly::Assembly;
pub mod session;
pub use camera::Camera;
pub type Result<T> = noah_pipeline::Result<T>;
pub const DISPLAY_POINTS: usize = 50_000;
pub const DISPLAY_FACES: usize = 20_000;
#[derive(Debug, Clone)]
pub enum Geometry {
    Points(PointCloud),
    Mesh(TriangleMesh),
}
impl Geometry {
    pub fn positions(&self) -> &[[f64; 3]] {
        match self {
            Self::Points(c) => c.positions(),
            Self::Mesh(m) => m.positions(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Project,
    Run,
    RunSurface,
    Ply,
    Obj,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: PathBuf,
    pub kind: SourceKind,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub translation: [f64; 3],
    pub angles: [f64; 3],
    pub scale: f64,
}
impl Default for Placement {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            angles: [0.; 3],
            scale: 1.,
        }
    }
}
impl Placement {
    pub fn transform(&self) -> Result<Transform> {
        Ok(Transform::from_euler(
            self.translation,
            self.angles,
            self.scale,
        )?)
    }
}
#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub source: usize,
    pub geometry: Arc<Geometry>,
    pub base: Transform,
    pub placement: Placement,
    pub visible: bool,
    pub unit: String,
    pub frame: String,
    pub parent: Option<String>,
    pub explode_direction: [f64; 3],
}
impl Item {
    /// Edição E3 aplicada depois da transformação original de E1.
    pub fn transform(&self) -> Result<Transform> {
        Ok(self.placement.transform()?.compose(&self.base)?)
    }
    pub fn validate(&self) -> Result<()> {
        let t = self.transform()?;
        for &p in self.geometry.positions() {
            let world = t.apply(p)?;
            if world.iter().any(|v| v.abs() > 1e12) {
                return Err("geometria fora do alcance visual de +/-1e12".into());
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Scene {
    pub sources: Vec<Source>,
    pub items: Vec<Item>,
    pub camera: Camera,
    pub selected: Option<String>,
    pub point_size: f32,
    pub grid: bool,
    pub wireframe: bool,
    pub assemblies: Vec<Assembly>,
    pub explosion: f64,
    pub explosion_distance: f64,
    pub calibration: Option<measurement::ScaleReference>,
    pub measurements: Vec<measurement::Measurement>,
    pub view: view::ViewOptions,
    pub angles: Vec<inspection::AngularMeasurement>,
    pub dimension_measurements: Vec<inspection::DimensionMeasurement>,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            sources: vec![],
            items: vec![],
            camera: Camera::default(),
            selected: None,
            point_size: 2.5,
            grid: true,
            wireframe: false,
            assemblies: vec![],
            explosion: 0.,
            explosion_distance: 1.,
            calibration: None,
            measurements: vec![],
            view: Default::default(),
            angles: vec![],
            dimension_measurements: vec![],
        }
    }
}
impl Scene {
    pub fn add_source(&mut self, path: &Path) -> Result<()> {
        self.add_source_with_surface(path, true)
    }
    pub(crate) fn add_source_with_surface(
        &mut self,
        path: &Path,
        prefer_surface: bool,
    ) -> Result<()> {
        let (source, mut items) = load_source_mode(path, self.sources.len(), prefer_surface)?;
        if self.sources.iter().any(|s| s.path == source.path) {
            return Err("fonte já aberta nesta sessão".into());
        }
        if self.sources.len() >= 32 || self.items.len() + items.len() > 1000 {
            return Err("limite de 32 fontes/1000 objetos".into());
        }
        if self
            .items
            .iter()
            .map(|i| i.geometry.positions().len())
            .sum::<usize>()
            + items
                .iter()
                .map(|i| i.geometry.positions().len())
                .sum::<usize>()
            > 2_000_000
        {
            return Err("cena excede 2 milhões de vértices".into());
        }
        let existing: HashSet<_> = self.items.iter().map(|i| i.id.as_str()).collect();
        if items.iter().any(|i| existing.contains(i.id.as_str())) {
            return Err("identidade duplicada na cena".into());
        }
        self.sources.push(source);
        self.items.append(&mut items);
        self.fit(false)?;
        Ok(())
    }
    pub fn fit(&mut self, selected_only: bool) -> Result<()> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut count = 0;
        let transforms = self.world_transforms(true)?;
        for (index, item) in self.items.iter().enumerate() {
            if !item.visible
                || (selected_only
                    && self.selected.as_ref() != Some(&item.id)
                    && !self
                        .selected
                        .as_ref()
                        .is_some_and(|id| self.belongs_to(index, id)))
            {
                continue;
            }
            let t = transforms[index];
            for &p in item.geometry.positions() {
                let p = t.apply(p)?;
                for i in 0..3 {
                    min[i] = min[i].min(p[i]);
                    max[i] = max[i].max(p[i]);
                }
                count += 1;
            }
        }
        if count == 0 {
            return Ok(());
        }
        self.camera.target = std::array::from_fn(|i| (min[i] + max[i]) * 0.5);
        let radius = (0..3)
            .map(|i| (max[i] - min[i]).powi(2))
            .sum::<f64>()
            .sqrt()
            * 0.5;
        self.camera.distance =
            (radius.max(1e-6) / 25f64.to_radians().sin() * 1.3).clamp(1e-9, 1e15);
        self.camera.validate()
    }
    pub fn set_placement(&mut self, index: usize, placement: Placement) -> Result<()> {
        let item = self.items.get_mut(index).ok_or("objeto ausente")?;
        let old = std::mem::replace(&mut item.placement, placement);
        if let Err(error) = self.validate_world() {
            self.items[index].placement = old;
            return Err(error);
        }
        Ok(())
    }
}
fn source_fingerprint(path: &Path, kind: &SourceKind) -> Result<String> {
    let file = match kind {
        SourceKind::Project => path.join("project.json"),
        SourceKind::Run | SourceKind::RunSurface => path.join("run.json"),
        _ => path.to_path_buf(),
    };
    Ok(hash_file(&file, 64 * 1024 * 1024)?.0)
}
pub fn load_source(path: &Path, source_index: usize) -> Result<(Source, Vec<Item>)> {
    load_source_mode(path, source_index, true)
}
fn load_source_mode(
    path: &Path,
    source_index: usize,
    prefer_surface: bool,
) -> Result<(Source, Vec<Item>)> {
    let path = std::fs::canonicalize(path)?;
    let kind = if path.is_dir() {
        if path.join("project.json").is_file() {
            SourceKind::Project
        } else if path.join("run.json").is_file() {
            SourceKind::Run
        } else {
            return Err("pasta sem projeto E1 ou run E2".into());
        }
    } else {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "ply" => SourceKind::Ply,
            "obj" => SourceKind::Obj,
            _ => return Err("abra PLY ASCII, OBJ triangular, pasta E1 ou run E2".into()),
        }
    };
    let mut source = Source {
        sha256: source_fingerprint(&path, &kind)?,
        path,
        kind,
    };
    let mut items = Vec::new();
    let name = source
        .path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Geometria")
        .to_owned();
    match source.kind {
        SourceKind::Project => {
            let stored = StoredProject::open(&source.path)?;
            let p = stored.project();
            for object in p.objects() {
                let asset = p
                    .assets()
                    .iter()
                    .find(|a| a.id == object.asset_id)
                    .ok_or("asset ausente")?;
                items.push(Item {
                    id: format!("project:{:032x}:{:032x}", p.id.0, object.id.0),
                    name: object.name.clone(),
                    source: source_index,
                    geometry: Arc::new(Geometry::Points(asset.cloud.clone())),
                    base: object.transform,
                    placement: Placement::default(),
                    visible: true,
                    parent: None,
                    explode_direction: [0.; 3],
                    unit: format!("{:?}", asset.unit),
                    frame: "Destro / Z-up declarado".into(),
                });
            }
        }
        SourceKind::Run | SourceKind::RunSurface => {
            let m = noah_pipeline::inspect(&source.path)?;
            if matches!(
                m.status,
                noah_pipeline::Status::Running | noah_pipeline::Status::Prepared
            ) {
                return Err("aguarde o término do run antes de abrir".into());
            }
            if !m.artifacts.iter().any(|a| a.path == "exports/sparse.ply") {
                return Err("run sem nuvem esparsa registrada".into());
            }
            let surface = prefer_surface
                && m.artifacts
                    .iter()
                    .any(|a| a.path == "exports/mesh.obj" && a.kind == "surface_mesh_obj")
                && m.artifacts.iter().any(|a| {
                    a.path == "exports/mesh.metadata.json" && a.kind == "surface_provenance"
                });
            let relative = if surface {
                "exports/mesh.obj"
            } else {
                "exports/sparse.ply"
            };
            let bytes = read_bounded(&source.path.join(relative), noah_storage::MAX_PLY_BYTES)?;
            let geometry = if surface {
                source.kind = SourceKind::RunSurface;
                Geometry::Mesh(noah_storage::obj::parse(&bytes)?)
            } else {
                Geometry::Points(noah_storage::ply::parse(&bytes)?)
            };
            items.push(Item {
                id: if surface {
                    format!("run:{}:surface", m.run_id)
                } else {
                    format!("run:{}", m.run_id)
                },
                name: format!(
                    "{name} / {}",
                    if surface {
                        "superfície sem textura"
                    } else {
                        "esparsa"
                    }
                ),
                source: source_index,
                geometry: Arc::new(geometry),
                base: Transform::default(),
                placement: Placement::default(),
                visible: true,
                parent: None,
                explode_direction: [0.; 3],
                unit: "Unknown".into(),
                frame: format!("SfM arbitrário / run {:?}", m.status),
            });
        }
        SourceKind::Ply | SourceKind::Obj => {
            let bytes = read_bounded(&source.path, noah_storage::MAX_PLY_BYTES)?;
            let geometry = if source.kind == SourceKind::Ply {
                Geometry::Points(noah_storage::ply::parse(&bytes)?)
            } else {
                Geometry::Mesh(noah_storage::obj::parse(&bytes)?)
            };
            items.push(Item {
                id: format!("file:{}", source.sha256),
                name,
                source: source_index,
                geometry: Arc::new(geometry),
                base: Transform::default(),
                placement: Placement::default(),
                visible: true,
                parent: None,
                explode_direction: [0.; 3],
                unit: "Unknown".into(),
                frame: "Referencial do arquivo / não calibrado".into(),
            });
        }
    }
    for item in &items {
        item.validate()?;
    }
    Ok((source, items))
}
/// Primitivas projetadas compartilhadas por renderer e seleção; amostra global limitada.
#[derive(Debug, Clone)]
pub struct Primitive {
    pub item: usize,
    pub pixels: Vec<[f64; 2]>,
    pub depth: f64,
    pub color: [u8; 3],
}
pub fn project(scene: &Scene, size: [f64; 2]) -> Vec<Primitive> {
    let point_count: usize = scene
        .items
        .iter()
        .filter(|i| i.visible)
        .map(|i| match i.geometry.as_ref() {
            Geometry::Points(p) => p.positions().len(),
            _ => 0,
        })
        .sum();
    let face_count: usize = scene
        .items
        .iter()
        .filter(|i| i.visible)
        .map(|i| match i.geometry.as_ref() {
            Geometry::Mesh(m) => m.triangles().len(),
            _ => 0,
        })
        .sum();
    let ps = point_count.div_ceil(DISPLAY_POINTS).max(1);
    let fs = face_count.div_ceil(DISPLAY_FACES).max(1);
    let Ok(transforms) = scene.world_transforms(true) else {
        return vec![];
    };
    let mut output = Vec::new();
    let mut point_index = 0;
    let mut face_index = 0;
    for (item_index, item) in scene.items.iter().enumerate() {
        if !item.visible {
            continue;
        }
        let t = transforms[item_index];
        let projected = |p: [f64; 3]| t.apply(p).ok().and_then(|p| scene.camera.project(p, size));
        match item.geometry.as_ref() {
            Geometry::Points(cloud) => {
                for (i, &p) in cloud.positions().iter().enumerate() {
                    let show = point_index % ps == 0;
                    point_index += 1;
                    if !show {
                        continue;
                    }
                    if let Some(p) = projected(p)
                        && p[0] >= 0.
                        && p[0] <= size[0]
                        && p[1] >= 0.
                        && p[1] <= size[1]
                    {
                        output.push(Primitive {
                            item: item_index,
                            pixels: vec![[p[0], p[1]]],
                            depth: p[2],
                            color: cloud.colors().map(|c| c[i]).unwrap_or([125, 212, 218]),
                        });
                    }
                }
            }
            Geometry::Mesh(mesh) => {
                for face in mesh.triangles() {
                    let show = face_index % fs == 0;
                    face_index += 1;
                    if !show {
                        continue;
                    }
                    let points: Option<Vec<_>> = face
                        .iter()
                        .map(|&i| projected(mesh.positions()[i as usize]))
                        .collect();
                    if let Some(points) = points {
                        let pixels: Vec<_> = points.iter().map(|p| [p[0], p[1]]).collect();
                        if pixels.iter().all(|p| p[0] < 0.)
                            || pixels.iter().all(|p| p[0] > size[0])
                            || pixels.iter().all(|p| p[1] < 0.)
                            || pixels.iter().all(|p| p[1] > size[1])
                        {
                            continue;
                        }
                        output.push(Primitive {
                            item: item_index,
                            pixels,
                            depth: points.iter().map(|p| p[2]).sum::<f64>() / 3.,
                            color: {
                                let palette = [
                                    [96, 156, 181],
                                    [155, 174, 189],
                                    [225, 173, 93],
                                    [103, 189, 171],
                                    [134, 154, 206],
                                ];
                                let base = palette[item_index % palette.len()];
                                let vertices = face.map(|i| mesh.positions()[i as usize]);
                                let a = camera::sub(vertices[1], vertices[0]);
                                let b = camera::sub(vertices[2], vertices[0]);
                                let n = [
                                    a[1] * b[2] - a[2] * b[1],
                                    a[2] * b[0] - a[0] * b[2],
                                    a[0] * b[1] - a[1] * b[0],
                                ];
                                let norm = camera::dot(n, n).sqrt();
                                let shade = if norm > 1e-12 {
                                    0.55 + 0.45 * camera::dot(n, [0.25, -0.45, 0.85]).abs() / norm
                                } else {
                                    0.65
                                };
                                base.map(|v| (v as f64 * shade.min(1.)) as u8)
                            },
                        });
                    }
                }
            }
        }
    }
    output.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    output
}
fn inside_triangle(p: [f64; 2], tri: &[[f64; 2]]) -> bool {
    let cross =
        |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
    let a = cross(tri[0], tri[1]);
    let b = cross(tri[1], tri[2]);
    let c = cross(tri[2], tri[0]);
    let area = (tri[1][0] - tri[0][0]) * (tri[2][1] - tri[0][1])
        - (tri[1][1] - tri[0][1]) * (tri[2][0] - tri[0][0]);
    area.abs() > 1e-9 && ((a >= 0. && b >= 0. && c >= 0.) || (a <= 0. && b <= 0. && c <= 0.))
}
pub fn pick(primitives: &[Primitive], point: [f64; 2], radius: f64) -> Option<usize> {
    primitives
        .iter()
        .rev()
        .find(|p| {
            if p.pixels.len() == 1 {
                (p.pixels[0][0] - point[0]).powi(2) + (p.pixels[0][1] - point[1]).powi(2)
                    <= radius.powi(2)
            } else {
                inside_triangle(point, &p.pixels)
            }
        })
        .map(|p| p.item)
}

/// Exporta uma peça no estado montado; deslocamento da vista explodida é ignorado.
pub fn export_item(scene: &Scene, index: usize, path: &Path) -> Result<()> {
    let item = scene
        .items
        .get(index)
        .ok_or("selecione uma peça para exportar")?;
    let t = scene.world_transforms(false)?[index];
    let positions = item
        .geometry
        .positions()
        .iter()
        .map(|&p| t.apply(p))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let bytes = match item.geometry.as_ref() {
        Geometry::Points(cloud) => noah_storage::ply::encode(&PointCloud::new(
            positions,
            cloud.colors().map(|c| c.to_vec()),
        )?),
        Geometry::Mesh(mesh) => {
            let mut text = String::new();
            for p in positions {
                text.push_str(&format!("v {} {} {}\n", p[0], p[1], p[2]));
            }
            for face in mesh.triangles() {
                text.push_str(&format!(
                    "f {} {} {}\n",
                    face[0] + 1,
                    face[1] + 1,
                    face[2] + 1
                ));
            }
            text.into_bytes()
        }
    };
    let meta = path.with_file_name(format!(
        "{}.metadata.json",
        path.file_name().ok_or("nome ausente")?.to_string_lossy()
    ));
    if path.exists() || meta.exists() {
        return Err("arquivo de destino/metadados já existe".into());
    }
    let source = &scene.sources[item.source];
    let info = serde_json::to_vec_pretty(
        &serde_json::json!({"schema_version":1,"name":item.name,"unit":item.unit,"frame":item.frame,"source_sha256":source.sha256,"state":"assembled","explosion_applied":false}),
    )?;
    noah_pipeline::write_new(path, &bytes)?;
    noah_pipeline::write_new(&meta, &info)
}
