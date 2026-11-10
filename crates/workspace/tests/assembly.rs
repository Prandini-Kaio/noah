use noah_workspace::{Placement, Scene, demo, export_item, session};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-assembly-{:032x}", noah_storage::new_id()));
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
fn composed_transforms_explosion_reversible_and_raw_preserved() {
    let t = Temp::new();
    let mut scene = demo::create(&t.0.join("demo")).unwrap();
    assert_eq!(scene.items.len(), 10);
    assert_eq!(scene.assemblies.len(), 3);
    let source = scene.sources[0].path.clone();
    let raw = fs::read(&source).unwrap();
    let base = scene.world_transforms(false).unwrap();
    let placements: Vec<_> = scene.items.iter().map(|i| i.placement.clone()).collect();
    scene.explosion = 1.;
    let exploded = scene.world_transforms(true).unwrap();
    assert_ne!(base, exploded);
    scene.explosion = 0.;
    assert_eq!(scene.world_transforms(true).unwrap(), base);
    assert_eq!(
        scene
            .items
            .iter()
            .map(|i| i.placement.clone())
            .collect::<Vec<_>>(),
        placements
    );
    assert_eq!(fs::read(source).unwrap(), raw);
    let group = scene.assemblies[0].id.clone();
    scene
        .set_group_placement(
            &group,
            Placement {
                translation: [4., 5., 6.],
                ..Default::default()
            },
        )
        .unwrap();
    let moved = scene.world_transforms(false).unwrap();
    for (before, after) in base.iter().zip(&moved) {
        let a = before.apply([0.; 3]).unwrap();
        let b = after.apply([0.; 3]).unwrap();
        for axis in 0..3 {
            assert!((b[axis] - a[axis] - [4., 5., 6.][axis]).abs() < 1e-12);
        }
    }
}
#[test]
fn cycles_missing_parents_and_invalid_group_updates_roll_back() {
    let t = Temp::new();
    let mut scene = demo::create(&t.0.join("demo")).unwrap();
    let root = scene.assemblies[0].id.clone();
    let child = scene.assemblies[1].id.clone();
    assert!(scene.set_parent(&root, Some(child)).is_err());
    assert!(scene.assemblies[0].parent.is_none());
    let item = scene.items[0].id.clone();
    let old = scene.items[0].parent.clone();
    assert!(scene.set_parent(&item, Some("missing".into())).is_err());
    assert_eq!(scene.items[0].parent, old);
    let old = scene.assemblies[0].placement.clone();
    assert!(
        scene
            .set_group_placement(
                &root,
                Placement {
                    scale: 0.,
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(scene.assemblies[0].placement, old);
    scene.validate_world().unwrap();
}
#[test]
fn schema_two_roundtrip_and_schema_one_compatibility() {
    let t = Temp::new();
    let mut scene = demo::create(&t.0.join("demo")).unwrap();
    scene.explosion = 0.7;
    scene.explosion_distance = 3.;
    scene.items[0].explode_direction = [0., 0., 1.];
    scene.items[0].name = "Base renomeada".into();
    let path = t.0.join("workspace.json");
    session::save(&scene, &path).unwrap();
    let restored = session::load(&path).unwrap();
    assert_eq!(restored.assemblies, scene.assemblies);
    assert_eq!(restored.items[0].parent, scene.items[0].parent);
    assert_eq!(restored.items[0].name, "Base renomeada");
    assert_eq!(
        restored.world_transforms(true).unwrap(),
        scene.world_transforms(true).unwrap()
    );
    let mut legacy = Scene::default();
    legacy.add_source(&scene.sources[0].path).unwrap();
    let old = t.0.join("legacy.json");
    session::save(&legacy, &old).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&old).unwrap()).unwrap();
    value["schema_version"] = 1.into();
    for key in ["assemblies", "explosion", "explosion_distance"] {
        value.as_object_mut().unwrap().remove(key);
    }
    for entity in value["entities"].as_array_mut().unwrap() {
        for key in ["parent", "explode_direction", "name"] {
            entity.as_object_mut().unwrap().remove(key);
        }
    }
    fs::write(&old, serde_json::to_vec(&value).unwrap()).unwrap();
    let opened = session::load(&old).unwrap();
    assert!(opened.assemblies.is_empty());
    assert_eq!(opened.explosion, 0.);
}
#[test]
fn export_ignores_visual_explosion_and_records_units() {
    let t = Temp::new();
    let mut scene = demo::create(&t.0.join("demo")).unwrap();
    let first = t.0.join("assembled.obj");
    export_item(&scene, 0, &first).unwrap();
    scene.explosion = 1.;
    let second = t.0.join("exploded-view.obj");
    export_item(&scene, 0, &second).unwrap();
    assert_eq!(fs::read(first).unwrap(), fs::read(&second).unwrap());
    assert!(export_item(&scene, 0, &second).is_err());
    let info: serde_json::Value =
        serde_json::from_slice(&fs::read(t.0.join("exploded-view.obj.metadata.json")).unwrap())
            .unwrap();
    assert_eq!(info["explosion_applied"], false);
    assert_eq!(info["unit"], "Unknown");
}
