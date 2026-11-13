//! Montagens de cena e deslocamentos de visualização não destrutivos.
use crate::{Placement, Result, Scene};
use noah_geometry::Transform;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assembly {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
    pub placement: Placement,
}
impl Scene {
    pub fn add_assembly(&mut self, name: String) -> Result<String> {
        if self.assemblies.len() >= 1000 || name.trim().is_empty() {
            return Err("nome vazio ou limite de montagens excedido".into());
        }
        let id = format!("assembly:{:032x}", noah_storage::new_id());
        self.assemblies.push(Assembly {
            id: id.clone(),
            name,
            parent: None,
            placement: Placement::default(),
        });
        Ok(id)
    }
    pub fn validate_hierarchy(&self) -> Result<()> {
        if self.assemblies.len() > 1000
            || !self.explosion.is_finite()
            || !(0.0..=1.0).contains(&self.explosion)
            || !self.explosion_distance.is_finite()
            || !(0.0..=1e12).contains(&self.explosion_distance)
        {
            return Err("montagens/explosão fora dos limites".into());
        }
        let mut ids = HashSet::new();
        for group in &self.assemblies {
            if group.id.is_empty() || group.name.trim().is_empty() || !ids.insert(&group.id) {
                return Err("montagem sem nome/identidade ou ID duplicado".into());
            }
            group.placement.transform()?;
        }
        for item in &self.items {
            if ids.contains(&item.id) {
                return Err("ID de objeto coincide com montagem".into());
            }
            if item
                .explode_direction
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1e6)
            {
                return Err("vetor de explosão inválido".into());
            }
        }
        for parent in self
            .assemblies
            .iter()
            .map(|g| g.parent.as_ref())
            .chain(self.items.iter().map(|i| i.parent.as_ref()))
            .flatten()
        {
            if !ids.contains(parent) {
                return Err("montagem pai ausente".into());
            }
        }
        for group in &self.assemblies {
            let mut chain = HashSet::new();
            let mut current = Some(group.id.as_str());
            while let Some(id) = current {
                if chain.len() >= 64 || !chain.insert(id) {
                    return Err("ciclo ou profundidade de montagem excedida".into());
                }
                current = self
                    .assemblies
                    .iter()
                    .find(|g| g.id == id)
                    .and_then(|g| g.parent.as_deref());
            }
        }
        Ok(())
    }
    pub fn belongs_to(&self, item: usize, group: &str) -> bool {
        let mut parent = self.items[item].parent.as_deref();
        for _ in 0..64 {
            match parent {
                Some(id) if id == group => return true,
                Some(id) => {
                    parent = self
                        .assemblies
                        .iter()
                        .find(|g| g.id == id)
                        .and_then(|g| g.parent.as_deref())
                }
                None => return false,
            }
        }
        false
    }
    pub fn world_transforms(&self, exploded: bool) -> Result<Vec<Transform>> {
        self.validate_hierarchy()?;
        let mut transforms = Vec::with_capacity(self.items.len());
        for item in &self.items {
            let mut t = item.transform()?;
            let mut parent = item.parent.as_deref();
            while let Some(id) = parent {
                let group = self
                    .assemblies
                    .iter()
                    .find(|g| g.id == id)
                    .ok_or("pai ausente")?;
                t = group.placement.transform()?.compose(&t)?;
                parent = group.parent.as_deref();
            }
            transforms.push(t);
        }
        if exploded
            && self.explosion > 0.
            && self.view.disassembly.mode != crate::disassembly::Mode::Radial
        {
            let offsets = crate::disassembly::offsets(self, &transforms)?;
            for (t, offset) in transforms.iter_mut().zip(offsets) {
                *t = Transform::new(offset, 0., 1.)?.compose(t)?;
            }
        } else if exploded && self.explosion > 0. && !self.items.is_empty() {
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            let mut centers = Vec::new();
            for (item, t) in self.items.iter().zip(&transforms) {
                let mut lo = [f64::INFINITY; 3];
                let mut hi = [f64::NEG_INFINITY; 3];
                for &p in item.geometry.positions() {
                    let p = t.apply(p)?;
                    for axis in 0..3 {
                        lo[axis] = lo[axis].min(p[axis]);
                        hi[axis] = hi[axis].max(p[axis]);
                        min[axis] = min[axis].min(p[axis]);
                        max[axis] = max[axis].max(p[axis]);
                    }
                }
                centers.push(if item.geometry.positions().is_empty() {
                    t.translation()
                } else {
                    std::array::from_fn(|i| (lo[i] + hi[i]) * 0.5)
                });
            }
            let center: [f64; 3] = std::array::from_fn(|i| {
                if min[i].is_finite() {
                    (min[i] + max[i]) * 0.5
                } else {
                    0.
                }
            });
            for (i, t) in transforms.iter_mut().enumerate() {
                let explicit = self.items[i].explode_direction;
                let mut d = if explicit.iter().any(|v| v.abs() > 1e-12) {
                    explicit
                } else {
                    std::array::from_fn(|axis| centers[i][axis] - center[axis])
                };
                let norm = d.iter().map(|v| v * v).sum::<f64>().sqrt();
                if norm < 1e-12 {
                    d = [0.; 3];
                    d[i % 3] = if (i / 3) % 2 == 0 { 1. } else { -1. };
                } else {
                    d = d.map(|v| v / norm);
                }
                let offset = d.map(|v| v * self.explosion_distance * self.explosion);
                *t = Transform::new(offset, 0., 1.)?.compose(t)?;
            }
        }
        Ok(transforms)
    }
    pub fn validate_world(&self) -> Result<()> {
        let transforms = self.world_transforms(true)?;
        for (item, t) in self.items.iter().zip(transforms) {
            for &p in item.geometry.positions() {
                if t.apply(p)?.iter().any(|v| v.abs() > 1e12) {
                    return Err("montagem fora do alcance visual de +/-1e12".into());
                }
            }
        }
        Ok(())
    }
    pub fn set_parent(&mut self, id: &str, parent: Option<String>) -> Result<()> {
        let old = if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            std::mem::replace(&mut item.parent, parent)
        } else if let Some(group) = self.assemblies.iter_mut().find(|g| g.id == id) {
            std::mem::replace(&mut group.parent, parent)
        } else {
            return Err("objeto/montagem ausente".into());
        };
        if let Err(error) = self.validate_world() {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.parent = old;
            } else if let Some(group) = self.assemblies.iter_mut().find(|g| g.id == id) {
                group.parent = old;
            }
            return Err(error);
        }
        Ok(())
    }
    pub fn set_group_placement(&mut self, id: &str, placement: Placement) -> Result<()> {
        let group = self
            .assemblies
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or("montagem ausente")?;
        let old = std::mem::replace(&mut group.placement, placement);
        if let Err(error) = self.validate_world() {
            self.assemblies
                .iter_mut()
                .find(|g| g.id == id)
                .ok_or("montagem ausente")?
                .placement = old;
            return Err(error);
        }
        Ok(())
    }
}
