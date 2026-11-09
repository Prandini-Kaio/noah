use std::{
    fs,
    process::{Command, Output},
};
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_noah"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn create_import_transform_reopen_export_and_errors() {
    let root = std::env::temp_dir().join(format!("noah-cli-{:032x}", noah_storage::new_id()));
    fs::create_dir(&root).unwrap();
    let project = root.join("project");
    let p = project.to_str().unwrap();
    let input = root.join("input.ply");
    fs::write(
        &input,
        include_bytes!("../../../tests/datasets/reference-rgb.ply"),
    )
    .unwrap();
    let i = input.to_str().unwrap();
    assert!(run(&["create", p, "CLI demo"]).status.success());
    assert!(!run(&["import", p, i, "Peça"]).status.success());
    let imported = run(&["import", p, i, "Peça", "--frame", "z-up"]);
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let out = String::from_utf8(imported.stdout).unwrap();
    let id = out.trim().strip_prefix("object_id=").unwrap();
    assert!(
        run(&[
            "transform",
            p,
            id,
            "3",
            "4",
            "5",
            "0",
            "0",
            "1.5707963267948966",
            "2"
        ])
        .status
        .success()
    );
    let inspect = run(&["inspect", p]);
    assert!(inspect.status.success());
    let text = String::from_utf8(inspect.stdout).unwrap();
    assert!(text.contains("unit=Unknown") && text.contains("points=3") && text.contains(id));
    assert!(run(&["validate", p]).status.success());
    let export = root.join("out.ply");
    let e = export.to_str().unwrap();
    assert!(run(&["export", p, id, e]).status.success());
    let cloud = noah_storage::ply::parse(&fs::read(&export).unwrap()).unwrap();
    for (a, b) in cloud.positions()[0].iter().zip([3., 6., 9.]) {
        assert!((a - b).abs() < 1e-12);
    }
    assert!(!run(&["export", p, id, e]).status.success());
    assert!(
        !run(&["transform", p, id, "NaN", "0", "0", "0", "0", "0", "1"])
            .status
            .success()
    );
    assert!(!run(&["unknown"]).status.success());
    fs::remove_dir_all(root).unwrap();
}
