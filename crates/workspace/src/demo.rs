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

/// Synthetic nested armour panels and inner core; no inferred hidden geometry.
pub fn create_armour(folder: &Path) -> Result<Scene> {
    std::fs::create_dir(folder)?;
    let mut scene = Scene::default();
    let root = scene.add_assembly("Armadura · exemplo sintético".into())?;
    let shell = scene.add_assembly("01 · Carcaça exterior".into())?;
    let core = scene.add_assembly("02 · Núcleo interno".into())?;
    scene.set_parent(&shell, Some(root.clone()))?;
    scene.set_parent(&core, Some(root.clone()))?;
    let chest = scene.add_assembly("Peitoral · seis painéis".into())?;
    let helmet = scene.add_assembly("Capacete".into())?;
    let arms = scene.add_assembly("Proteções dos braços".into())?;
    for id in [&chest, &helmet, &arms] {
        scene.set_parent(id, Some(shell.clone()))?;
    }
    let parts = [
        (
            "nucleo-tronco",
            [-0.65, -0.35, 0.6],
            [0.65, 0.35, 2.8],
            &core,
        ),
        ("nucleo-cabeca", [-0.4, -0.3, 3.25], [0.4, 0.3, 3.9], &core),
        ("peitoral-frente", [-1., -0.65, 0.4], [1., -0.5, 3.], &chest),
        ("peitoral-costas", [-1., 0.5, 0.4], [1., 0.65, 3.], &chest),
        (
            "peitoral-esquerda",
            [-1., -0.5, 0.4],
            [-0.85, 0.5, 3.],
            &chest,
        ),
        ("peitoral-direita", [0.85, -0.5, 0.4], [1., 0.5, 3.], &chest),
        (
            "peitoral-topo",
            [-0.85, -0.5, 2.9],
            [0.85, 0.5, 3.05],
            &chest,
        ),
        (
            "peitoral-base",
            [-0.85, -0.5, 0.3],
            [0.85, 0.5, 0.45],
            &chest,
        ),
        ("elmo-frente", [-0.6, -0.55, 3.1], [0.6, -0.4, 4.1], &helmet),
        ("elmo-costas", [-0.6, 0.4, 3.1], [0.6, 0.55, 4.1], &helmet),
        ("elmo-topo", [-0.6, -0.4, 4.0], [0.6, 0.4, 4.15], &helmet),
        (
            "braco-esquerdo",
            [-1.6, -0.45, 1.6],
            [-1.2, 0.45, 2.9],
            &arms,
        ),
        ("braco-direito", [1.2, -0.45, 1.6], [1.6, 0.45, 2.9], &arms),
    ];
    for (name, min, max, parent) in parts {
        let mut obj = String::new();
        for p in [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [max[0], max[1], max[2]],
            [min[0], max[1], max[2]],
        ] {
            obj.push_str(&format!("v {} {} {}\n", p[0], p[1], p[2]));
        }
        obj.push_str("f 1 3 2\nf 1 4 3\nf 5 6 7\nf 5 7 8\nf 1 2 6\nf 1 6 5\nf 2 3 7\nf 2 7 6\nf 3 4 8\nf 3 8 7\nf 4 1 5\nf 4 5 8\n");
        let file = folder.join(format!("{name}.obj"));
        noah_pipeline::write_new(&file, obj.as_bytes())?;
        scene.add_source(&file)?;
        let id = scene.items.last().ok_or("peça ausente")?.id.clone();
        scene.set_parent(&id, Some(parent.clone()))?;
    }
    scene.view.disassembly.mode = crate::disassembly::Mode::Armour;
    scene.view.disassembly.depth = 4;
    scene.explosion_distance = 0.9;
    scene.selected = Some(root);
    scene.fit(false)?;
    Ok(scene)
}
