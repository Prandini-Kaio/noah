use noah_workspace::{demo, disassembly::Mode, session};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p =
            std::env::temp_dir().join(format!("noah-disassembly-{:032x}", noah_storage::new_id()));
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
fn nested_frontier_preserves_subassemblies_deeper_levels_reveal_parts_and_restore() {
    let t = Temp::new();
    let mut s = demo::create_armour(&t.0.join("armour")).unwrap();
    assert_eq!(s.items.len(), 13);
    let raw: Vec<_> = s
        .items
        .iter()
        .map(|i| i.geometry.positions().to_vec())
        .collect();
    let assembled = s.world_transforms(false).unwrap();
    s.view.disassembly.depth = 1;
    s.explosion = 1.;
    let outer = s.world_transforms(true).unwrap();
    let shell = s
        .assemblies
        .iter()
        .find(|g| g.name.contains("Carcaça"))
        .unwrap()
        .id
        .clone();
    let shell_indices: Vec<_> = (0..s.items.len())
        .filter(|&i| s.belongs_to(i, &shell))
        .collect();
    assert!(shell_indices.len() > 5);
    let offset = outer[shell_indices[0]].translation();
    assert!(offset.iter().any(|v| v.abs() > 0.1));
    for &i in &shell_indices {
        assert_eq!(outer[i].translation(), offset);
    }
    assert_eq!(outer[0].translation(), assembled[0].translation());
    s.view.disassembly.depth = 4;
    let deeper = s.world_transforms(true).unwrap();
    assert!(
        shell_indices
            .windows(2)
            .any(|w| deeper[w[0]].translation() != deeper[w[1]].translation())
    );
    assert_eq!(s.world_transforms(false).unwrap(), assembled);
    for (i, item) in s.items.iter().enumerate() {
        assert_eq!(item.geometry.positions(), raw[i]);
        let p = item.geometry.positions();
        let a = deeper[i].apply(p[0]).unwrap();
        let b = deeper[i].apply(p[1]).unwrap();
        let before = (0..3).map(|k| (p[0][k] - p[1][k]).powi(2)).sum::<f64>();
        let after = (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f64>();
        assert!((before - after).abs() < 1e-10);
    }
    s.explosion = 0.;
    assert_eq!(s.world_transforms(true).unwrap(), assembled);
}
#[test]
fn layer_scope_hidden_parts_options_and_session8_roundtrip() {
    let t = Temp::new();
    let mut s = demo::create_armour(&t.0.join("armour")).unwrap();
    let shell = s
        .assemblies
        .iter()
        .find(|g| g.name.contains("Peitoral"))
        .unwrap()
        .id
        .clone();
    s.selected = Some(shell.clone());
    s.view.disassembly.selected_only = true;
    s.view.disassembly.mode = Mode::Layers;
    s.view.disassembly.depth = 1;
    s.explosion = 1.;
    let assembled = s.world_transforms(false).unwrap();
    let exploded = s.world_transforms(true).unwrap();
    assert!(exploded.iter().zip(&assembled).any(|(a, b)| a != b));
    for i in 0..s.items.len() {
        if !s.belongs_to(i, &shell) {
            assert_eq!(exploded[i], assembled[i]);
        }
    }
    let path = t.0.join("session.json");
    session::save(&s, &path).unwrap();
    let loaded = session::load(&path).unwrap();
    assert_eq!(loaded.view.disassembly, s.view.disassembly);
    assert_eq!(loaded.world_transforms(true).unwrap(), exploded);
    let mut v: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["schema_version"], 8);
    v["schema_version"] = 7.into();
    v["view"].as_object_mut().unwrap().remove("disassembly");
    let legacy = t.0.join("legacy.json");
    fs::write(&legacy, serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(
        session::load(&legacy).unwrap().view.disassembly.mode,
        Mode::Radial
    );
    s.view.disassembly.depth = 0;
    assert!(s.view.validate().is_err());
    s.view.disassembly.depth = 4;
    s.view.disassembly.axis = 3;
    assert!(s.world_transforms(true).is_err());
}
