#![cfg(unix)]
use noah_workspace::{Geometry, Scene, SourceKind, measurement::Anchor, session};
use std::{fs, os::unix::fs::PermissionsExt, sync::Arc, time::Duration};
#[test]
fn surface_and_legacy_run_representations_roundtrip_without_retargeting_anchors() {
    let temp =
        std::env::temp_dir().join(format!("noah-run-surface-{:032x}", noah_storage::new_id()));
    fs::create_dir(&temp).unwrap();
    let images = temp.join("images");
    fs::create_dir(&images).unwrap();
    for i in 0..3 {
        fs::write(images.join(format!("{i}.jpg")), b"\xff\xd8\xfftest").unwrap();
    }
    let engine = temp.join("motor.py");
    fs::write(
        &engine,
        "#!/usr/bin/env python3\nprint('controlled stage')\n",
    )
    .unwrap();
    fs::set_permissions(&engine, fs::Permissions::from_mode(0o755)).unwrap();
    let root = temp.join("run");
    let mut job = noah_pipeline::Job::create(
        &root,
        &images,
        &engine,
        Default::default(),
        Duration::from_secs(3),
    )
    .unwrap();
    job.step("fixture", vec![]).unwrap();
    fs::create_dir(root.join("exports")).unwrap();
    let cloud = noah_geometry::PointCloud::new(vec![[9., 0., 0.], [12., 0., 0.]], None).unwrap();
    fs::write(
        root.join("exports/sparse.ply"),
        noah_storage::ply::encode(&cloud),
    )
    .unwrap();
    fs::write(
        root.join("exports/mesh.obj"),
        "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n",
    )
    .unwrap();
    fs::write(
        root.join("exports/mesh.metadata.json"),
        "{\"textured\":false}",
    )
    .unwrap();
    job.artifact("exports/sparse.ply", "sparse_point_cloud")
        .unwrap();
    job.artifact("exports/mesh.obj", "surface_mesh_obj")
        .unwrap();
    job.artifact("exports/mesh.metadata.json", "surface_provenance")
        .unwrap();
    job.succeed().unwrap();
    let uuid = job.manifest().run_id.clone();
    let mut s = Scene::default();
    s.add_source(&root).unwrap();
    assert_eq!(s.sources[0].kind, SourceKind::RunSurface);
    assert!(matches!(s.items[0].geometry.as_ref(), Geometry::Mesh(_)));
    let id = s.items[0].id.clone();
    s.add_measurement(
        Anchor {
            item_id: id.clone(),
            vertex: 0,
        },
        Anchor {
            item_id: id,
            vertex: 1,
        },
        "Surface AB".into(),
    )
    .unwrap();
    let file = temp.join("surface.json");
    session::save(&s, &file).unwrap();
    let r = session::load(&file).unwrap();
    assert_eq!(r.sources, s.sources);
    assert_eq!(r.measurements, s.measurements);
    assert_eq!(
        r.raw_distance(&r.measurements[0].a, &r.measurements[0].b)
            .unwrap(),
        1.
    );
    let mut legacy = s.clone();
    legacy.sources[0].kind = SourceKind::Run;
    legacy.items[0].geometry = Arc::new(Geometry::Points(cloud));
    legacy.items[0].id = format!("run:{uuid}");
    legacy.measurements.clear();
    let id = legacy.items[0].id.clone();
    legacy
        .add_measurement(
            Anchor {
                item_id: id.clone(),
                vertex: 0,
            },
            Anchor {
                item_id: id,
                vertex: 1,
            },
            "Legacy AB".into(),
        )
        .unwrap();
    let old = temp.join("legacy.json");
    session::save(&legacy, &old).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&fs::read(&old).unwrap()).unwrap();
    json["schema_version"] = 5.into();
    fs::write(&old, serde_json::to_vec(&json).unwrap()).unwrap();
    let r = session::load(&old).unwrap();
    assert!(matches!(r.items[0].geometry.as_ref(), Geometry::Points(_)));
    assert_eq!(
        r.raw_distance(&r.measurements[0].a, &r.measurements[0].b)
            .unwrap(),
        3.
    );
    let migrated = temp.join("migrated.json");
    session::save(&r, &migrated).unwrap();
    let r = session::load(&migrated).unwrap();
    assert_eq!(r.sources[0].kind, SourceKind::Run);
    assert_eq!(
        r.raw_distance(&r.measurements[0].a, &r.measurements[0].b)
            .unwrap(),
        3.
    );
    fs::remove_dir_all(temp).unwrap();
}
