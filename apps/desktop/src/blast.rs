//! Disposable presentation geometry. Right-handed world XYZ; no assets are mutated.
use noah_workspace::{
    Camera, Geometry, Primitive, Scene,
    camera::{add, dot, mul, sub},
};
const LIMIT: usize = 20_000;
#[derive(Clone)]
struct Shard {
    item: usize,
    vertices: Vec<[f64; 3]>,
    center: [f64; 3],
    velocity: [f64; 3],
    axis: [f64; 3],
    spin: f64,
    color: [u8; 3],
}
pub struct Blast {
    shards: Vec<Shard>,
    panels: Option<usize>,
    pub targets: Vec<usize>,
    pub progress: f64,
    pub playing: bool,
    pub reverse: bool,
    pub strength: f64,
    radius: f64,
    pub frame: Option<(Camera, Camera)>,
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: [f64; 3]) -> [f64; 3] {
    let n = dot(a, a).sqrt();
    if n > 1e-15 {
        mul(a, 1. / n)
    } else {
        [1., 0., 0.]
    }
}
fn noise(seed: usize, lane: usize) -> f64 {
    let mut n = (seed as u64)
        .wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add(lane as u64 + 17);
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((n ^ (n >> 31)) >> 11) as f64 / ((1u64 << 53) as f64) * 2. - 1.
}
fn split(t: [[f64; 3]; 3]) -> [[[f64; 3]; 3]; 4] {
    let ab = mul(add(t[0], t[1]), 0.5);
    let bc = mul(add(t[1], t[2]), 0.5);
    let ca = mul(add(t[2], t[0]), 0.5);
    [[t[0], ab, ca], [ab, t[1], bc], [ca, bc, t[2]], [ab, bc, ca]]
}
impl Blast {
    pub fn new(scene: &Scene, selected_only: bool, strength: f64) -> noah_workspace::Result<Self> {
        Self::with_detail(scene, selected_only, strength, 3)
    }
    pub fn with_detail(
        scene: &Scene,
        selected_only: bool,
        strength: f64,
        detail: u32,
    ) -> noah_workspace::Result<Self> {
        if !strength.is_finite() || !(0.2..=6.).contains(&strength) || detail > 4 {
            return Err("força/detalhe de explosão inválidos".into());
        }
        let transforms = scene.world_transforms(true)?;
        let targets: Vec<_> = scene
            .items
            .iter()
            .enumerate()
            .filter(|(i, item)| {
                item.visible
                    && (!selected_only
                        || scene.selected.as_ref() == Some(&item.id)
                        || scene
                            .selected
                            .as_ref()
                            .is_some_and(|id| scene.belongs_to(*i, id)))
            })
            .map(|(i, _)| i)
            .collect();
        if targets.is_empty() {
            return Err("Selecione uma peça/montagem visível ou escolha Toda a cena".into());
        }
        let faces: usize = targets
            .iter()
            .map(|&i| match scene.items[i].geometry.as_ref() {
                Geometry::Mesh(m) => m.triangles().len(),
                Geometry::Points(p) => p.positions().len(),
            })
            .sum();
        let stride = faces.div_ceil(LIMIT).max(1);
        let mesh_faces: usize = targets
            .iter()
            .map(|&i| match scene.items[i].geometry.as_ref() {
                Geometry::Mesh(m) => m.triangles().len(),
                _ => 0,
            })
            .sum();
        let cloud_points = faces - mesh_faces;
        let mut level = detail;
        while level > 0
            && mesh_faces.div_ceil(stride) * 4usize.pow(level) + cloud_points.div_ceil(stride)
                > LIMIT
        {
            level -= 1;
        }

        let mut raw = Vec::new();
        let mut serial = 0;
        let palette = [
            [96, 156, 181],
            [155, 174, 189],
            [225, 173, 93],
            [103, 189, 171],
            [134, 154, 206],
        ];
        for &i in &targets {
            match scene.items[i].geometry.as_ref() {
                Geometry::Mesh(m) => {
                    for face in m.triangles() {
                        let take = serial % stride == 0;
                        serial += 1;
                        if !take {
                            continue;
                        }
                        let mut t = [[0.; 3]; 3];
                        for k in 0..3 {
                            t[k] = transforms[i].apply(m.positions()[face[k] as usize])?;
                        }
                        let mut fragments = vec![t];
                        for _ in 0..level {
                            fragments = fragments.into_iter().flat_map(split).collect();
                        }
                        for tri in fragments {
                            raw.push((i, tri.to_vec(), palette[i % 5]));
                        }
                    }
                }
                Geometry::Points(p) => {
                    for (j, &point) in p.positions().iter().enumerate() {
                        let take = serial % stride == 0;
                        serial += 1;
                        if take {
                            raw.push((
                                i,
                                vec![transforms[i].apply(point)?],
                                p.colors().map(|c| c[j]).unwrap_or(palette[i % 5]),
                            ));
                        }
                    }
                }
            }
        }
        if raw.is_empty() {
            return Err("A seleção não contém geometria para animar".into());
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for (_, vs, _) in &raw {
            for p in vs {
                for k in 0..3 {
                    min[k] = min[k].min(p[k]);
                    max[k] = max[k].max(p[k]);
                }
            }
        }
        let origin = mul(add(min, max), 0.5);
        let radius = dot(sub(max, min), sub(max, min)).sqrt().max(1e-9) * 0.5;
        let shards = raw
            .into_iter()
            .enumerate()
            .map(|(seed, (item, vertices, color))| {
                let center = mul(
                    vertices.iter().copied().fold([0.; 3], add),
                    1. / vertices.len() as f64,
                );
                let jitter = std::array::from_fn(|k| noise(seed + item * 7919, k));
                let radial = mul(sub(center, origin), 1. / radius);
                let velocity = mul(
                    unit(add(radial, mul(jitter, 0.8))),
                    radius * (2.0 + 2.0 * noise(seed, 4).abs()),
                );
                Shard {
                    item,
                    vertices,
                    center,
                    velocity,
                    axis: unit(jitter),
                    spin: noise(seed, 5) * 5.,
                    color,
                }
            })
            .collect();
        Ok(Self {
            shards,
            panels: None,
            targets,
            progress: 0.,
            playing: true,
            reverse: false,
            strength,
            frame: None,
            radius,
        })
    }
    /// Rigid surface panels: original faces grouped by orientation and local region.
    pub fn with_panels(
        scene: &Scene,
        selected_only: bool,
        strength: f64,
    ) -> noah_workspace::Result<Self> {
        let mut meshes = scene.clone();
        for item in &mut meshes.items {
            if !matches!(item.geometry.as_ref(), Geometry::Mesh(_)) {
                item.visible = false;
            }
        }
        let mut blast = Self::with_detail(&meshes, selected_only, strength, 0)?;
        let mut groups: std::collections::BTreeMap<(usize, usize, usize, usize), Vec<usize>> =
            std::collections::BTreeMap::new();
        let mut bounds = std::collections::BTreeMap::new();
        for &i in &blast.targets {
            let mut lo = [f64::INFINITY; 3];
            let mut hi = [f64::NEG_INFINITY; 3];
            for s in blast.shards.iter().filter(|s| s.item == i) {
                for p in &s.vertices {
                    for a in 0..3 {
                        lo[a] = lo[a].min(p[a]);
                        hi[a] = hi[a].max(p[a]);
                    }
                }
            }
            bounds.insert(i, (lo, hi));
        }
        for (i, s) in blast.shards.iter().enumerate() {
            let n = cross(
                sub(s.vertices[1], s.vertices[0]),
                sub(s.vertices[2], s.vertices[0]),
            );
            let axis = (0..3)
                .max_by(|&a, &b| n[a].abs().total_cmp(&n[b].abs()))
                .unwrap_or(0);
            let (lo, hi) = bounds[&s.item];
            let center: [f64; 3] = std::array::from_fn(|a| (lo[a] + hi[a]) * 0.5);
            let face = axis * 2 + usize::from(s.center[axis] >= center[axis]);
            let cell = |a: usize| {
                if matches!(meshes.items[s.item].geometry.as_ref(),Geometry::Mesh(m) if m.triangles().len()<=48)
                {
                    return 0;
                }
                (((s.center[a] - lo[a]) / (hi[a] - lo[a]).max(1e-12) * 2.)
                    .floor()
                    .clamp(0., 1.)) as usize
            };
            groups
                .entry((s.item, face, cell((axis + 1) % 3), cell((axis + 2) % 3)))
                .or_default()
                .push(i);
        }
        for (seed, indices) in groups.values().enumerate() {
            let mut center = [0.; 3];
            let mut count = 0.;
            for &i in indices {
                for &p in &blast.shards[i].vertices {
                    center = add(center, p);
                    count += 1.;
                }
            }
            center = mul(center, 1. / count);
            let item = blast.shards[indices[0]].item;
            let (lo, hi) = bounds[&item];
            let origin = mul(add(lo, hi), 0.5);
            let velocity = mul(unit(sub(center, origin)), blast.radius * 1.6);
            let axis = unit(std::array::from_fn(|a| noise(seed, a)));
            for &i in indices {
                let s = &mut blast.shards[i];
                s.center = center;
                s.velocity = velocity;
                s.axis = axis;
                s.spin = noise(seed, 5) * 0.45;
            }
        }
        blast.panels = Some(groups.len());
        Ok(blast)
    }
    pub fn follow_camera(&mut self, camera: &Camera) {
        let old = self.progress;
        self.progress = 1.;
        let mut to = camera.clone();
        let ok = self.fit_camera(&mut to).is_ok();
        self.progress = old;
        if ok {
            self.frame = Some((camera.clone(), to));
        }
    }
    pub fn camera(&self) -> Option<Camera> {
        self.frame.as_ref().map(|(from, to)| {
            let t = self.progress * self.progress * (3. - 2. * self.progress);
            let mut c = from.clone();
            c.target =
                std::array::from_fn(|i| from.target[i] + (to.target[i] - from.target[i]) * t);
            c.distance = from.distance + (to.distance - from.distance) * t;
            c
        })
    }
    pub fn fit_camera(&self, camera: &mut Camera) -> noah_workspace::Result<()> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for s in &self.shards {
            for p in self.vertices(s) {
                for k in 0..3 {
                    min[k] = min[k].min(p[k]);
                    max[k] = max[k].max(p[k]);
                }
            }
        }
        let mut next = camera.clone();
        next.target = mul(add(min, max), 0.5);
        let span = sub(max, min);
        let radius = dot(span, span).sqrt() * 0.5;
        next.distance = (radius.max(1e-6) / 25f64.to_radians().sin() * 1.3).clamp(1e-9, 1e15);
        next.validate()?;
        *camera = next;
        Ok(())
    }
    pub fn count(&self) -> usize {
        self.panels.unwrap_or(self.shards.len())
    }
    pub fn tick(&mut self, dt: f64) {
        if self.playing {
            self.progress = (self.progress
                + dt.clamp(0., 0.1) / 1.6 * if self.reverse { -1. } else { 1. })
            .clamp(0., 1.);
            if self.progress == 0. || self.progress == 1. {
                self.playing = false;
            }
        }
    }
    fn vertices(&self, s: &Shard) -> Vec<[f64; 3]> {
        let t = self.progress;
        let travel = (1. - (-3. * t).exp()) / (1. - (-3f64).exp());
        let shift = add(
            mul(s.velocity, travel * self.strength),
            [0., 0., -self.radius * 0.2 * t * t * self.strength],
        );
        let angle = s.spin * t;
        let (sn, cs) = angle.sin_cos();
        s.vertices
            .iter()
            .map(|&p| {
                if t == 0. {
                    return p;
                }
                let v = sub(p, s.center);
                let rotated = add(
                    add(mul(v, cs), mul(cross(s.axis, v), sn)),
                    mul(s.axis, dot(s.axis, v) * (1. - cs)),
                );
                add(add(s.center, rotated), shift)
            })
            .collect()
    }
    pub fn project(&self, camera: &Camera, size: [f64; 2]) -> Vec<Primitive> {
        let mut out = Vec::with_capacity(self.shards.len());
        for s in &self.shards {
            if let Some(points) = self
                .vertices(s)
                .into_iter()
                .map(|p| camera.project(p, size))
                .collect::<Option<Vec<_>>>()
            {
                if points.iter().all(|p| p[0] < 0.)
                    || points.iter().all(|p| p[0] > size[0])
                    || points.iter().all(|p| p[1] < 0.)
                    || points.iter().all(|p| p[1] > size[1])
                {
                    continue;
                }
                let shade = if points.len() == 3 {
                    let v = self.vertices(s);
                    let n = unit(cross(sub(v[1], v[0]), sub(v[2], v[0])));
                    0.55 + 0.45 * dot(n, unit([0.25, -0.45, 0.85])).abs()
                } else {
                    1.
                };
                out.push(Primitive {
                    item: s.item,
                    depth: points.iter().map(|p| p[2]).sum::<f64>() / points.len() as f64,
                    pixels: points.iter().map(|p| [p[0], p[1]]).collect(),
                    color: s.color.map(|c| (c as f64 * shade) as u8),
                });
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> Scene {
        let mut s = Scene::default();
        s.add_source(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/datasets/reference-mesh.obj"),
        )
        .unwrap();
        s
    }
    #[test]
    fn surface_coverage_restore_and_raw_unchanged() {
        let s = scene();
        let raw = s.items[0].geometry.positions().to_vec();
        let mut b = Blast::new(&s, false, 1.).unwrap();
        let Geometry::Mesh(m) = s.items[0].geometry.as_ref() else {
            panic!()
        };
        assert_eq!(b.count(), m.triangles().len() * 64);
        let area = |t: &[[f64; 3]]| {
            let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
            dot(n, n).sqrt() * 0.5
        };
        let expected: f64 = m
            .triangles()
            .iter()
            .map(|f| area(&f.map(|i| raw[i as usize])))
            .sum();
        let actual: f64 = b.shards.iter().map(|sh| area(&b.vertices(sh))).sum();
        assert!((expected - actual).abs() < 1e-10);
        let initial = b.vertices(&b.shards[0]);
        b.progress = 1.;
        assert_ne!(initial, b.vertices(&b.shards[0]));
        b.reverse = true;
        b.playing = true;
        for _ in 0..20 {
            b.tick(0.1);
        }
        assert_eq!(b.progress, 0.);
        assert_eq!(initial, b.vertices(&b.shards[0]));
        assert_eq!(raw, s.items[0].geometry.positions());
    }
    #[test]
    fn deterministic_radial_impulse_and_rigid_shards() {
        let s = scene();
        let mut a = Blast::new(&s, false, 1.).unwrap();
        let mut b = Blast::new(&s, false, 1.).unwrap();
        a.progress = 0.7;
        b.progress = 0.7;
        let mut signs = [[false; 2]; 3];
        for (sa, sb) in a.shards.iter().zip(&b.shards) {
            assert_eq!(a.vertices(sa), b.vertices(sb));
            let v = a.vertices(sa);
            let before = sub(sa.vertices[0], sa.vertices[1]);
            let after = sub(v[0], v[1]);
            assert!((dot(before, before) - dot(after, after)).abs() < 1e-10);
            for k in 0..3 {
                signs[k][usize::from(sa.velocity[k] > 0.)] = true;
            }
        }
        assert!(signs.iter().all(|s| s[0] && s[1]));
    }
    #[test]
    fn empty_and_unselected_fail() {
        assert!(Blast::new(&Scene::default(), false, 1.).is_err());
        assert!(Blast::new(&scene(), true, 1.).is_err());
    }
    #[test]
    fn cloud_colors_selection_and_hierarchy_are_preserved() {
        let mut s = scene();
        let group = s.add_assembly("Alvo".into()).unwrap();
        let id = s.items[0].id.clone();
        s.set_parent(&id, Some(group.clone())).unwrap();
        s.set_group_placement(
            &group,
            noah_workspace::Placement {
                translation: [3., 2., 1.],
                scale: 2.,
                ..Default::default()
            },
        )
        .unwrap();
        let mut other = s.items[0].clone();
        other.id = "outside".into();
        other.parent = None;
        s.items.push(other);
        s.selected = Some(group);
        let b = Blast::new(&s, true, 1.).unwrap();
        assert_eq!(b.targets, vec![0]);
        let t = s.world_transforms(true).unwrap()[0];
        let expected = t.apply(s.items[0].geometry.positions()[0]).unwrap();
        assert_eq!(b.shards[0].vertices[0], expected);
        s.items[0].visible = false;
        assert!(Blast::new(&s, true, 1.).is_err());
        let mut cloud = Scene::default();
        cloud
            .add_source(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/datasets/reference-rgb.ply"),
            )
            .unwrap();
        let b = Blast::new(&cloud, false, 1.).unwrap();
        let Geometry::Points(p) = cloud.items[0].geometry.as_ref() else {
            panic!()
        };
        assert_eq!(b.count(), p.positions().len());
        for (i, sh) in b.shards.iter().enumerate() {
            assert_eq!(sh.vertices, vec![p.positions()[i]]);
            assert_eq!(sh.color, p.colors().unwrap()[i]);
        }
    }
    #[test]
    fn budget_is_bounded_and_camera_fit_is_finite() {
        let mut s = scene();
        s.items[0].geometry = std::sync::Arc::new(Geometry::Mesh(
            noah_geometry::TriangleMesh::new(
                vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                vec![[0, 1, 2]; 40_001],
            )
            .unwrap(),
        ));
        let mut b = Blast::new(&s, false, 3.).unwrap();
        assert!(b.count() <= LIMIT);
        b.progress = 1.;
        let mut c = s.camera.clone();
        b.fit_camera(&mut c).unwrap();
        c.validate().unwrap();
        assert!(!b.project(&c, [800., 600.]).is_empty());
    }
}

#[cfg(test)]
mod detail_tests {
    use super::*;
    #[test]
    fn small_shards_cover_surface_and_travel_further_with_bounded_detail() {
        let mut s = Scene::default();
        s.add_source(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/datasets/inspection-box.obj"),
        )
        .unwrap();
        let coarse = Blast::with_detail(&s, false, 1., 2).unwrap();
        let mut fine = Blast::with_detail(&s, false, 1., 3).unwrap();
        assert_eq!(fine.count(), 768);
        assert_eq!(fine.count(), coarse.count() * 4);
        fine.progress = 1.;
        for shard in &fine.shards {
            let v = fine.vertices(shard);
            let centroid = mul(v.iter().copied().fold([0.; 3], add), 1. / v.len() as f64);
            assert!(
                dot(sub(centroid, shard.center), sub(centroid, shard.center)).sqrt()
                    > fine.radius * 1.5
            );
        }
        fine.follow_camera(&s.camera);
        assert!(fine.camera().unwrap().distance > s.camera.distance);
        assert!(Blast::with_detail(&s, false, f64::NAN, 3).is_err());
        assert!(Blast::with_detail(&s, false, 1., 5).is_err());
    }
}

#[cfg(test)]
mod panel_tests {
    use super::*;
    #[test]
    fn panels_use_original_faces_coherent_motion_and_exact_restore() {
        let mut s = Scene::default();
        s.add_source(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/datasets/inspection-box.obj"),
        )
        .unwrap();
        let before = s.items[0].geometry.positions().to_vec();
        let mut b = Blast::with_panels(&s, false, 1.).unwrap();
        let Geometry::Mesh(m) = s.items[0].geometry.as_ref() else {
            panic!()
        };
        assert_eq!(b.shards.len(), m.triangles().len());
        assert_eq!(b.count(), 6);
        assert!(b.shards.iter().all(|s| s.vertices.len() == 3));
        let original: Vec<_> = b.shards.iter().map(|s| b.vertices(s)).collect();
        b.progress = 1.;
        assert!(
            b.shards
                .iter()
                .zip(&original)
                .any(|(s, v)| b.vertices(s) != *v)
        );
        b.progress = 0.;
        for (s, v) in b.shards.iter().zip(original) {
            assert_eq!(b.vertices(s), v);
        }
        assert_eq!(s.items[0].geometry.positions(), before);
    }
}
