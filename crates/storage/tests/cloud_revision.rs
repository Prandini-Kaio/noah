use noah_geometry::PointCloud;
use noah_storage::cloud_revision::{self, SourceRef};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-clean-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn revisions_are_immutable_indexed_color_preserving_and_hash_verified() {
    let t = Temp::new();
    let c = PointCloud::new(
        vec![[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]],
        Some(vec![[1, 2, 3], [4, 5, 6], [7, 8, 9]]),
    )
    .unwrap();
    let src = SourceRef {
        path: t.0.join("source.ply"),
        kind: "ply".into(),
        sha256: "a".repeat(64),
        item_id: "source".into(),
        unit: "Unknown".into(),
        frame: "unknown".into(),
    };
    let out = t.0.join("revision");
    cloud_revision::create(&out, src.clone(), &c, vec![0, 2], vec![]).unwrap();
    let (m, cloud) = cloud_revision::load(&out).unwrap();
    assert_eq!(m.kept_indices, vec![0, 2]);
    assert_eq!(cloud.colors().unwrap(), &[[1, 2, 3], [7, 8, 9]]);
    assert_eq!(c.positions().len(), 3);
    assert!(cloud_revision::create(&out, src.clone(), &c, vec![0], vec![]).is_err());
    assert!(cloud_revision::create(&t.0.join("empty"), src, &c, vec![], vec![]).is_err());
    assert!(!t.0.join("empty").exists());
    assert!(cloud_revision::run_selection(&out).is_err());
    fs::write(out.join("cloud.ply"), noah_storage::ply::encode(&c)).unwrap();
    assert!(cloud_revision::load(&out).is_err());
}
#[test]
fn revision_chain_composes_original_sparse_indices() {
    let t = Temp::new();
    let c = PointCloud::new((0..5).map(|i| [i as f64, 0., 0.]).collect(), None).unwrap();
    let source = SourceRef {
        path: t.0.join("run"),
        kind: "run".into(),
        sha256: "a".repeat(64),
        item_id: "run:test".into(),
        unit: "unknown".into(),
        frame: "sfm".into(),
    };
    let a = t.0.join("a");
    cloud_revision::create(&a, source.clone(), &c, vec![0, 2, 3, 4], vec![]).unwrap();
    let (_, cloud) = cloud_revision::load(&a).unwrap();
    let parent = SourceRef {
        path: a.clone(),
        kind: "cloud_revision".into(),
        sha256: noah_pipeline_hash(&a.join("cloud-revision.json")),
        ..source
    };
    let b = t.0.join("b");
    cloud_revision::create(&b, parent, &cloud, vec![1, 3], vec![]).unwrap();
    assert_eq!(cloud_revision::run_selection(&b).unwrap().1, vec![2, 4]);
    fs::write(a.join("cloud-revision.json"), b"{}").unwrap();
    assert!(cloud_revision::run_selection(&b).is_err());
}
fn noah_pipeline_hash(p: &std::path::Path) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(fs::read(p).unwrap()))
}
