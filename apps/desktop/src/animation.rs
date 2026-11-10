use noah_workspace::Camera;
#[derive(Clone)]
pub struct ExplosionTransition {
    pub from: f64,
    pub to: f64,
    pub camera_from: Camera,
    pub camera_to: Camera,
    pub elapsed: f64,
    pub duration: f64,
}
impl ExplosionTransition {
    pub fn amount(&self) -> f64 {
        let t = (self.elapsed / self.duration.max(1e-6)).clamp(0., 1.);
        t * t * (3. - 2. * t)
    }
    pub fn value(&self) -> f64 {
        self.from + (self.to - self.from) * self.amount()
    }
    pub fn camera(&self) -> Camera {
        let t = self.amount();
        let mut c = self.camera_to.clone();
        c.target = std::array::from_fn(|i| {
            self.camera_from.target[i] + (self.camera_to.target[i] - self.camera_from.target[i]) * t
        });
        c.distance =
            self.camera_from.distance + (self.camera_to.distance - self.camera_from.distance) * t;
        c
    }
    pub fn tick(&mut self, dt: f64) -> bool {
        self.elapsed = (self.elapsed + dt.clamp(0., 0.1)).min(self.duration);
        self.elapsed < self.duration
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smooth_reversible_and_exact_endpoints() {
        let c = Camera::default();
        let mut t = ExplosionTransition {
            from: 0.,
            to: 1.,
            camera_from: c.clone(),
            camera_to: c,
            elapsed: 0.,
            duration: 1.,
        };
        assert_eq!(t.value(), 0.);
        let mut old = 0.;
        for _ in 0..10 {
            t.tick(0.1);
            assert!(t.value() >= old);
            old = t.value();
        }
        t.tick(0.1);
        assert_eq!(t.value(), 1.);
        t.from = 1.;
        t.to = 0.;
        t.elapsed = 0.;
        assert_eq!(t.value(), 1.);
        t.elapsed = 1.;
        assert_eq!(t.value(), 0.);
    }
}
