use noah_workspace::{Placement, Scene, measurement::Anchor, session};
fn scene() -> Scene {
    let mut s = Scene::default();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/alinhamento");
    s.add_source(&root.join("fixa.obj")).unwrap();
    s.add_source(&root.join("movel.obj")).unwrap();
    s
}
fn a(s: &Scene, item: usize, vertex: usize) -> Anchor {
    Anchor {
        item_id: s.items[item].id.clone(),
        vertex,
    }
}
#[test]
fn known_translation_preview_apply_and_undo_preserve_sources() {
    let mut s = scene();
    let raw = s.items[1].geometry.positions().to_vec();
    let fixed = s.items[0].placement.clone();
    let source = a(&s, 1, 0);
    let target = a(&s, 0, 1);
    let plan = s.plan_alignment(source.clone(), target.clone()).unwrap();
    assert_eq!(plan.delta(), [-4., 0., 0.]);
    assert_eq!(s.items[1].placement, Placement::default());
    let undo = s.apply_alignment(&plan).unwrap();
    assert!(s.raw_distance(&source, &target).unwrap() < 1e-12);
    assert_eq!(s.items[0].placement, fixed);
    assert_eq!(raw, s.items[1].geometry.positions());
    assert!(!plan.is_current(&s));
    s.undo_alignment(&undo).unwrap();
    assert_eq!(s.items[1].placement, Placement::default());
    assert_eq!(raw, s.items[1].geometry.positions());
}
#[test]
fn parent_rotation_scale_and_base_are_respected() {
    let mut s = scene();
    let root = s.add_assembly("Root".into()).unwrap();
    let inner = s.add_assembly("Inner".into()).unwrap();
    s.set_parent(&inner, Some(root.clone())).unwrap();
    let id = s.items[1].id.clone();
    s.set_parent(&id, Some(inner.clone())).unwrap();
    s.set_group_placement(
        &root,
        Placement {
            translation: [2., 3., 4.],
            angles: [0.3, -0.8, 0.2],
            scale: 2.,
        },
    )
    .unwrap();
    s.set_group_placement(
        &inner,
        Placement {
            angles: [-0.2, 0.4, 0.9],
            scale: 0.5,
            ..Default::default()
        },
    )
    .unwrap();
    s.items[1].base =
        noah_geometry::Transform::from_euler([1., -2., 4.], [0.4, 0.1, 0.2], 1.3).unwrap();
    s.set_placement(
        1,
        Placement {
            angles: [0.1, 0.2, 0.3],
            scale: 1.2,
            ..Default::default()
        },
    )
    .unwrap();
    let before = s.items[1].placement.clone();
    let source = a(&s, 1, 2);
    let target = a(&s, 0, 4);
    let p = s.plan_alignment(source.clone(), target.clone()).unwrap();
    let u = s.apply_alignment(&p).unwrap();
    assert!(s.raw_distance(&source, &target).unwrap() < 1e-10);
    assert_eq!(s.items[1].placement.angles, before.angles);
    assert_eq!(s.items[1].placement.scale, before.scale);
    s.undo_alignment(&u).unwrap();
    assert_eq!(s.items[1].placement, before);
}
#[test]
fn changed_context_and_bad_anchors_do_not_apply_or_undo() {
    let mut s = scene();
    assert!(s.plan_alignment(a(&s, 0, 0), a(&s, 0, 1)).is_err());
    assert!(s.plan_alignment(a(&s, 1, 99), a(&s, 0, 0)).is_err());
    let p = s.plan_alignment(a(&s, 1, 0), a(&s, 0, 1)).unwrap();
    s.set_placement(
        0,
        Placement {
            translation: [1., 0., 0.],
            ..Default::default()
        },
    )
    .unwrap();
    let before = s.items[1].placement.clone();
    assert!(s.apply_alignment(&p).is_err());
    assert_eq!(s.items[1].placement, before);
    let p = s.plan_alignment(a(&s, 1, 0), a(&s, 0, 1)).unwrap();
    let undo = s.apply_alignment(&p).unwrap();
    s.set_placement(0, Placement::default()).unwrap();
    let before = s.items[1].placement.clone();
    assert!(s.undo_alignment(&undo).is_err());
    assert_eq!(s.items[1].placement, before);
}
#[test]
fn applied_alignment_persists_and_explosion_is_ignored() {
    let mut s = scene();
    s.explosion = 1.;
    s.explosion_distance = 100.;
    let source = a(&s, 1, 0);
    let target = a(&s, 0, 1);
    let p = s.plan_alignment(source.clone(), target.clone()).unwrap();
    assert_eq!(p.delta(), [-4., 0., 0.]);
    s.apply_alignment(&p).unwrap();
    let file =
        std::env::temp_dir().join(format!("noah-align-{:032x}.json", noah_storage::new_id()));
    session::save(&s, &file).unwrap();
    let restored = session::load(&file).unwrap();
    std::fs::remove_file(file).unwrap();
    assert!(restored.raw_distance(&source, &target).unwrap() < 1e-12);
    assert_eq!(restored.items[1].placement.translation, [-4., 0., 0.]);
}
