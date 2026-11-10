use noah_workspace::{
    Placement, Scene,
    measurement::{self, Anchor, LengthUnit},
    session,
};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-measure-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn scene(&self) -> Scene {
        let p = self.0.join("points.ply");
        fs::write(
            &p,
            include_bytes!("../../../tests/datasets/reference-rgb.ply"),
        )
        .unwrap();
        let mut s = Scene::default();
        s.add_source(&p).unwrap();
        s
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn anchor(s: &Scene, vertex: usize) -> Anchor {
    Anchor {
        item_id: s.items[0].id.clone(),
        vertex,
    }
}
#[test]
fn known_distance_scale_and_unknown_uncertainty() {
    let t = Temp::new();
    let mut s = t.scene();
    let a = anchor(&s, 1);
    let b = anchor(&s, 2);
    let c = anchor(&s, 0);
    let d = s.distance(&a, &b).unwrap();
    assert_eq!(d.value, 1.);
    assert_eq!(d.unit, None);
    s.calibrate(
        a.clone(),
        b.clone(),
        100.,
        LengthUnit::Millimetre,
        "Régua de referência informada".into(),
    )
    .unwrap();
    let d = s.distance(&c, &b).unwrap();
    assert!((d.value - 5f64.sqrt() * 100.).abs() < 1e-10);
    assert_eq!(d.unit, Some(LengthUnit::Millimetre));
    s.add_measurement(c, b, "Diagonal".into()).unwrap();
    assert_eq!(s.measurements[0].uncertainty, None);
}
#[test]
fn explosion_and_rigid_motion_do_not_change_measurement_but_scale_change_is_stale() {
    let t = Temp::new();
    let mut s = t.scene();
    let a = anchor(&s, 1);
    let b = anchor(&s, 2);
    s.calibrate(
        a.clone(),
        b.clone(),
        1.,
        LengthUnit::Metre,
        "Distância conhecida".into(),
    )
    .unwrap();
    s.explosion = 1.;
    s.explosion_distance = 25.;
    assert_eq!(s.distance(&a, &b).unwrap().value, 1.);
    s.set_placement(
        0,
        Placement {
            translation: [10., 20., 30.],
            angles: [0., 0., 0.4],
            ..Default::default()
        },
    )
    .unwrap();
    assert!((s.distance(&a, &b).unwrap().value - 1.).abs() < 1e-12);
    s.set_placement(
        0,
        Placement {
            scale: 2.,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(s.distance(&a, &b).is_err());
    s.calibrate(
        a.clone(),
        b.clone(),
        1.,
        LengthUnit::Metre,
        "Recalibração".into(),
    )
    .unwrap();
    assert_eq!(s.distance(&a, &b).unwrap().value, 1.);
}
#[test]
fn persistence_and_report_keep_anchors_provenance_and_stale_state() {
    let t = Temp::new();
    let mut s = t.scene();
    let a = anchor(&s, 1);
    let b = anchor(&s, 2);
    s.calibrate(
        a.clone(),
        b.clone(),
        100.,
        LengthUnit::Millimetre,
        "Paquímetro declarado".into(),
    )
    .unwrap();
    s.add_measurement(a, b, "Referência AB".into()).unwrap();
    let file = t.0.join("workspace.json");
    session::save(&s, &file).unwrap();
    let restored = session::load(&file).unwrap();
    assert_eq!(restored.calibration, s.calibration);
    assert_eq!(restored.measurements, s.measurements);
    let report = t.0.join("report.json");
    measurement::export_report(&s, &report).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
    assert_eq!(json["measurements"][0]["unit"], "mm");
    assert_eq!(json["measurements"][0]["value"], 100.);
    assert_eq!(
        json["measurements"][0]["uncertainty"],
        serde_json::Value::Null
    );
    assert_eq!(json["metrology_validated"], false);
    assert!(measurement::export_report(&s, &report).is_err());
    s.set_placement(
        0,
        Placement {
            scale: 2.,
            ..Default::default()
        },
    )
    .unwrap();
    let stale = t.0.join("stale.json");
    session::save(&s, &stale).unwrap();
    let opened = session::load(&stale).unwrap();
    assert!(opened.scale_factor().is_err());
    assert!(measurement::export_report(&opened, &t.0.join("invalid-report.json")).is_err());
}
#[test]
fn invalid_references_and_corrupted_anchors_are_rejected() {
    let t = Temp::new();
    let mut s = t.scene();
    let a = anchor(&s, 1);
    let b = anchor(&s, 2);
    for distance in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(
            s.calibrate(
                a.clone(),
                b.clone(),
                distance,
                LengthUnit::Metre,
                "Teste".into()
            )
            .is_err()
        );
    }
    assert!(
        s.calibrate(a.clone(), a.clone(), 1., LengthUnit::Metre, "Teste".into())
            .is_err()
    );
    assert!(
        s.calibrate(a.clone(), b.clone(), 1., LengthUnit::Metre, "".into())
            .is_err()
    );
    assert!(s.raw_distance(&anchor(&s, 99), &b).is_err());
    s.add_measurement(a, b, "M".into()).unwrap();
    let p = t.0.join("session.json");
    session::save(&s, &p).unwrap();
    let mut data: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    data["measurements"][0]["a"]["vertex"] = 9999.into();
    fs::write(&p, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(session::load(&p).is_err());
}
#[test]
fn screen_pick_snaps_to_original_vertex() {
    let t = Temp::new();
    let s = t.scene();
    let a = anchor(&s, 1);
    let position = s.anchor_position(&a, true).unwrap();
    let pixel = s.camera.project(position, [1000., 800.]).unwrap();
    assert_eq!(
        measurement::pick_anchor(&s, [pixel[0] + 1., pixel[1]], [1000., 800.], 12.),
        Some(a)
    );
    assert!(measurement::pick_anchor(&s, [-100., -100.], [1000., 800.], 12.).is_none());
}
