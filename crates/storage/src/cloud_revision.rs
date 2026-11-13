//! Immutable cleaning revisions: local coordinates, original indices and recipe.
use crate::{Result, ply};
use noah_geometry::{PointCloud, cleaning};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    pub path: PathBuf,
    pub kind: String,
    pub sha256: String,
    pub item_id: String,
    pub unit: String,
    pub frame: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    pub schema_version: u32,
    pub id: String,
    pub source: SourceRef,
    pub original_sha256: String,
    pub cloud_sha256: String,
    pub original_count: usize,
    pub kept_indices: Vec<usize>,
    pub operations: Vec<serde_json::Value>,
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn write(root: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(name))?;
    f.write_all(bytes)?;
    Ok(())
}
fn read(root: &Path, name: &str, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let p = root.join(name);
    if !fs::symlink_metadata(&p)?.is_file() {
        return Err("revisão exige arquivo regular".into());
    }
    let f = fs::File::open(p)?;
    if f.metadata()?.len() > limit {
        return Err("revisão excede limite".into());
    }
    let mut b = Vec::new();
    f.take(limit + 1).read_to_end(&mut b)?;
    if b.len() as u64 > limit {
        return Err("revisão excede limite".into());
    }
    Ok(b)
}
pub fn create(
    output: &Path,
    source: SourceRef,
    cloud: &PointCloud,
    indices: Vec<usize>,
    operations: Vec<serde_json::Value>,
) -> Result<()> {
    cleaning::validate(cloud, &vec![true; cloud.positions().len()])?;
    if !source.path.is_absolute() || source.sha256.len() != 64 || operations.len() > 64 {
        return Err("fonte/receita de revisão inválida".into());
    }
    let cleaned = cleaning::subset(cloud, &indices)?;
    let original = ply::encode(cloud);
    let bytes = ply::encode(&cleaned);
    if original.len() as u64 > crate::MAX_PLY_BYTES || bytes.len() as u64 > crate::MAX_PLY_BYTES {
        return Err("nuvem excede limite".into());
    }
    let m = Revision {
        schema_version: 1,
        id: format!("{:032x}", crate::new_id()),
        source,
        original_sha256: hash(&original),
        cloud_sha256: hash(&bytes),
        original_count: cloud.positions().len(),
        kept_indices: indices,
        operations,
    };
    let meta = serde_json::to_vec_pretty(&m)?;
    if meta.len() > 16 * 1024 * 1024 {
        return Err("receita excede limite".into());
    }
    fs::create_dir(output)?;
    write(output, "original.ply", &original)?;
    write(output, "cloud.ply", &bytes)?;
    write(output, "cloud-revision.json", &meta)
}
pub fn load(root: &Path) -> Result<(Revision, PointCloud)> {
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err("diretório de revisão não pode ser link".into());
    }
    let m: Revision =
        serde_json::from_slice(&read(root, "cloud-revision.json", 16 * 1024 * 1024)?)?;
    if m.schema_version != 1
        || m.original_count > cleaning::MAX_CLEAN_POINTS
        || m.operations.len() > 64
        || m.id.len() != 32
        || !m.id.bytes().all(|v| v.is_ascii_hexdigit())
        || !m.source.path.is_absolute()
    {
        return Err("revisão incompatível".into());
    }
    let input = read(root, "original.ply", crate::MAX_PLY_BYTES)?;
    let output = read(root, "cloud.ply", crate::MAX_PLY_BYTES)?;
    if hash(&input) != m.original_sha256 || hash(&output) != m.cloud_sha256 {
        return Err("revisão adulterada".into());
    }
    let original = ply::parse(&input)?;
    let cloud = ply::parse(&output)?;
    if original.positions().len() != m.original_count
        || cleaning::subset(&original, &m.kept_indices)? != cloud
    {
        return Err("índices da revisão não correspondem à nuvem".into());
    }
    Ok((m, cloud))
}

/// Resolve a bounded immutable revision chain to original sparse-run indices.
pub fn run_selection(root: &Path) -> Result<(SourceRef, Vec<usize>)> {
    resolve(root, 0)
}
fn resolve(root: &Path, depth: usize) -> Result<(SourceRef, Vec<usize>)> {
    if depth >= 16 {
        return Err("cadeia de revisão excede 16 níveis".into());
    }
    let (m, _) = load(root)?;
    if m.source.kind == "run" {
        return Ok((m.source, m.kept_indices));
    }
    if m.source.kind != "cloud_revision" {
        return Err("superfície requer origem em run sparse, não PLY/projeto isolado".into());
    }
    let parent = &m.source.path;
    if hash(&read(parent, "cloud-revision.json", 16 * 1024 * 1024)?) != m.source.sha256 {
        return Err("revisão pai alterada".into());
    }
    let (_, parent_cloud) = load(parent)?;
    let original = ply::parse(&read(root, "original.ply", crate::MAX_PLY_BYTES)?)?;
    if original != parent_cloud {
        return Err("pontos não correspondem à revisão pai".into());
    }
    let (source, parent_indices) = resolve(parent, depth + 1)?;
    let indices = m
        .kept_indices
        .iter()
        .map(|&i| {
            parent_indices
                .get(i)
                .copied()
                .ok_or("índice fora da revisão pai")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((source, indices))
}
