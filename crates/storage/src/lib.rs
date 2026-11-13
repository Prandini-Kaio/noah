//! Persistência E1: schema explícito, limites e dados brutos preservados.
use noah_core::{AssetId, GeometryAsset, Object, ObjectId, Project, ProjectId, ReferenceFrame};
use noah_geometry::{PointCloud, Transform, UnitState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    error::Error,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub mod colmap_ply;
pub mod obj;
pub mod ply;
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub const MAX_POINTS: usize = 1_000_000;
pub const MAX_PLY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_META: u64 = 4 * 1024 * 1024;
fn fail<T>(message: &str) -> Result<T> {
    Err(message.into())
}
pub fn new_id() -> u128 {
    uuid::Uuid::new_v4().as_u128()
}
fn hex(id: u128) -> String {
    format!("{id:032x}")
}
fn id(text: &str) -> Result<u128> {
    if text.len() != 32
        || !text
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return fail("ID inválido");
    }
    Ok(u128::from_str_radix(text, 16)?)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    asset_id: String,
    raw_sha256: String,
    cloud_sha256: String,
    source_name: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    id: String,
    name: String,
    objects: Vec<ObjectDto>,
    assets: Vec<AssetDto>,
    sources: Vec<Source>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetDto {
    id: String,
    unit: String,
    frame: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectDto {
    id: String,
    name: String,
    asset_id: String,
    translation: [f64; 3],
    rotation: [[f64; 3]; 3],
    scale: f64,
}

/// Sessão de um projeto. E1 pressupõe um único escritor por diretório.
pub struct StoredProject {
    root: PathBuf,
    project: Project,
    sources: Vec<Source>,
}
impl StoredProject {
    pub fn project(&self) -> &Project {
        &self.project
    }
    pub fn create(root: &Path, name: String) -> Result<Self> {
        fs::create_dir(root)?;
        fs::create_dir(root.join("raw"))?;
        fs::create_dir(root.join("derived"))?;
        let stored = Self {
            root: root.to_path_buf(),
            project: Project::new(ProjectId(new_id()), name),
            sources: vec![],
        };
        stored.save()?;
        Ok(stored)
    }
    pub fn open(root: &Path) -> Result<Self> {
        check_dir(root)?;
        check_dir(&root.join("raw"))?;
        check_dir(&root.join("derived"))?;
        let manifest: Manifest =
            serde_json::from_slice(&read_limited(&root.join("project.json"), MAX_META)?)?;
        if manifest.schema_version != 1 {
            return fail("schema de projeto não suportado (esperado 1)");
        }
        if manifest.assets.len() > 1000
            || manifest.objects.len() > 1000
            || manifest.sources.len() != manifest.assets.len()
        {
            return fail("limite ou quantidade de entidades inválido");
        }
        let mut project = Project::new(ProjectId(id(&manifest.id)?), manifest.name);
        let mut source_ids = HashSet::new();
        for source in &manifest.sources {
            id(&source.asset_id)?;
            if !source_ids.insert(&source.asset_id) {
                return fail("proveniência duplicada");
            }
        }
        let mut total = 0;
        for asset in manifest.assets {
            let asset_id = AssetId(id(&asset.id)?);
            let source = manifest
                .sources
                .iter()
                .find(|s| s.asset_id == asset.id)
                .ok_or("proveniência ausente")?;
            let raw = read_limited(
                &root.join("raw").join(format!("{}.ply", asset.id)),
                MAX_PLY_BYTES,
            )?;
            if digest(&raw) != source.raw_sha256 {
                return fail("hash do arquivo bruto não corresponde");
            }
            let bytes = read_limited(
                &root.join("derived").join(format!("{}.npc", asset.id)),
                27 * MAX_POINTS as u64 + 17,
            )?;
            if digest(&bytes) != source.cloud_sha256 {
                return fail("hash da nuvem não corresponde");
            }
            let cloud = decode_cloud(&bytes)?;
            total += cloud.positions().len();
            if total > MAX_POINTS {
                return fail("projeto excede limite total de pontos");
            }
            if ply::parse(&raw)? != cloud {
                return fail("nuvem interna não corresponde à origem PLY");
            }
            let unit = match asset.unit.as_str() {
                "unknown" => UnitState::Unknown,
                "m" => UnitState::Metre,
                "mm" => UnitState::Millimetre,
                _ => return fail("unidade não suportada"),
            };
            if asset.frame != "right-handed-z-up" {
                return fail("referencial não suportado");
            }
            project
                .add_asset(GeometryAsset {
                    id: asset_id,
                    cloud,
                    unit,
                    frame: ReferenceFrame::RightHandedZUp,
                })
                .map_err(|e| format!("asset inválido: {e:?}"))?;
        }
        for object in manifest.objects {
            project
                .add_object(Object {
                    id: ObjectId(id(&object.id)?),
                    name: object.name,
                    asset_id: AssetId(id(&object.asset_id)?),
                    transform: Transform::from_parts(
                        object.translation,
                        object.rotation,
                        object.scale,
                    )?,
                })
                .map_err(|e| format!("objeto inválido: {e:?}"))?;
        }
        let stored = Self {
            root: root.to_path_buf(),
            project,
            sources: manifest.sources,
        };
        stored.validate_transforms()?;
        Ok(stored)
    }
    fn manifest(&self) -> Manifest {
        Manifest {
            schema_version: 1,
            id: hex(self.project.id.0),
            name: self.project.name.clone(),
            assets: self
                .project
                .assets()
                .iter()
                .map(|a| AssetDto {
                    id: hex(a.id.0),
                    unit: match a.unit {
                        UnitState::Unknown => "unknown",
                        UnitState::Metre => "m",
                        UnitState::Millimetre => "mm",
                    }
                    .into(),
                    frame: "right-handed-z-up".into(),
                })
                .collect(),
            objects: self
                .project
                .objects()
                .iter()
                .map(|o| ObjectDto {
                    id: hex(o.id.0),
                    name: o.name.clone(),
                    asset_id: hex(o.asset_id.0),
                    translation: o.transform.translation(),
                    rotation: o.transform.rotation(),
                    scale: o.transform.scale(),
                })
                .collect(),
            sources: self.sources.clone(),
        }
    }
    fn validate_transforms(&self) -> Result<()> {
        for object in self.project.objects() {
            let asset = self
                .project
                .assets()
                .iter()
                .find(|a| a.id == object.asset_id)
                .ok_or("asset ausente")?;
            for &p in asset.cloud.positions() {
                object.transform.apply(p)?;
            }
        }
        Ok(())
    }
    fn save(&self) -> Result<()> {
        self.validate_transforms()?;
        let bytes = serde_json::to_vec_pretty(&self.manifest())?;
        if bytes.len() as u64 > MAX_META {
            return fail("metadados excedem limite");
        }
        let target = self.root.join("project.json");
        if fs::symlink_metadata(&target).is_ok() {
            check_file(&target)?;
        }
        let temp = self.root.join(format!(".project-{}.tmp", hex(new_id())));
        let result = (|| {
            write_new(&temp, &bytes)?;
            fs::rename(&temp, &target)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
    /// Importa um PLY declarado no referencial canônico; sem converter unidade.
    pub fn import(&mut self, input: &Path, name: String, unit: UnitState) -> Result<ObjectId> {
        check_dir(&self.root.join("raw"))?;
        check_dir(&self.root.join("derived"))?;
        let raw = read_limited(input, MAX_PLY_BYTES)?;
        let cloud = ply::parse(&raw)?;
        let total: usize = self
            .project
            .assets()
            .iter()
            .map(|a| a.cloud.positions().len())
            .sum();
        if self.project.assets().len() >= 1000
            || self.project.objects().len() >= 1000
            || total + cloud.positions().len() > MAX_POINTS
        {
            return fail("limite do projeto excedido");
        }
        let asset_id = AssetId(new_id());
        let object_id = ObjectId(new_id());
        let key = hex(asset_id.0);
        let bytes = encode_cloud(&cloud);
        write_new(&self.root.join("raw").join(format!("{key}.ply")), &raw)?;
        write_new(
            &self.root.join("derived").join(format!("{key}.npc")),
            &bytes,
        )?;
        let old = self.project.clone();
        self.project
            .add_asset(GeometryAsset {
                id: asset_id,
                cloud,
                unit,
                frame: ReferenceFrame::RightHandedZUp,
            })
            .map_err(|e| format!("{e:?}"))?;
        self.project
            .add_object(Object {
                id: object_id,
                name,
                asset_id,
                transform: Transform::default(),
            })
            .map_err(|e| format!("{e:?}"))?;
        self.sources.push(Source {
            asset_id: key,
            raw_sha256: digest(&raw),
            cloud_sha256: digest(&bytes),
            source_name: input
                .file_name()
                .ok_or("nome da origem ausente")?
                .to_string_lossy()
                .into_owned(),
        });
        if let Err(error) = self.save() {
            self.project = old;
            self.sources.pop();
            return Err(error);
        }
        Ok(object_id)
    }
    pub fn transform(&mut self, object: ObjectId, transform: Transform) -> Result<()> {
        let old = self.project.clone();
        if !self.project.set_transform(object, transform) {
            return fail("objeto não encontrado");
        }
        if let Err(error) = self.save() {
            self.project = old;
            return Err(error);
        }
        Ok(())
    }
    /// Exporta coordenadas mundiais para arquivo novo. Unidade e frame são do asset.
    pub fn export(&self, object_id: ObjectId, output: &Path) -> Result<()> {
        let object = self
            .project
            .objects()
            .iter()
            .find(|o| o.id == object_id)
            .ok_or("objeto não encontrado")?;
        let asset = self
            .project
            .assets()
            .iter()
            .find(|a| a.id == object.asset_id)
            .ok_or("asset ausente")?;
        let points = asset
            .cloud
            .positions()
            .iter()
            .map(|&p| object.transform.apply(p))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let cloud = PointCloud::new(points, asset.cloud.colors().map(|c| c.to_vec()))?;
        write_new(output, &ply::encode(&cloud))
    }
}
fn check_dir(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return fail("diretório inválido ou symlink");
    }
    Ok(())
}
fn check_file(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return fail("arquivo inválido ou symlink");
    }
    Ok(())
}
fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    check_file(path)?;
    let file = File::open(path)?;
    if file.metadata()?.len() > limit {
        return fail("arquivo excede limite de tamanho");
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return fail("arquivo excede limite de tamanho");
    }
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn encode_cloud(cloud: &PointCloud) -> Vec<u8> {
    let mut bytes = b"NOAHPC01".to_vec();
    bytes.extend_from_slice(&(cloud.positions().len() as u64).to_le_bytes());
    bytes.push(u8::from(cloud.colors().is_some()));
    for (i, p) in cloud.positions().iter().enumerate() {
        for v in p {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        if let Some(colors) = cloud.colors() {
            bytes.extend_from_slice(&colors[i]);
        }
    }
    bytes
}
fn decode_cloud(bytes: &[u8]) -> Result<PointCloud> {
    if bytes.len() < 17 || &bytes[..8] != b"NOAHPC01" || bytes[16] > 1 {
        return fail("nuvem interna inválida");
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into()?);
    if count > MAX_POINTS as u64 {
        return fail("nuvem excede limite de pontos");
    }
    let rgb = bytes[16] == 1;
    let stride = if rgb { 27 } else { 24 };
    if bytes.len() != 17 + count as usize * stride {
        return fail("nuvem interna truncada ou excedente");
    }
    let mut points = Vec::with_capacity(count as usize);
    let mut colors = rgb.then(|| Vec::with_capacity(count as usize));
    for chunk in bytes[17..].chunks_exact(stride) {
        points.push([
            f64::from_le_bytes(chunk[..8].try_into()?),
            f64::from_le_bytes(chunk[8..16].try_into()?),
            f64::from_le_bytes(chunk[16..24].try_into()?),
        ]);
        if let Some(colors) = &mut colors {
            colors.push(chunk[24..27].try_into()?);
        }
    }
    Ok(PointCloud::new(points, colors)?)
}
