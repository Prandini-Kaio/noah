//! One-pair translational alignment in assembled world XYZ. Raw assets stay intact.
use crate::{Placement, Result, Scene, camera::sub, measurement::Anchor};
use noah_geometry::Transform;
#[derive(Debug, Clone)]
struct GeometryPin(std::sync::Arc<crate::Geometry>);
impl PartialEq for GeometryPin {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Signature {
    items: Vec<(String, Option<String>, Transform, GeometryPin)>,
    groups: Vec<(String, Option<String>, Placement)>,
}
fn signature(scene: &Scene) -> Result<Signature> {
    Ok(Signature {
        items: scene
            .items
            .iter()
            .zip(scene.world_transforms(false)?)
            .map(|(i, t)| {
                (
                    i.id.clone(),
                    i.parent.clone(),
                    t,
                    GeometryPin(i.geometry.clone()),
                )
            })
            .collect(),
        groups: scene
            .assemblies
            .iter()
            .map(|g| (g.id.clone(), g.parent.clone(), g.placement.clone()))
            .collect(),
    })
}
#[derive(Debug, Clone, PartialEq)]
pub struct AlignmentPlan {
    source: Anchor,
    target: Anchor,
    delta: [f64; 3],
    before: Placement,
    after: Placement,
    context: Signature,
}
impl AlignmentPlan {
    pub fn source(&self) -> &Anchor {
        &self.source
    }
    pub fn target(&self) -> &Anchor {
        &self.target
    }
    pub fn delta(&self) -> [f64; 3] {
        self.delta
    }
    pub fn placement(&self) -> &Placement {
        &self.after
    }
    pub fn is_current(&self, scene: &Scene) -> bool {
        signature(scene).is_ok_and(|s| s == self.context)
    }
}
#[derive(Debug, Clone)]
pub struct AlignmentUndo {
    source_id: String,
    before: Placement,
    expected: Signature,
}
impl AlignmentUndo {
    pub fn is_current(&self, scene: &Scene) -> bool {
        signature(scene).is_ok_and(|s| s == self.expected)
    }
}
impl Scene {
    /// Translation only: move source vertex onto target vertex, preserving angles/scale.
    pub fn plan_alignment(&self, source: Anchor, target: Anchor) -> Result<AlignmentPlan> {
        if source.item_id == target.item_id {
            return Err("escolha duas peças diferentes para alinhar".into());
        }
        let from = self.anchor_position(&source, false)?;
        let to = self.anchor_position(&target, false)?;
        let delta = sub(to, from);
        let index = self
            .items
            .iter()
            .position(|i| i.id == source.item_id)
            .ok_or("peça móvel ausente")?;
        let item = &self.items[index];
        let before = item.placement.clone();
        let mut after = before.clone();
        let mut parent = Transform::default();
        let mut current = item.parent.as_deref();
        while let Some(id) = current {
            let g = self
                .assemblies
                .iter()
                .find(|g| g.id == id)
                .ok_or("montagem pai ausente")?;
            parent = g.placement.transform()?.compose(&parent)?;
            current = g.parent.as_deref();
        }
        let r = parent.rotation();
        let scale = parent.scale();
        let local: [f64; 3] =
            std::array::from_fn(|i| (0..3).map(|j| r[j][i] * delta[j]).sum::<f64>() / scale);
        for (v, d) in after.translation.iter_mut().zip(local) {
            *v += d;
        }
        let mut preview = self.clone();
        preview.set_placement(index, after.clone())?;
        Ok(AlignmentPlan {
            source,
            target,
            delta,
            before,
            after,
            context: signature(self)?,
        })
    }
    pub fn apply_alignment(&mut self, plan: &AlignmentPlan) -> Result<AlignmentUndo> {
        if !plan.is_current(self) {
            return Err("cena mudou; recalcule o alinhamento antes de aplicar".into());
        }
        let index = self
            .items
            .iter()
            .position(|i| i.id == plan.source.item_id)
            .ok_or("peça móvel ausente")?;
        self.set_placement(index, plan.after.clone())?;
        let expected = match signature(self) {
            Ok(s) => s,
            Err(e) => {
                self.items[index].placement = plan.before.clone();
                return Err(e);
            }
        };
        Ok(AlignmentUndo {
            source_id: plan.source.item_id.clone(),
            before: plan.before.clone(),
            expected,
        })
    }
    pub fn undo_alignment(&mut self, undo: &AlignmentUndo) -> Result<()> {
        if !undo.is_current(self) {
            return Err(
                "outras alterações ocorreram; desfazer alinhamento não está mais disponível".into(),
            );
        }
        let index = self
            .items
            .iter()
            .position(|i| i.id == undo.source_id)
            .ok_or("peça móvel ausente")?;
        self.set_placement(index, undo.before.clone())
    }
}
