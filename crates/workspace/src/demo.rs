//! Demonstração sintética embarcada; materializa fontes para sessão reabrível.
use crate::{Result, Scene};
use std::path::Path;
const PARTS: [(&str, &str); 10] = [
    ("base", include_str!("../../../examples/assembly/base.obj")),
    (
        "suporte-esquerdo",
        include_str!("../../../examples/assembly/suporte-esquerdo.obj"),
    ),
    (
        "suporte-direito",
        include_str!("../../../examples/assembly/suporte-direito.obj"),
    ),
    (
        "mancal-esquerdo",
        include_str!("../../../examples/assembly/mancal-esquerdo.obj"),
    ),
    (
        "mancal-direito",
        include_str!("../../../examples/assembly/mancal-direito.obj"),
    ),
    ("eixo", include_str!("../../../examples/assembly/eixo.obj")),
    (
        "volante",
        include_str!("../../../examples/assembly/volante.obj"),
    ),
    (
        "anel-retentor",
        include_str!("../../../examples/assembly/anel-retentor.obj"),
    ),
    (
        "manivela",
        include_str!("../../../examples/assembly/manivela.obj"),
    ),
    (
        "cubo-manivela",
        include_str!("../../../examples/assembly/cubo-manivela.obj"),
    ),
];
pub fn create(folder: &Path) -> Result<Scene> {
    std::fs::create_dir(folder)?;
    let mut scene = Scene::default();
    for (name, text) in PARTS {
        let file = folder.join(format!("{name}.obj"));
        noah_pipeline::write_new(&file, text.as_bytes())?;
        scene.add_source(&file)?;
    }
    let root = scene.add_assembly("Conjunto mecânico demonstrativo".into())?;
    let support = scene.add_assembly("Base e suportes".into())?;
    scene.set_parent(&support, Some(root.clone()))?;
    let rotating = scene.add_assembly("Eixo e acionamento".into())?;
    scene.set_parent(&rotating, Some(root.clone()))?;
    for i in 0..scene.items.len() {
        let parent = if i < 5 {
            support.clone()
        } else {
            rotating.clone()
        };
        let id = scene.items[i].id.clone();
        scene.set_parent(&id, Some(parent))?;
    }
    scene.explosion_distance = 2.5;
    scene.selected = None;
    scene.fit(false)?;
    Ok(scene)
}
