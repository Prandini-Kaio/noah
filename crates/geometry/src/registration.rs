//! Positive similarity registration from paired XYZ coordinates; no sensor/I/O.
use crate::{InvalidGeometry, Transform};
pub struct Fit {
    pub transform: Transform,
    pub rmse: f64,
    pub max_residual: f64,
    pub reference_radius: f64,
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn spread(points: &[[f64; 3]], center: [f64; 3]) -> bool {
    let vectors: Vec<_> = points.iter().map(|&p| sub(p, center)).collect();
    let Some(a) = vectors
        .iter()
        .max_by(|&&a, &&b| dot(a, a).total_cmp(&dot(b, b)))
    else {
        return false;
    };
    let n = dot(*a, *a);
    n > 1e-20
        && vectors.iter().any(|&b| {
            let c = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            dot(c, c) > n * n * 1e-10
        })
}
/// Least-squares Horn fit. At least three distinct non-collinear pairs; <=500 pairs.
pub fn similarity(source: &[[f64; 3]], target: &[[f64; 3]]) -> Result<Fit, InvalidGeometry> {
    if source.len() != target.len()
        || !(3..=500).contains(&source.len())
        || source
            .iter()
            .chain(target)
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1e12)
    {
        return Err(InvalidGeometry);
    }
    let center = |p: &[[f64; 3]]| {
        std::array::from_fn(|a| p.iter().map(|v| v[a]).sum::<f64>() / p.len() as f64)
    };
    let cs = center(source);
    let ct = center(target);
    if !spread(source, cs) || !spread(target, ct) {
        return Err(InvalidGeometry);
    }
    let mut h = [[0.; 3]; 3];
    let mut variance = 0.;
    let mut target_variance = 0.;
    for (&s, &t) in source.iter().zip(target) {
        let a = sub(s, cs);
        let b = sub(t, ct);
        variance += dot(a, a);
        target_variance += dot(b, b);
        for i in 0..3 {
            for j in 0..3 {
                h[i][j] += a[i] * b[j];
            }
        }
    }
    let trace = h[0][0] + h[1][1] + h[2][2];
    let mut n = [
        [
            trace,
            h[1][2] - h[2][1],
            h[2][0] - h[0][2],
            h[0][1] - h[1][0],
        ],
        [
            h[1][2] - h[2][1],
            h[0][0] - h[1][1] - h[2][2],
            h[0][1] + h[1][0],
            h[0][2] + h[2][0],
        ],
        [
            h[2][0] - h[0][2],
            h[0][1] + h[1][0],
            -h[0][0] + h[1][1] - h[2][2],
            h[1][2] + h[2][1],
        ],
        [
            h[0][1] - h[1][0],
            h[0][2] + h[2][0],
            h[1][2] + h[2][1],
            -h[0][0] - h[1][1] + h[2][2],
        ],
    ];
    let norm = n.iter().flatten().map(|v| v.abs()).fold(0., f64::max);
    if norm <= 0. {
        return Err(InvalidGeometry);
    }
    for row in &mut n {
        for v in row {
            *v /= norm;
        }
    }
    let mut eigen = [[0.; 4]; 4];
    for (i, row) in eigen.iter_mut().enumerate() {
        row[i] = 1.;
    }
    for _ in 0..100 {
        let mut pair = (0, 1);
        for i in 0..4 {
            for j in i + 1..4 {
                if n[i][j].abs() > n[pair.0][pair.1].abs() {
                    pair = (i, j);
                }
            }
        }
        let (p, q) = pair;
        if n[p][q].abs() < 1e-14 {
            break;
        }
        let angle = 0.5 * (2. * n[p][q]).atan2(n[q][q] - n[p][p]);
        let (c, s) = (angle.cos(), angle.sin());
        let (pp, qq, pq) = (n[p][p], n[q][q], n[p][q]);
        n[p][p] = c * c * pp - 2. * s * c * pq + s * s * qq;
        n[q][q] = s * s * pp + 2. * s * c * pq + c * c * qq;
        n[p][q] = 0.;
        n[q][p] = 0.;
        for k in 0..4 {
            if k != p && k != q {
                let (a, b) = (n[k][p], n[k][q]);
                n[k][p] = c * a - s * b;
                n[p][k] = n[k][p];
                n[k][q] = s * a + c * b;
                n[q][k] = n[k][q];
            }
            let (a, b) = (eigen[k][p], eigen[k][q]);
            eigen[k][p] = c * a - s * b;
            eigen[k][q] = s * a + c * b;
        }
    }
    let index = (0..4)
        .max_by(|&a, &b| n[a][a].total_cmp(&n[b][b]))
        .ok_or(InvalidGeometry)?;
    let [w, x, y, z] = std::array::from_fn(|i| eigen[i][index]);
    let r = [
        [
            1. - 2. * (y * y + z * z),
            2. * (x * y - z * w),
            2. * (x * z + y * w),
        ],
        [
            2. * (x * y + z * w),
            1. - 2. * (x * x + z * z),
            2. * (y * z - x * w),
        ],
        [
            2. * (x * z - y * w),
            2. * (y * z + x * w),
            1. - 2. * (x * x + y * y),
        ],
    ];
    let rotate = |p: [f64; 3]| std::array::from_fn(|a| dot(r[a], p));
    let scale = source
        .iter()
        .zip(target)
        .map(|(&s, &t)| dot(rotate(sub(s, cs)), sub(t, ct)))
        .sum::<f64>()
        / variance;
    if !scale.is_finite() || !(1e-9..=1e9).contains(&scale) {
        return Err(InvalidGeometry);
    }
    let rc = rotate(cs);
    let t = Transform::from_parts(std::array::from_fn(|a| ct[a] - scale * rc[a]), r, scale)?;
    let mut sum = 0.;
    let mut max: f64 = 0.;
    for (&s, &target) in source.iter().zip(target) {
        let d = sub(t.apply(s)?, target);
        let d2 = dot(d, d);
        sum += d2;
        max = max.max(d2.sqrt());
    }
    Ok(Fit {
        transform: t,
        rmse: (sum / source.len() as f64).sqrt(),
        max_residual: max,
        reference_radius: (target_variance / target.len() as f64).sqrt(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovers_rotation_scale_translation_planar_and_rejects_degenerate() {
        let p = vec![
            [0., 0., 0.],
            [2., 0., 0.],
            [0., 1., 0.],
            [2., 1., 0.],
            [0.3, 0.7, 0.],
        ];
        let known = Transform::from_euler([8., -3., 2.], [0.4, -0.3, 1.1], 2.5).unwrap();
        let target: Vec<_> = p.iter().map(|&p| known.apply(p).unwrap()).collect();
        let fit = similarity(&p, &target).unwrap();
        assert!(fit.rmse < 1e-10);
        assert!((fit.transform.scale() - 2.5).abs() < 1e-10);
        for (&p, &t) in p.iter().zip(&target) {
            let got = fit.transform.apply(p).unwrap();
            assert!((0..3).all(|a| (got[a] - t[a]).abs() < 1e-10));
        }
        assert!(similarity(&[[0.; 3]; 4], &[[0.; 3]; 4]).is_err());
        assert!(
            similarity(
                &[[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]],
                &[[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]]
            )
            .is_err()
        );
    }
    #[test]
    fn noisy_pairs_have_reported_nonzero_residual() {
        let p = vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        let mut q = p.clone();
        q[3][0] += 0.1;
        let fit = similarity(&p, &q).unwrap();
        assert!(fit.rmse > 0.001 && fit.rmse < 0.1);
        assert!(fit.max_residual >= fit.rmse);
    }
}
