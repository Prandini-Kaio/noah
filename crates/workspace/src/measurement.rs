//! Distâncias no estado montado; calibração explícita não modifica geometria.
use crate::{Result, Scene};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub item_id: String,
    pub vertex: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
    Metre,
    Millimetre,
}
impl LengthUnit {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Metre => "m",
            Self::Millimetre => "mm",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleReference {
    pub a: Anchor,
    pub b: Anchor,
    pub known_distance: f64,
    pub unit: LengthUnit,
    pub scene_distance: f64,
    pub provenance: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    pub id: String,
    pub name: String,
    pub a: Anchor,
    pub b: Anchor,
    pub method: String,
    pub uncertainty: Option<f64>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Distance {
    pub value: f64,
    pub unit: Option<LengthUnit>,
    pub scene_distance: f64,
}
fn length(a: [f64; 3], b: [f64; 3]) -> Result<f64> {
    let d = a
        .into_iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    if !d.is_finite() {
        return Err("distância fora do alcance numérico".into());
    }
    Ok(d)
}
impl Scene {
    pub fn anchor_position(&self, anchor: &Anchor, exploded: bool) -> Result<[f64; 3]> {
        let index = self
            .items
            .iter()
            .position(|i| i.id == anchor.item_id)
            .ok_or("peça da medição ausente")?;
        let point = self.items[index]
            .geometry
            .positions()
            .get(anchor.vertex)
            .ok_or("vértice da medição ausente")?;
        Ok(self.world_transforms(exploded)?[index].apply(*point)?)
    }
    pub fn raw_distance(&self, a: &Anchor, b: &Anchor) -> Result<f64> {
        length(
            self.anchor_position(a, false)?,
            self.anchor_position(b, false)?,
        )
    }
    pub fn calibrate(
        &mut self,
        a: Anchor,
        b: Anchor,
        known_distance: f64,
        unit: LengthUnit,
        provenance: String,
    ) -> Result<()> {
        if !known_distance.is_finite()
            || known_distance <= 0.
            || known_distance > 1e12
            || provenance.trim().is_empty()
            || provenance.len() > 1024
        {
            return Err(
                "declare distância positiva (até 1e12), unidade e origem da referência".into(),
            );
        }
        let scene_distance = self.raw_distance(&a, &b)?;
        if scene_distance <= 1e-12 {
            return Err("referência deve ter comprimento maior que 1e-12 unidades da cena".into());
        }
        if known_distance / scene_distance <= 0. {
            return Err("fator de escala fora do alcance numérico".into());
        }
        self.calibration = Some(ScaleReference {
            a,
            b,
            known_distance,
            unit,
            scene_distance,
            provenance,
        });
        Ok(())
    }
    pub fn scale_factor(&self) -> Result<Option<(f64, LengthUnit)>> {
        let Some(reference) = &self.calibration else {
            return Ok(None);
        };
        if !reference.known_distance.is_finite()
            || reference.known_distance <= 0.
            || reference.known_distance > 1e12
            || !reference.scene_distance.is_finite()
            || reference.scene_distance <= 1e-12
            || reference.provenance.trim().is_empty()
            || reference.provenance.len() > 1024
        {
            return Err("referência de escala inválida".into());
        }
        let now = self.raw_distance(&reference.a, &reference.b)?;
        if (now - reference.scene_distance).abs() > reference.scene_distance * 1e-9 {
            return Err(
                "referência de escala alterada; recalibre antes de usar medidas físicas".into(),
            );
        }
        let factor = reference.known_distance / reference.scene_distance;
        if !factor.is_finite() || factor <= 0. {
            return Err("fator de escala inválido".into());
        }
        Ok(Some((factor, reference.unit)))
    }
    pub fn distance(&self, a: &Anchor, b: &Anchor) -> Result<Distance> {
        let raw = self.raw_distance(a, b)?;
        let (factor, unit) = match self.scale_factor()? {
            Some((f, u)) => (f, Some(u)),
            None => (1., None),
        };
        let value = raw * factor;
        if !value.is_finite() {
            return Err("medida fora do alcance numérico".into());
        }
        Ok(Distance {
            value,
            unit,
            scene_distance: raw,
        })
    }
    pub fn add_measurement(&mut self, a: Anchor, b: Anchor, name: String) -> Result<String> {
        if self.measurements.len() >= 1000 || name.trim().is_empty() || name.len() > 256 {
            return Err("nome vazio/inválido ou limite de 1000 medidas".into());
        }
        self.raw_distance(&a, &b)?;
        if a == b {
            return Err("escolha dois vértices distintos".into());
        }
        let id = format!("measurement:{:032x}", noah_storage::new_id());
        self.measurements.push(Measurement {
            id: id.clone(),
            name,
            a,
            b,
            method: "Distância euclidiana entre vértices, estado montado".into(),
            uncertainty: None,
        });
        Ok(id)
    }
    pub fn validate_measurements(&self) -> Result<()> {
        if self.measurements.len() > 1000 {
            return Err("limite de medidas excedido".into());
        }
        let mut ids = HashSet::new();
        for m in &self.measurements {
            if m.id.is_empty()
                || m.id.len() > 128
                || !ids.insert(&m.id)
                || m.name.trim().is_empty()
                || m.name.len() > 256
                || m.method != "Distância euclidiana entre vértices, estado montado"
                || m.uncertainty.is_some()
            {
                return Err("medição inválida ou incerteza não suportada".into());
            }
            self.raw_distance(&m.a, &m.b)?;
            if m.a == m.b {
                return Err("medição sem comprimento válido".into());
            }
        }
        if let Some(c) = &self.calibration {
            if !c.known_distance.is_finite()
                || c.known_distance <= 0.
                || c.known_distance > 1e12
                || !c.scene_distance.is_finite()
                || c.scene_distance <= 1e-12
                || c.provenance.trim().is_empty()
                || c.provenance.len() > 1024
            {
                return Err("referência de escala inválida".into());
            }
            self.raw_distance(&c.a, &c.b)?;
        }
        Ok(())
    }
}
/// Snap de vértice em pixels. Oclusão segue a limitação aproximada do renderer.
pub fn pick_anchor(scene: &Scene, point: [f64; 2], size: [f64; 2], radius: f64) -> Option<Anchor> {
    let transforms = scene.world_transforms(true).ok()?;
    let mut best: Option<(f64, f64, Anchor)> = None;
    for (index, item) in scene.items.iter().enumerate() {
        if !item.visible {
            continue;
        }
        for (vertex, &p) in item.geometry.positions().iter().enumerate() {
            let projected = transforms[index]
                .apply(p)
                .ok()
                .and_then(|p| scene.camera.project(p, size));
            if let Some(p) = projected {
                if p[0] < 0. || p[0] > size[0] || p[1] < 0. || p[1] > size[1] {
                    continue;
                }
                let d = (p[0] - point[0]).powi(2) + (p[1] - point[1]).powi(2);
                if d <= radius * radius
                    && best.as_ref().is_none_or(|(old, depth, _)| {
                        d < *old - 0.25 || ((d - old).abs() <= 0.25 && p[2] < *depth)
                    })
                {
                    best = Some((
                        d,
                        p[2],
                        Anchor {
                            item_id: item.id.clone(),
                            vertex,
                        },
                    ));
                }
            }
        }
    }
    best.map(|(_, _, a)| a)
}

/// Relatório reexecutável com âncoras/proveniência; recusa escala desatualizada.
pub fn export_report(scene: &Scene, path: &std::path::Path) -> Result<()> {
    scene.validate_measurements()?;
    scene.scale_factor()?;
    let mut entries = Vec::new();
    for measurement in &scene.measurements {
        let d = scene.distance(&measurement.a, &measurement.b)?;
        entries.push(serde_json::json!({"id":measurement.id,"name":measurement.name,"a":measurement.a,"b":measurement.b,"value":d.value,"unit":d.unit.map(|u|u.symbol()).unwrap_or("scene_units"),"scene_distance":d.scene_distance,"method":measurement.method,"uncertainty":measurement.uncertainty}));
    }
    let report = serde_json::json!({"schema_version":1,"state":"assembled","metrology_validated":false,"calibration":scene.calibration,"sources":scene.sources,"measurements":entries});
    noah_pipeline::write_new(path, &serde_json::to_vec_pretty(&report)?)
}

/// Seleção controlada: até oito candidatos de tela, teste frontal em malhas.
/// Acima de 200 mil faces visíveis, isolar peças ou desabilitar o teste frontal.
pub fn pick_anchor_controlled(scene: &Scene, point: [f64; 2], size: [f64; 2]) -> Option<Anchor> {
    use crate::{
        Geometry,
        camera::{self, Projection},
    };
    let options = &scene.view;
    if options.selected_only && scene.selected.is_none() {
        return None;
    }
    let transforms = scene.world_transforms(true).ok()?;
    let faces: usize = scene
        .items
        .iter()
        .filter(|i| i.visible)
        .map(|i| match i.geometry.as_ref() {
            Geometry::Mesh(m) => m.triangles().len(),
            _ => 0,
        })
        .sum();
    if options.occlusion && faces > 200_000 {
        return None;
    }
    let mut candidates: Vec<(f64, f64, Anchor, [f64; 3])> = Vec::new();
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for (index, item) in scene.items.iter().enumerate() {
        if !item.visible {
            continue;
        }
        let eligible = !options.selected_only
            || scene.selected.as_ref() == Some(&item.id)
            || scene
                .selected
                .as_ref()
                .is_some_and(|id| scene.belongs_to(index, id));
        for (vertex, &p) in item.geometry.positions().iter().enumerate() {
            let p = transforms[index].apply(p).ok()?;
            for a in 0..3 {
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
            if !eligible {
                continue;
            }
            if let Some(q) = scene.camera.project(p, size) {
                if q[0] < 0. || q[0] > size[0] || q[1] < 0. || q[1] > size[1] {
                    continue;
                }
                let d = (q[0] - point[0]).powi(2) + (q[1] - point[1]).powi(2);
                if d <= options.snap_radius.powi(2) {
                    candidates.push((
                        d,
                        q[2],
                        Anchor {
                            item_id: item.id.clone(),
                            vertex,
                        },
                        p,
                    ));
                    candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
                    candidates.truncate(8);
                }
            }
        }
    }
    let diagonal = camera::sub(hi, lo)
        .iter()
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();
    let reach = diagonal * 2. + scene.camera.distance + 1.;
    for (_, _, anchor, point) in candidates {
        if !options.occlusion || faces == 0 {
            return Some(anchor);
        }
        let (eye, _, _, forward) = scene.camera.basis();
        let origin = if scene.camera.projection == Projection::Orthographic {
            camera::sub(point, camera::mul(forward, reach))
        } else {
            eye
        };
        let delta = camera::sub(point, origin);
        let distance = camera::dot(delta, delta).sqrt();
        if distance <= 0. {
            continue;
        }
        let direction = camera::mul(delta, 1. / distance);
        let tolerance = (distance * 1e-10).max(1e-10);
        let mut occluded = false;
        'objects: for (index, item) in scene.items.iter().enumerate() {
            if !item.visible {
                continue;
            }
            if let Geometry::Mesh(mesh) = item.geometry.as_ref() {
                for triangle in mesh.triangles() {
                    let tri =
                        triangle.map(|i| transforms[index].apply(mesh.positions()[i as usize]));
                    let [Ok(a), Ok(b), Ok(c)] = tri else {
                        continue;
                    };
                    if let Some(t) = ray_triangle(origin, direction, [a, b, c])
                        && t < distance - tolerance
                    {
                        occluded = true;
                        break 'objects;
                    }
                }
            }
        }
        if !occluded {
            return Some(anchor);
        }
    }
    None
}
fn ray_triangle(origin: [f64; 3], direction: [f64; 3], tri: [[f64; 3]; 3]) -> Option<f64> {
    use crate::camera::{dot, sub};
    let cross = |a: [f64; 3], b: [f64; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let a = sub(tri[1], tri[0]);
    let b = sub(tri[2], tri[0]);
    let p = cross(direction, b);
    let determinant = dot(a, p);
    let epsilon = 1e-12 * (dot(a, a) * dot(b, b)).sqrt().max(1e-24);
    if determinant.abs() <= epsilon {
        return None;
    }
    let inverse = 1. / determinant;
    let s = sub(origin, tri[0]);
    let u = dot(s, p) * inverse;
    if !(-1e-10..=1. + 1e-10).contains(&u) {
        return None;
    }
    let q = cross(s, a);
    let v = dot(direction, q) * inverse;
    if v < -1e-10 || u + v > 1. + 1e-10 {
        return None;
    }
    let t = dot(b, q) * inverse;
    (t > 1e-12 && t.is_finite()).then_some(t)
}
