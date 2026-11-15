use noah_photogrammetry::comparison;
use std::{
    fs,
    path::{Path, PathBuf},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p =
            std::env::temp_dir().join(format!("noah-comparison-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(root: &Path, scale: f64, shift: [f64; 3], change: bool) {
    fs::create_dir(root).unwrap();
    fs::create_dir(root.join("images")).unwrap();
    fs::create_dir(root.join("model-0")).unwrap();
    fs::create_dir(root.join("exports")).unwrap();
    let mut inputs = Vec::new();
    let centers = [
        [0., 0., 0.],
        [2., 0., 0.],
        [0., 2., 0.],
        [0., 0., 2.],
        [1., 1., 1.],
    ];
    let mut images = String::new();
    for (i, c) in centers.iter().enumerate() {
        let name = format!("{i:04}.jpg");
        let bytes = format!("synthetic photo identity {i}").into_bytes();
        fs::write(root.join("images").join(&name), &bytes).unwrap();
        inputs.push(noah_pipeline::Input {
            original_name: name.clone(),
            snapshot: format!("images/{name}"),
            sha256: noah_storage::content_sha256(&bytes),
            bytes: bytes.len() as u64,
        });
        let center: [f64; 3] = std::array::from_fn(|a| c[a] * scale + shift[a]);
        images.push_str(&format!(
            "{} 1 0 0 0 {} {} {} 1 {}\n1 1 1\n",
            i + 1,
            -center[0],
            -center[1],
            -center[2],
            name
        ));
    }
    fs::write(
        root.join("model-0/cameras.txt"),
        "1 SIMPLE_RADIAL 800 600 700 400 300 0\n",
    )
    .unwrap();
    fs::write(root.join("model-0/images.txt"), images).unwrap();
    let mut points = String::new();
    for (i, p) in [
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [0., 0., 1.],
        [1., 1., 1.],
    ]
    .iter()
    .enumerate()
    {
        let mut p: [f64; 3] = std::array::from_fn(|a| p[a] * scale + shift[a]);
        if change && i == 4 {
            p[0] += 0.5 * scale;
        }
        points.push_str(&format!(
            "{} {} {} {} 255 0 0 0.1 1 0\n",
            i + 1,
            p[0],
            p[1],
            p[2]
        ));
    }
    fs::write(root.join("model-0/points3D.txt"), points).unwrap();
    fs::write(
        root.join("exports/reconstruction.json"),
        b"{\"selected_model\":\"0\"}",
    )
    .unwrap();
    let mut artifacts = Vec::new();
    for p in [
        "model-0/cameras.txt",
        "model-0/images.txt",
        "model-0/points3D.txt",
        "exports/reconstruction.json",
    ] {
        let (hash, bytes) = noah_pipeline::hash_file(&root.join(p), 64 * 1024 * 1024).unwrap();
        artifacts.push(noah_pipeline::Artifact {
            path: p.into(),
            kind: "fixture".into(),
            sha256: hash,
            bytes,
        });
    }
    let m = noah_pipeline::Manifest {
        schema_version: 1,
        run_id: root.file_name().unwrap().to_str().unwrap().into(),
        started_ms: 0,
        finished_ms: Some(1),
        status: noah_pipeline::Status::Succeeded,
        engine: "not executed".into(),
        engine_version: None,
        parameters: Default::default(),
        inputs,
        steps: vec![],
        artifacts,
        diagnostic: None,
    };
    fs::write(
        root.join("run.json"),
        serde_json::to_vec_pretty(&m).unwrap(),
    )
    .unwrap();
}
#[test]
fn aligns_common_photo_cameras_distances_reports_and_detects_tamper() {
    let t = Temp::new();
    let a = t.0.join("a");
    let b = t.0.join("b");
    run(&a, 1., [0.; 3], false);
    run(&b, 3., [8., -5., 2.], false);
    let c = comparison::compare(&a, &b, 0.1).unwrap();
    assert!(c.report.updated_to_reference.max < 1e-10);
    assert_eq!(c.report.registration.common_images, 5);
    assert!((c.report.registration.new_to_reference_scale - 1. / 3.).abs() < 1e-10);
    assert!(!c.report.metrology_validated && !c.report.physical_unit_known);
    let file = t.0.join("comparison.json");
    comparison::save(&c, &file).unwrap();
    assert_eq!(comparison::reopen(&file).unwrap().report, c.report);
    assert!(comparison::save(&c, &file).is_err());
    let d = t.0.join("changed");
    run(&d, 3., [8., -5., 2.], true);
    let c = comparison::compare(&a, &d, 0.1).unwrap();
    assert!((c.report.updated_to_reference.max - 0.5).abs() < 1e-10);
    assert_eq!(c.report.updated_to_reference.within_threshold_fraction, 0.8);
    assert!(comparison::compare(&a, &d, f64::NAN).is_err());
    fs::write(b.join("model-0/points3D.txt"), "tampered").unwrap();
    assert!(comparison::reopen(&file).is_err());
}
#[test]
fn identical_run_has_zero_distance_and_insufficient_common_images_rejected() {
    let t = Temp::new();
    let a = t.0.join("a");
    let b = t.0.join("b");
    run(&a, 1., [0.; 3], false);
    run(&b, 1., [0.; 3], false);
    assert!(
        comparison::compare(&a, &a, 0.)
            .unwrap()
            .report
            .reference_to_updated
            .max
            < 1e-12
    );
    let mut m: noah_pipeline::Manifest =
        serde_json::from_slice(&fs::read(b.join("run.json")).unwrap()).unwrap();
    for input in m.inputs.iter_mut().take(2) {
        fs::write(b.join(&input.snapshot), b"other").unwrap();
        input.sha256 = noah_storage::content_sha256(b"other");
        input.bytes = 5;
    }
    fs::write(b.join("run.json"), serde_json::to_vec(&m).unwrap()).unwrap();
    assert!(
        comparison::compare(&a, &b, 0.1)
            .err()
            .unwrap()
            .to_string()
            .contains("quatro")
    );
}
