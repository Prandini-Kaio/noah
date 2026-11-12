//! Assembled-state angular inspection and world-axis bounds. No geometry mutation.
use crate::{
    Result, Scene,
    camera::{dot, sub},
    measurement::{Anchor, LengthUnit},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
pub const ANGLE_METHOD: &str = "Ângulo ABC no vértice B, estado montado";
pub const DIMENSION_METHOD: &str = "Caixa XYZ mundial, todos os vértices e filhos, estado montado";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AngularMeasurement {
    pub id: String,
    pub name: String,
    pub a: Anchor,
    pub b: Anchor,
    pub c: Anchor,
    pub method: String,
    pub uncertainty: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionMeasurement {
    pub id: String,
    pub name: String,
    pub target_id: String,
    pub method: String,
    pub uncertainty: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub vertex_count: usize,
    pub item_count: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dimensions {
    pub bounds: Bounds,
    pub scene_size: [f64; 3],
    pub size: [f64; 3],
    pub unit: Option<LengthUnit>,
}
impl Bounds {
    /// Physical scale is explicit and validated by the scene, never inferred.
    pub fn dimensions(&self, scale: Option<(f64, LengthUnit)>) -> Result<Dimensions> {
        let scene_size = std::array::from_fn(|i| self.max[i] - self.min[i]);
        let (factor, unit) = scale.map(|(f, u)| (f, Some(u))).unwrap_or((1., None));
        let size = scene_size.map(|v| v * factor);
        if !factor.is_finite() || factor <= 0. || size.iter().any(|v| !v.is_finite() || *v < 0.) {
            return Err("dimensões fora do alcance numérico".into());
        }
        Ok(Dimensions {
            bounds: self.clone(),
            scene_size,
            size,
            unit,
        })
    }
}
impl Scene {
    /// Interior angle [0,180] degrees between BA and BC; B is the vertex.
    pub fn angle_degrees(&self, a: &Anchor, b: &Anchor, c: &Anchor) -> Result<f64> {
        if a == b || b == c || a == c {
            return Err("escolha três vértices distintos; B é o vértice do ângulo".into());
        }
        let b = self.anchor_position(b, false)?;
        let u = sub(self.anchor_position(a, false)?, b);
        let v = sub(self.anchor_position(c, false)?, b);
        let nu = u[0].hypot(u[1]).hypot(u[2]);
        let nv = v[0].hypot(v[1]).hypot(v[2]);
        if nu <= 1e-12 || nv <= 1e-12 {
            return Err("braços do ângulo devem exceder 1e-12 unidades da cena".into());
        }
        let u = u.map(|x| x / nu);
        let v = v.map(|x| x / nv);
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let sine = cross[0].hypot(cross[1]).hypot(cross[2]);
        let angle = sine.atan2(dot(u, v).clamp(-1., 1.)).to_degrees();
        if !angle.is_finite() {
            return Err("ângulo fora do alcance numérico".into());
        }
        Ok(angle)
    }
    /// Uses all descendants, including hidden objects; un-sampled, world-axis AABB.
    pub fn raw_bounds(&self, target: &str) -> Result<Bounds> {
        if !self.items.iter().any(|i| i.id == target)
            && !self.assemblies.iter().any(|g| g.id == target)
        {
            return Err("peça ou montagem das dimensões ausente".into());
        }
        let transforms = self.world_transforms(false)?;
        let mut out = Bounds {
            min: [f64::INFINITY; 3],
            max: [f64::NEG_INFINITY; 3],
            vertex_count: 0,
            item_count: 0,
        };
        for (index, item) in self.items.iter().enumerate() {
            if item.id != target && !self.belongs_to(index, target) {
                continue;
            }
            out.item_count += 1;
            for &p in item.geometry.positions() {
                let p = transforms[index].apply(p)?;
                for (k, &v) in p.iter().enumerate() {
                    out.min[k] = out.min[k].min(v);
                    out.max[k] = out.max[k].max(v);
                }
                out.vertex_count += 1;
            }
        }
        if out.vertex_count == 0 {
            return Err("seleção sem vértices para calcular dimensões".into());
        }
        Ok(out)
    }
    pub fn dimensions(&self, target: &str) -> Result<Dimensions> {
        self.raw_bounds(target)?.dimensions(self.scale_factor()?)
    }
    pub fn add_angle(&mut self, a: Anchor, b: Anchor, c: Anchor, name: String) -> Result<String> {
        if self.angles.len() >= 1000 || name.trim().is_empty() || name.len() > 256 {
            return Err("nome inválido ou limite de 1000 ângulos".into());
        }
        self.angle_degrees(&a, &b, &c)?;
        let id = format!("angle:{:032x}", noah_storage::new_id());
        self.angles.push(AngularMeasurement {
            id: id.clone(),
            name,
            a,
            b,
            c,
            method: ANGLE_METHOD.into(),
            uncertainty: None,
        });
        Ok(id)
    }
    pub fn add_dimensions(&mut self, target_id: String, name: String) -> Result<String> {
        if self.dimension_measurements.len() >= 100 || name.trim().is_empty() || name.len() > 256 {
            return Err("nome inválido ou limite de 100 caixas dimensionais".into());
        }
        self.raw_bounds(&target_id)?;
        let id = format!("dimensions:{:032x}", noah_storage::new_id());
        self.dimension_measurements.push(DimensionMeasurement {
            id: id.clone(),
            name,
            target_id,
            method: DIMENSION_METHOD.into(),
            uncertainty: None,
        });
        Ok(id)
    }
    pub(crate) fn validate_inspection(&self, ids: &mut HashSet<String>) -> Result<()> {
        if self.angles.len() > 1000 || self.dimension_measurements.len() > 100 {
            return Err("limite de inspeções excedido".into());
        }
        let valid =
            |id: &str, name: &str, method: &str, expected: &str, uncertainty: Option<f64>| {
                !id.is_empty()
                    && id.len() <= 128
                    && !name.trim().is_empty()
                    && name.len() <= 256
                    && method == expected
                    && uncertainty.is_none()
            };
        for m in &self.angles {
            if !valid(&m.id, &m.name, &m.method, ANGLE_METHOD, m.uncertainty)
                || !ids.insert(m.id.clone())
            {
                return Err("ângulo registrado inválido".into());
            }
            self.angle_degrees(&m.a, &m.b, &m.c)?;
        }
        for m in &self.dimension_measurements {
            if !valid(&m.id, &m.name, &m.method, DIMENSION_METHOD, m.uncertainty)
                || !ids.insert(m.id.clone())
            {
                return Err("dimensões registradas inválidas".into());
            }
            self.raw_bounds(&m.target_id)?;
        }
        Ok(())
    }
}
