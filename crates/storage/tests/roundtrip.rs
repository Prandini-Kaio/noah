use noah_geometry::{Transform, UnitState};
use noah_storage::{StoredProject, new_id, ply};
use std::{fs, path::PathBuf};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-test-{:032x}", new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn root(&self) -> PathBuf {
        self.0.join("project")
    }
    fn input(&self) -> PathBuf {
        let p = self.0.join("input.ply");
        fs::write(
            &p,
            include_bytes!("../../../tests/datasets/reference-rgb.ply"),
        )
        .unwrap();
        p
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn roundtrip_and_transformed_export_preserve_original() {
    for unit in [UnitState::Unknown, UnitState::Metre, UnitState::Millimetre] {
        let temp = Temp::new();
        let input = temp.input();
        let original = fs::read(&input).unwrap();
        let mut stored = StoredProject::create(&temp.root(), "Referência".into()).unwrap();
        let id = stored.import(&input, "Peça".into(), unit).unwrap();
        let transform =
            Transform::from_euler([3., 4., 5.], [0., 0., std::f64::consts::FRAC_PI_2], 2.).unwrap();
        stored.transform(id, transform).unwrap();
        let reopened = StoredProject::open(&temp.root()).unwrap();
        assert_eq!(stored.project(), reopened.project());
        assert_eq!(reopened.project().assets()[0].unit, unit);
        let output = temp.0.join("out.ply");
        reopened.export(id, &output).unwrap();
        let cloud = ply::parse(&fs::read(&output).unwrap()).unwrap();
        for (actual, expected) in cloud
            .positions()
            .iter()
            .flatten()
            .zip([3., 6., 9., 1., 4., 5., 3., 4., 5.])
        {
            assert!((actual - expected).abs() < 1e-12);
        }
        assert_eq!(
            cloud.colors(),
            Some([[255, 0, 0], [0, 255, 0], [0, 0, 255]].as_slice())
        );
        assert_eq!(fs::read(&input).unwrap(), original);
        assert!(reopened.export(id, &input).is_err());
        assert_eq!(fs::read(&input).unwrap(), original);
    }
}
#[test]
fn invalid_import_and_transform_do_not_change_manifest() {
    let temp = Temp::new();
    let input = temp.input();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    let id = stored
        .import(&input, "Peça".into(), UnitState::Unknown)
        .unwrap();
    let manifest = temp.root().join("project.json");
    let original = fs::read(&manifest).unwrap();
    fs::write(&input, "ply\nformat binary_little_endian 1.0\n").unwrap();
    assert!(
        stored
            .import(&input, "Invalid".into(), UnitState::Metre)
            .is_err()
    );
    assert_eq!(fs::read(&manifest).unwrap(), original);
    let overflow = Transform::new([f64::MAX; 3], 0., f64::MAX).unwrap();
    assert!(stored.transform(id, overflow).is_err());
    assert_eq!(fs::read(&manifest).unwrap(), original);
    assert_eq!(
        stored.project(),
        StoredProject::open(&temp.root()).unwrap().project()
    );
}
#[test]
fn schema_corruption_hashes_and_references_are_rejected() {
    let temp = Temp::new();
    let input = temp.input();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    stored
        .import(&input, "Peça".into(), UnitState::Unknown)
        .unwrap();
    let path = temp.root().join("project.json");
    let original = fs::read(&path).unwrap();
    for field in ["schema_version", "asset_id", "rotation"] {
        let mut json: serde_json::Value = serde_json::from_slice(&original).unwrap();
        match field {
            "schema_version" => json[field] = 99.into(),
            "asset_id" => json["objects"][0][field] = "00000000000000000000000000000000".into(),
            _ => json["objects"][0][field] = serde_json::json!([[0, 0, 0], [0, 0, 0], [0, 0, 0]]),
        }
        fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        assert!(StoredProject::open(&temp.root()).is_err());
    }
    fs::write(&path, b"{broken").unwrap();
    assert!(StoredProject::open(&temp.root()).is_err());
    fs::write(&path, &original).unwrap();
    let raw = fs::read_dir(temp.root().join("raw"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&raw, b"changed").unwrap();
    assert!(StoredProject::open(&temp.root()).is_err());
}
#[test]
fn parser_rejects_unsupported_truncated_and_nonfinite() {
    let valid = include_str!("../../../tests/datasets/reference-rgb.ply");
    for bad in [
        valid.replace("format ascii 1.0", "format binary_little_endian 1.0"),
        valid.replace("element vertex 3", "element vertex 1000001"),
        valid.replace("1 0 2 255 0 0", "NaN 0 2 255 0 0"),
        valid.replace("1 0 2 255 0 0", "1 0 2 256 0 0"),
        valid.replace("property uchar blue\n", ""),
        valid.replace("end_header", "element face 0\nend_header"),
        valid.replace("0 0 0 0 0 255\n", ""),
        format!("{valid}1 2 3\n"),
    ] {
        assert!(ply::parse(bad.as_bytes()).is_err(), "accepted {bad}");
    }
    let xyz=b"ply\nformat ascii 1.0\nelement vertex 0\nproperty float x\nproperty float y\nproperty float z\nend_header\n";
    assert!(ply::parse(xyz).unwrap().positions().is_empty());
    let exact=b"ply\nformat ascii 1.0\nelement vertex 1\nproperty double x\nproperty double y\nproperty double z\nend_header\n0.12345678901234568 -1e-200 1e200\n";
    let cloud = ply::parse(exact).unwrap();
    assert_eq!(ply::parse(&ply::encode(&cloud)).unwrap(), cloud);
}
#[cfg(unix)]
#[test]
fn symlink_assets_are_rejected() {
    let temp = Temp::new();
    let input = temp.input();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    stored
        .import(&input, "Peça".into(), UnitState::Unknown)
        .unwrap();
    let raw = fs::read_dir(temp.root().join("raw"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::remove_file(&raw).unwrap();
    std::os::unix::fs::symlink(&input, &raw).unwrap();
    assert!(StoredProject::open(&temp.root()).is_err());
}

#[test]
fn failed_save_rolls_back_session_and_keeps_previous_manifest() {
    let temp = Temp::new();
    let input = temp.input();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    let path = temp.root().join("project.json");
    let before = fs::read(&path).unwrap();
    // Limite de metadados excedido após criar assets; manifesto anterior permanece.
    assert!(
        stored
            .import(&input, "x".repeat(4 * 1024 * 1024), UnitState::Unknown)
            .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(stored.project().assets().is_empty());
    assert_eq!(
        stored.project(),
        StoredProject::open(&temp.root()).unwrap().project()
    );
}

#[test]
fn nontrivial_f64_roundtrip_and_binary_corruption() {
    use sha2::{Digest, Sha256};
    let temp = Temp::new();
    let input = temp.input();
    fs::write(&input,"ply\nformat ascii 1.0\nelement vertex 1\nproperty double x\nproperty double y\nproperty double z\nend_header\n0.12345678901234568 -1e-200 1e200\n").unwrap();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    let id = stored
        .import(&input, "Peça".into(), UnitState::Unknown)
        .unwrap();
    stored
        .transform(
            id,
            Transform::from_euler(
                [0.12345678901234568, 0., 0.],
                [0.123456789, 0.7654321, 0.2468],
                0.12345678901234568,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        stored.project(),
        StoredProject::open(&temp.root()).unwrap().project()
    );
    let file = fs::read_dir(temp.root().join("derived"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut bytes = fs::read(&file).unwrap();
    bytes.truncate(17);
    fs::write(&file, &bytes).unwrap();
    // Mesmo com hash coerente, decoder deve detectar comprimento inválido.
    let manifest = temp.root().join("project.json");
    let mut json: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    json["sources"][0]["cloud_sha256"] = format!("{:x}", Sha256::digest(&bytes)).into();
    fs::write(&manifest, serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(StoredProject::open(&temp.root()).is_err());
}

#[test]
fn oversized_files_and_headers_are_rejected_before_import() {
    let temp = Temp::new();
    let input = temp.0.join("huge.ply");
    let file = fs::File::create(&input).unwrap();
    file.set_len(noah_storage::MAX_PLY_BYTES + 1).unwrap();
    let mut stored = StoredProject::create(&temp.root(), "Test".into()).unwrap();
    assert!(
        stored
            .import(&input, "Huge".into(), UnitState::Unknown)
            .is_err()
    );
    assert!(stored.project().assets().is_empty());
    let header = format!(
        "ply\nformat ascii 1.0\n{}end_header\n",
        format!("comment {}\n", "x".repeat(1000)).repeat(70)
    );
    assert!(ply::parse(header.as_bytes()).is_err());
    let line = format!(
        "ply\nformat ascii 1.0\ncomment {}\nend_header\n",
        "x".repeat(4097)
    );
    assert!(ply::parse(line.as_bytes()).is_err());
}
