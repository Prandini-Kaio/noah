use noah_workspace::{
    Placement, Scene,
    measurement::{self, Anchor, LengthUnit},
    session,
};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p =
            std::env::temp_dir().join(format!("noah-inspection-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn scene(&self) -> Scene {
        let p = self.0.join("reference.obj");
        fs::write(
            &p,
            "v 1 0 0\nv 0 0 0\nv 0 1 0\nv -1 0 0\nv 2 0 0\nv 1 1 0\nv 0 0 0\nf 1 2 3\n",
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
fn a(s: &Scene, vertex: usize) -> Anchor {
    Anchor {
        item_id: s.items[0].id.clone(),
        vertex,
    }
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
#[test]
fn known_angles_extremes_and_degenerate_arms() {
    let t = Temp::new();
    let s = t.scene();
    near(
        s.angle_degrees(&a(&s, 0), &a(&s, 1), &a(&s, 2)).unwrap(),
        90.,
    );
    near(
        s.angle_degrees(&a(&s, 0), &a(&s, 1), &a(&s, 5)).unwrap(),
        45.,
    );
    near(
        s.angle_degrees(&a(&s, 0), &a(&s, 1), &a(&s, 3)).unwrap(),
        180.,
    );
    near(
        s.angle_degrees(&a(&s, 0), &a(&s, 1), &a(&s, 4)).unwrap(),
        0.,
    );
    assert!(s.angle_degrees(&a(&s, 6), &a(&s, 1), &a(&s, 2)).is_err());
    assert!(s.angle_degrees(&a(&s, 0), &a(&s, 0), &a(&s, 2)).is_err());
    assert!(s.angle_degrees(&a(&s, 99), &a(&s, 1), &a(&s, 2)).is_err());
}
#[test]
fn angles_ignore_explosion_rigid_motion_uniform_scale_and_stale_calibration() {
    let t = Temp::new();
    let mut s = t.scene();
    let (a, b, c) = (a(&s, 0), a(&s, 1), a(&s, 2));
    s.calibrate(
        a.clone(),
        b.clone(),
        100.,
        LengthUnit::Millimetre,
        "Referência".into(),
    )
    .unwrap();
    s.set_placement(
        0,
        Placement {
            translation: [7., -3., 12.],
            angles: [0.4, 0.8, -0.7],
            scale: 2.,
        },
    )
    .unwrap();
    s.explosion = 1.;
    s.explosion_distance = 100.;
    assert!(s.scale_factor().is_err());
    near(s.angle_degrees(&a, &b, &c).unwrap(), 90.);
}
#[test]
fn bounds_are_exact_world_xyz_with_hidden_descendants_and_calibration() {
    let t = Temp::new();
    let mut s = t.scene();
    let g = s.add_assembly("Conjunto".into()).unwrap();
    let subgroup = s.add_assembly("Subgrupo".into()).unwrap();
    s.set_parent(&subgroup, Some(g.clone())).unwrap();
    let id = s.items[0].id.clone();
    s.set_parent(&id, Some(subgroup)).unwrap();
    let mut other = s.items[0].clone();
    other.id = "other".into();
    other.visible = false;
    other.placement.translation = [10., 0., 0.];
    s.items.push(other);
    let d = s.dimensions(&g).unwrap();
    assert_eq!(d.bounds.min, [-1., 0., 0.]);
    assert_eq!(d.bounds.max, [12., 1., 0.]);
    assert_eq!(d.size, [13., 1., 0.]);
    assert_eq!(d.bounds.item_count, 2);
    assert_eq!(d.bounds.vertex_count, 14);
    assert_eq!(d.unit, None);
    s.calibrate(
        a(&s, 0),
        a(&s, 1),
        100.,
        LengthUnit::Millimetre,
        "Referência".into(),
    )
    .unwrap();
    assert_eq!(s.dimensions(&g).unwrap().size, [1300., 100., 0.]);
    s.explosion = 1.;
    s.explosion_distance = 1000.;
    assert_eq!(s.dimensions(&g).unwrap().size, [1300., 100., 0.]);
    s.set_placement(
        0,
        Placement {
            scale: 2.,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(s.dimensions(&g).is_err());
    assert!(s.raw_bounds(&g).is_ok());
}
#[test]
fn rotation_changes_aabb_and_empty_targets_fail() {
    let t = Temp::new();
    let mut s = t.scene();
    let id = s.items[0].id.clone();
    s.set_placement(
        0,
        Placement {
            angles: [0., 0., std::f64::consts::FRAC_PI_2],
            translation: [5., 6., 7.],
            ..Default::default()
        },
    )
    .unwrap();
    let d = s.dimensions(&id).unwrap();
    near(d.size[0], 1.);
    near(d.size[1], 3.);
    near(d.size[2], 0.);
    let g = s.add_assembly("Vazio".into()).unwrap();
    assert!(s.dimensions(&g).is_err());
    assert!(s.dimensions("missing").is_err());
    assert!(s.add_dimensions(g, "Vazio".into()).is_err());
}
#[test]
fn session5_report2_and_legacy_keep_identity_methods_and_transforms() {
    let t = Temp::new();
    let mut s = t.scene();
    let id = s.items[0].id.clone();
    s.add_angle(a(&s, 0), a(&s, 1), a(&s, 2), "Reto".into())
        .unwrap();
    s.add_dimensions(id, "Caixa".into()).unwrap();
    let p = t.0.join("session.json");
    session::save(&s, &p).unwrap();
    let restored = session::load(&p).unwrap();
    assert_eq!(s.angles, restored.angles);
    assert_eq!(s.dimension_measurements, restored.dimension_measurements);
    let report = t.0.join("report.json");
    measurement::export_report(&s, &report).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
    assert_eq!(json["schema_version"], 4);
    assert_eq!(json["angles"][0]["value"], 90.);
    assert_eq!(json["angles"][0]["unit"], "deg");
    assert_eq!(
        json["dimensions"][0]["size"],
        serde_json::json!([3., 1., 0.])
    );
    assert_eq!(json["dimensions"][0]["unit"], "scene_units");
    assert_eq!(json["instances"][0]["world_transform"]["scale"], 1.);
    assert!(json["angles"][0]["uncertainty"].is_null());
    assert!(measurement::export_report(&s, &report).is_err());
    let mut saved: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    assert_eq!(saved["schema_version"], 7);
    saved.as_object_mut().unwrap().remove("angles");
    saved
        .as_object_mut()
        .unwrap()
        .remove("dimension_measurements");
    for version in 1..=4 {
        saved["schema_version"] = version.into();
        fs::write(&p, serde_json::to_vec(&saved).unwrap()).unwrap();
        let old = session::load(&p).unwrap();
        assert!(old.angles.is_empty() && old.dimension_measurements.is_empty());
    }
}
#[test]
fn malformed_inspections_are_rejected_and_do_not_write_session() {
    let t = Temp::new();
    let mut s = t.scene();
    assert!(
        s.add_angle(a(&s, 0), a(&s, 1), a(&s, 2), " ".into())
            .is_err()
    );
    s.add_angle(a(&s, 0), a(&s, 1), a(&s, 2), "A".into())
        .unwrap();
    s.add_dimensions(s.items[0].id.clone(), "B".into()).unwrap();
    let valid = t.0.join("valid.json");
    session::save(&s, &valid).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&valid).unwrap()).unwrap();
    for (field, key, val) in [
        ("angles", "method", serde_json::json!("fake")),
        ("angles", "uncertainty", serde_json::json!(0.01)),
        (
            "dimension_measurements",
            "target_id",
            serde_json::json!("missing"),
        ),
    ] {
        let mut v = original.clone();
        v[field][0][key] = val;
        let p = t.0.join(format!("{field}-{key}.json"));
        fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(session::load(&p).is_err());
    }
    s.dimension_measurements[0].id = s.angles[0].id.clone();
    assert!(s.validate_measurements().is_err());
    let p = t.0.join("invalid.json");
    assert!(session::save(&s, &p).is_err());
    assert!(!p.exists());
}
