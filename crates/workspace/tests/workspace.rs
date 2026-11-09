use noah_geometry::UnitState;
use noah_storage::StoredProject;
use noah_workspace::{Camera, Placement, Scene, pick, project, session};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-scene-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn ply(&self) -> PathBuf {
        let path = self.0.join("points.ply");
        fs::write(
            &path,
            include_bytes!("../../../tests/datasets/reference-rgb.ply"),
        )
        .unwrap();
        path
    }
    fn obj(&self) -> PathBuf {
        let path = self.0.join("mesh.obj");
        fs::write(
            &path,
            include_bytes!("../../../tests/datasets/reference-mesh.obj"),
        )
        .unwrap();
        path
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn camera_orbit_pan_zoom_and_projection() {
    let mut c = Camera::default();
    let p = c.project(c.target, [800., 600.]).unwrap();
    assert!((p[0] - 400.).abs() < 1e-10 && (p[1] - 300.).abs() < 1e-10);
    let eye = c.basis().0;
    assert!(c.project(eye, [800., 600.]).is_none());
    let distance = c.distance;
    c.orbit([80., 40.]);
    c.zoom(100.);
    assert!(c.distance < distance);
    let target = c.target;
    c.pan([50., 0.], 600.);
    assert_ne!(target, c.target);
    c.validate().unwrap();
    c.pitch = f64::NAN;
    assert!(c.validate().is_err());
}
#[test]
fn select_visible_geometry_and_apply_instance_without_mutating_source() {
    let t = Temp::new();
    let input = t.ply();
    let bytes = fs::read(&input).unwrap();
    let mut scene = Scene::default();
    scene.add_source(&input).unwrap();
    let primitives = project(&scene, [1000., 800.]);
    assert!(!primitives.is_empty());
    assert_eq!(pick(&primitives, primitives[0].pixels[0], 7.), Some(0));
    let point = scene.items[0].geometry.positions()[0];
    scene
        .set_placement(
            0,
            Placement {
                translation: [10., 0., 0.],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(scene.items[0].geometry.positions()[0], point);
    assert_eq!(fs::read(&input).unwrap(), bytes);
    let old = scene.items[0].placement.clone();
    assert!(
        scene
            .set_placement(
                0,
                Placement {
                    scale: 0.,
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(scene.items[0].placement, old);
    scene.items[0].visible = false;
    assert!(project(&scene, [1000., 800.]).is_empty());
}
#[test]
fn session_roundtrip_restores_sources_camera_selection_and_transforms() {
    let t = Temp::new();
    let input = t.ply();
    let mesh = t.obj();
    let original = fs::read(&mesh).unwrap();
    let mut scene = Scene::default();
    scene.add_source(&input).unwrap();
    scene.add_source(&mesh).unwrap();
    scene
        .set_placement(
            1,
            Placement {
                translation: [3., 4., 5.],
                angles: [0.1, 0.2, 0.3],
                scale: 2.,
            },
        )
        .unwrap();
    scene.items[0].visible = false;
    scene.selected = Some(scene.items[1].id.clone());
    scene.camera.orbit([35., 20.]);
    scene.camera.zoom(20.);
    scene.point_size = 4.;
    scene.grid = false;
    scene.wireframe = true;
    let path = t.0.join("session.json");
    session::save(&scene, &path).unwrap();
    let restored = session::load(&path).unwrap();
    assert_eq!(restored.sources, scene.sources);
    assert_eq!(restored.camera, scene.camera);
    assert_eq!(restored.selected, scene.selected);
    assert_eq!(restored.items[1].placement, scene.items[1].placement);
    assert!(!restored.items[0].visible);
    assert!(restored.wireframe);
    assert!(!restored.grid);
    assert_eq!(restored.point_size, 4.);
    assert_eq!(fs::read(&mesh).unwrap(), original);
    assert!(session::save(&scene, &path).is_err());
    fs::write(&input, b"changed").unwrap();
    assert!(session::load(&path).is_err());
}
#[test]
fn project_e1_loads_and_failure_keeps_previous_scene() {
    let t = Temp::new();
    let input = t.ply();
    let root = t.0.join("project");
    let mut stored = StoredProject::create(&root, "Test".into()).unwrap();
    stored
        .import(&input, "Object".into(), UnitState::Millimetre)
        .unwrap();
    let mut scene = Scene::default();
    scene.add_source(&root).unwrap();
    assert!(scene.items[0].unit.contains("Millimetre"));
    let count = scene.items.len();
    let camera = scene.camera.clone();
    assert!(scene.add_source(&t.0.join("missing.ply")).is_err());
    assert_eq!(scene.items.len(), count);
    assert_eq!(scene.camera, camera);
    let path = t.0.join("session.json");
    session::save(&scene, &path).unwrap();
    assert!(session::load(&path).is_ok());
    let raw = fs::read_dir(root.join("raw"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(raw, b"corrupt").unwrap();
    assert!(session::load(&path).is_err());
}
#[test]
fn mesh_selection_and_session_schema_validation() {
    let t = Temp::new();
    let mut scene = Scene::default();
    scene.add_source(&t.obj()).unwrap();
    let primitives = project(&scene, [1000., 800.]);
    assert!(!primitives.is_empty());
    let triangle = primitives
        .iter()
        .rev()
        .find(|p| p.pixels.len() == 3)
        .unwrap();
    let center = [
        triangle.pixels.iter().map(|p| p[0]).sum::<f64>() / 3.,
        triangle.pixels.iter().map(|p| p[1]).sum::<f64>() / 3.,
    ];
    assert_eq!(pick(&primitives, center, 7.), Some(0));
    let path = t.0.join("session.json");
    session::save(&scene, &path).unwrap();
    let original = fs::read(&path).unwrap();
    for mode in ["schema", "camera", "selection", "duplicate"] {
        let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        match mode {
            "schema" => value["schema_version"] = 99.into(),
            "camera" => value["camera"]["distance"] = (-1).into(),
            "selection" => value["selected"] = "missing".into(),
            _ => {
                let entity = value["entities"][0].clone();
                value["entities"].as_array_mut().unwrap().push(entity);
            }
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(session::load(&path).is_err());
    }
}
