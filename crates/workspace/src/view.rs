//! Preferências de apresentação e escala das réguas, sem dependência da UI.
use crate::{Camera, Result, Scene};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    ArcticDark,
    ArcticLight,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ViewOptions {
    pub theme: ThemeMode,
    pub disassembly: crate::disassembly::Options,
    pub rulers: bool,
    pub axes: bool,
    pub annotations: bool,
    pub reduced_motion: bool,
    pub decimals: usize,
    pub snap_radius: f64,
    pub selected_only: bool,
    pub occlusion: bool,
}
impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            theme: ThemeMode::ArcticDark,
            disassembly: Default::default(),
            rulers: true,
            axes: true,
            annotations: true,
            reduced_motion: false,
            decimals: 4,
            snap_radius: 8.,
            selected_only: false,
            occlusion: true,
        }
    }
}
impl ViewOptions {
    pub fn validate(&self) -> Result<()> {
        self.disassembly.validate()?;
        if self.decimals > 9
            || !self.snap_radius.is_finite()
            || !(3.0..=24.0).contains(&self.snap_radius)
        {
            return Err("preferências de medição inválidas".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct RulerScale {
    pub units_per_pixel: f64,
    pub unit: &'static str,
    pub calibrated: bool,
    pub stale: bool,
    pub perspective: bool,
    pub origin: [f64; 2],
}
pub fn ruler_scale(scene: &Scene, camera: &Camera, height: f64) -> Result<RulerScale> {
    camera.validate()?;
    if !height.is_finite() || height <= 0. {
        return Err("altura inválida".into());
    }
    let (factor, unit, calibrated, stale) = match scene.scale_factor() {
        Ok(Some((f, u))) => (f, u.symbol(), true, false),
        Ok(None) => (1., "u. cena", false, false),
        Err(_) => (1., "u. cena", false, true),
    };
    let (_, right, up, _) = camera.basis();
    Ok(RulerScale {
        units_per_pixel: camera.units_per_pixel(height) * factor,
        unit,
        calibrated,
        stale,
        perspective: camera.projection == crate::camera::Projection::Perspective,
        origin: [
            crate::camera::dot(camera.target, right) * factor,
            crate::camera::dot(camera.target, up) * factor,
        ],
    })
}
/// Passo 1/2/5 × potência de dez para etiquetas legíveis.
pub fn nice_step(value: f64) -> f64 {
    if !value.is_finite() || value <= 0. {
        return 1.;
    }
    let exponent = 10f64.powf(value.log10().floor());
    let fraction = value / exponent;
    let step = if fraction <= 1. {
        1.
    } else if fraction <= 2. {
        2.
    } else if fraction <= 5. {
        5.
    } else {
        10.
    };
    step * exponent
}
