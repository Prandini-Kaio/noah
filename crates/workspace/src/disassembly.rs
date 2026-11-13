//! Hierarchical rigid-part presentation. World XYZ, translations only; no asset edits.
use crate::{Result, Scene};
use noah_geometry::Transform;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Radial,
    Layers,
    Armour,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub mode: Mode,
    pub depth: usize,
    pub axis: usize,
    pub selected_only: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Radial,
            depth: 4,
            axis: 2,
            selected_only: false,
        }
    }
}
impl Options {
    pub fn validate(&self) -> Result<()> {
        if !(1..=8).contains(&self.depth) || self.axis > 2 {
            return Err("desmontagem exige profundidade 1–8 e eixo XYZ".into());
        }
        Ok(())
    }
}
#[derive(Clone)]
struct Bounds {
    min: [f64; 3],
    max: [f64; 3],
}
impl Bounds {
    fn empty() -> Self {
        Self {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
        }
    }
    fn point(&mut self, p: [f64; 3]) {
        for (a, &v) in p.iter().enumerate() {
            self.min[a] = self.min[a].min(v);
            self.max[a] = self.max[a].max(v);
        }
    }
    fn center(&self) -> [f64; 3] {
        std::array::from_fn(|a| (self.min[a] + self.max[a]) * 0.5)
    }
    fn contains(&self, other: &Self) -> bool {
        (0..3).all(|a| self.min[a] <= other.min[a] && self.max[a] >= other.max[a])
    }
    fn span(&self) -> f64 {
        (0..3)
            .map(|a| (self.max[a] - self.min[a]).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
/// Shared offsets for a frontier preserve child subassemblies until deeper levels open.
pub fn offsets(scene: &Scene, transforms: &[Transform]) -> Result<Vec<[f64; 3]>> {
    let options = &scene.view.disassembly;
    options.validate()?;
    if transforms.len() != scene.items.len() {
        return Err("transformações incompatíveis".into());
    }
    let mut paths = Vec::new();
    let mut bounds = Vec::new();
    let mut included = Vec::new();
    for (i, item) in scene.items.iter().enumerate() {
        let mut path = vec![item.id.clone()];
        let mut parent = item.parent.as_deref();
        for _ in 0..64 {
            let Some(id) = parent else { break };
            path.push(id.into());
            parent = scene
                .assemblies
                .iter()
                .find(|g| g.id == id)
                .and_then(|g| g.parent.as_deref());
        }
        path.reverse();
        paths.push(path);
        let mut b = Bounds::empty();
        for &p in item.geometry.positions() {
            b.point(transforms[i].apply(p)?);
        }
        if item.geometry.positions().is_empty() {
            b.point(transforms[i].translation());
        }
        bounds.push(b);
        included.push(
            item.visible
                && (!options.selected_only
                    || scene.selected.as_ref() == Some(&item.id)
                    || scene
                        .selected
                        .as_ref()
                        .is_some_and(|id| scene.belongs_to(i, id))),
        );
    }
    while let Some(first) = paths
        .iter()
        .enumerate()
        .find(|(i, _)| included[*i])
        .and_then(|(_, p)| p.first())
        .cloned()
    {
        if paths
            .iter()
            .enumerate()
            .filter(|(i, _)| included[*i])
            .all(|(_, p)| p.len() > 1 && p.first() == Some(&first))
        {
            for (i, path) in paths.iter_mut().enumerate() {
                if included[i] {
                    path.remove(0);
                }
            }
        } else {
            break;
        }
    }
    let mut levels = Vec::new();
    for depth in 0..options.depth {
        let mut parents: BTreeMap<Vec<String>, BTreeMap<String, Vec<usize>>> = BTreeMap::new();
        for (i, path) in paths.iter().enumerate() {
            if included[i] && path.len() > depth {
                parents
                    .entry(path[..depth].to_vec())
                    .or_default()
                    .entry(path[depth].clone())
                    .or_default()
                    .push(i);
            }
        }
        let partitions: Vec<_> = parents
            .into_values()
            .filter(|children| children.len() > 1)
            .collect();
        if !partitions.is_empty() {
            levels.push(partitions);
        }
    }
    let mut offsets = vec![[0.; 3]; scene.items.len()];
    let count = levels.len().max(1);
    for (level, partitions) in levels.into_iter().enumerate() {
        let phase = if options.mode == Mode::Armour {
            let q = (scene.explosion * count as f64 - level as f64).clamp(0., 1.);
            q * q * (3. - 2. * q)
        } else {
            scene.explosion
        };
        for children in partitions {
            let mut groups: Vec<_> = children
                .into_values()
                .map(|indices| {
                    let mut b = Bounds::empty();
                    for &i in &indices {
                        b.point(bounds[i].min);
                        b.point(bounds[i].max);
                    }
                    (indices, b)
                })
                .collect();
            groups.sort_by(|a, b| {
                a.1.center()[options.axis]
                    .total_cmp(&b.1.center()[options.axis])
                    .then(a.0[0].cmp(&b.0[0]))
            });
            let mut parent = Bounds::empty();
            for (_, b) in &groups {
                parent.point(b.min);
                parent.point(b.max);
            }
            let center = parent.center();
            let inner = if options.mode == Mode::Armour {
                groups
                    .iter()
                    .enumerate()
                    .find(|(i, (_, b))| {
                        groups.iter().enumerate().all(|(j, (_, other))| {
                            i == &j || other.contains(b) && other.span() > b.span() * 1.1
                        })
                    })
                    .map(|(i, _)| i)
            } else {
                None
            };
            for (rank, (indices, b)) in groups.iter().enumerate() {
                let mut direction = [0.; 3];
                if options.mode == Mode::Layers {
                    direction[options.axis] = (rank as f64 - (groups.len() - 1) as f64 * 0.5) * 1.5;
                } else if inner != Some(rank) {
                    direction = std::array::from_fn(|a| b.center()[a] - center[a]);
                    let norm = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if norm < 1e-12 {
                        direction = [0.; 3];
                        direction[(options.axis + rank) % 3] = if rank % 2 == 0 { 1. } else { -1. };
                    } else {
                        direction = direction.map(|v| v / norm);
                    }
                }
                for &i in indices {
                    for (a, &d) in direction.iter().enumerate() {
                        offsets[i][a] += d * scene.explosion_distance * phase;
                    }
                }
            }
        }
    }
    Ok(offsets)
}
