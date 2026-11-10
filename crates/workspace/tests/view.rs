use noah_workspace::{
    Camera, Scene,
    camera::{self, Projection},
    measurement::{self, Anchor, LengthUnit},
    session,
    view::{ThemeMode, nice_step, ruler_scale},
};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-view-{:032x}", noah_storage::new_id()));
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
#[test]
fn orthographic_is_uniform_across_depth_and_matches_ruler() {
    let c = Camera {
        projection: Projection::Orthographic,
        ..Default::default()
    };
    let (_, right, _, forward) = c.basis();
    let p = camera::add(c.target, right);
    let q = camera::add(p, camera::mul(forward, 50.));
    let a = c.project(p, [1000., 800.]).unwrap();
    let b = c.project(q, [1000., 800.]).unwrap();
    assert!((a[0] - b[0]).abs() < 1e-10 && (a[1] - b[1]).abs() < 1e-10);
    let center = c.project(c.target, [1000., 800.]).unwrap();
    assert!(((a[0] - center[0]) * c.units_per_pixel(800.) - 1.).abs() < 1e-12);
    let perspective = Camera {
        projection: Projection::Perspective,
        ..c
    };
    assert!(
        perspective.project(p, [1000., 800.]).unwrap()[0]
            > perspective.project(q, [1000., 800.]).unwrap()[0]
    );
}
#[test]
fn ruler_uses_explicit_scale_and_rejects_stale_physical_units() {
    let t = Temp::new();
    let mut s = t.scene();
    let a = Anchor {
        item_id: s.items[0].id.clone(),
        vertex: 1,
    };
    let b = Anchor {
        item_id: s.items[0].id.clone(),
        vertex: 2,
    };
    let raw = ruler_scale(&s, &s.camera, 800.).unwrap();
    assert_eq!(raw.unit, "u. cena");
    s.calibrate(a, b, 100., LengthUnit::Millimetre, "Teste sintético".into())
        .unwrap();
    let mm = ruler_scale(&s, &s.camera, 800.).unwrap();
    assert_eq!(mm.unit, "mm");
    assert!(mm.calibrated);
    assert!((mm.units_per_pixel / raw.units_per_pixel - 100.).abs() < 1e-10);
    s.items[0].placement.scale = 2.;
    let stale = ruler_scale(&s, &s.camera, 800.).unwrap();
    assert!(stale.stale && !stale.calibrated);
    assert_eq!(stale.unit, "u. cena");
    assert_eq!(nice_step(0.31), 0.5);
    assert_eq!(nice_step(11.), 20.);
    assert!(ruler_scale(&s, &s.camera, 0.).is_err());
}
#[test]
fn arctic_preferences_and_projection_roundtrip_and_legacy_defaults() {
    let t = Temp::new();
    let mut s = t.scene();
    s.view.theme = ThemeMode::ArcticLight;
    s.view.rulers = false;
    s.view.decimals = 7;
    s.view.reduced_motion = true;
    s.camera.projection = Projection::Orthographic;
    let p = t.0.join("view.json");
    session::save(&s, &p).unwrap();
    let reopened = session::load(&p).unwrap();
    assert_eq!(reopened.view, s.view);
    assert_eq!(reopened.camera, s.camera);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
    value["schema_version"] = 3.into();
    value.as_object_mut().unwrap().remove("view");
    value["camera"]
        .as_object_mut()
        .unwrap()
        .remove("projection");
    fs::write(&p, serde_json::to_vec(&value).unwrap()).unwrap();
    let legacy = session::load(&p).unwrap();
    assert_eq!(legacy.view.theme, ThemeMode::ArcticDark);
    assert_eq!(legacy.camera.projection, Projection::Perspective);
    assert!(legacy.view.rulers);
    value["schema_version"] = 4.into();
    value["view"] = serde_json::json!({"decimals":99});
    fs::write(&p, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(session::load(&p).is_err());
}
#[test]
fn snap_filters_occluded_vertices_and_selected_parts() {
    let t = Temp::new();
    let front = t.0.join("front.obj");
    let back = t.0.join("back.obj");
    fs::write(&front, "v 1 -2 -2\nv 1 2 -2\nv 1 0 2\nf 1 2 3\n").unwrap();
    fs::write(&back, "v 0 0 0\nv 0 1 -1\nv 0 -1 -1\nf 1 2 3\n").unwrap();
    let mut s = Scene::default();
    s.add_source(&front).unwrap();
    s.add_source(&back).unwrap();
    s.camera = Camera {
        target: [0.; 3],
        distance: 5.,
        yaw: 0.,
        pitch: 0.,
        projection: Projection::Orthographic,
    };
    assert!(measurement::pick_anchor_controlled(&s, [500., 400.], [1000., 800.]).is_none());
    s.view.occlusion = false;
    let a = measurement::pick_anchor_controlled(&s, [500., 400.], [1000., 800.]).unwrap();
    assert_eq!(a.item_id, s.items[1].id);
    s.view.selected_only = true;
    s.selected = Some(s.items[0].id.clone());
    assert!(measurement::pick_anchor_controlled(&s, [500., 400.], [1000., 800.]).is_none());
    s.selected = Some(s.items[1].id.clone());
    s.items[0].visible = false;
    s.view.occlusion = true;
    assert_eq!(
        measurement::pick_anchor_controlled(&s, [500., 400.], [1000., 800.]),
        Some(a)
    );
}
