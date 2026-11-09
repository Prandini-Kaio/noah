use crate::Result;
use serde::{Deserialize, Serialize};
pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: [f64; 3], s: f64) -> [f64; 3] {
    a.map(|v| v * s)
}
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub target: [f64; 3],
    pub distance: f64,
    pub yaw: f64,
    pub pitch: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            target: [0.; 3],
            distance: 5.,
            yaw: -1.1,
            pitch: 0.45,
        }
    }
}
impl Camera {
    pub fn validate(&self) -> Result<()> {
        if self.target.iter().any(|v| !v.is_finite() || v.abs() > 1e12)
            || !self.distance.is_finite()
            || !(1e-9..=1e15).contains(&self.distance)
            || !self.yaw.is_finite()
            || self.yaw.abs() > 1e6
            || !self.pitch.is_finite()
            || self.pitch.abs() > 1.55
        {
            return Err("estado de câmera fora dos limites".into());
        }
        Ok(())
    }
    /// Retorna posição, direita, cima e direção do olho para o alvo.
    pub fn basis(&self) -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let forward = [-cp * cy, -cp * sy, -sp];
        let right = [-sy, cy, 0.];
        let up = [-sp * cy, -sp * sy, cp];
        (
            sub(self.target, mul(forward, self.distance)),
            right,
            up,
            forward,
        )
    }
    /// Tela local em pixels, origem no canto superior esquerdo, FOV vertical 50°.
    pub fn project(&self, point: [f64; 3], size: [f64; 2]) -> Option<[f64; 3]> {
        if size.iter().any(|v| !v.is_finite() || *v <= 0.) {
            return None;
        }
        let (eye, right, up, forward) = self.basis();
        let delta = sub(point, eye);
        let depth = dot(delta, forward);
        if !depth.is_finite() || depth <= self.distance * 1e-5 {
            return None;
        }
        let focal = size[1] * 0.5 / (25f64.to_radians().tan());
        let result = [
            size[0] * 0.5 + dot(delta, right) * focal / depth,
            size[1] * 0.5 - dot(delta, up) * focal / depth,
            depth,
        ];
        result.iter().all(|v| v.is_finite()).then_some(result)
    }
    pub fn orbit(&mut self, delta: [f64; 2]) {
        self.yaw = (self.yaw - delta[0] * 0.008).rem_euclid(std::f64::consts::TAU);
        self.pitch = (self.pitch + delta[1] * 0.008).clamp(-1.55, 1.55);
    }
    pub fn zoom(&mut self, delta: f64) {
        if delta.is_finite() {
            self.distance = (self.distance * (-delta * 0.002).exp()).clamp(1e-9, 1e15);
        }
    }
    pub fn pan(&mut self, delta: [f64; 2], height: f64) {
        if height <= 0. {
            return;
        }
        let (_, right, up, _) = self.basis();
        let scale = 2. * self.distance * 25f64.to_radians().tan() / height;
        let target = add(
            self.target,
            add(mul(right, -delta[0] * scale), mul(up, delta[1] * scale)),
        );
        if target.iter().all(|v| v.is_finite() && v.abs() <= 1e12) {
            self.target = target;
        }
    }
}
