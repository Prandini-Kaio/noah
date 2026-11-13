//! Exact local-coordinate point selection and neighbourhood filters; no I/O.
use crate::PointCloud;
use std::{cmp::Ordering, collections::BinaryHeap};
pub const MAX_CLEAN_POINTS: usize = 200_000;
pub type Result<T> = std::result::Result<T, &'static str>;
pub fn validate(cloud: &PointCloud, active: &[bool]) -> Result<()> {
    if cloud.positions().len() != active.len()
        || active.len() > MAX_CLEAN_POINTS
        || active.is_empty()
        || cloud.positions().iter().flatten().any(|v| v.abs() > 1e12)
    {
        return Err("limpeza aceita 1–200 mil pontos finitos no alcance +/-1e12");
    }
    Ok(())
}
pub fn subset(cloud: &PointCloud, indices: &[usize]) -> Result<PointCloud> {
    if indices.is_empty()
        || indices.windows(2).any(|w| w[0] >= w[1])
        || indices
            .last()
            .is_some_and(|&i| i >= cloud.positions().len())
    {
        return Err("revisão exige índices crescentes, únicos e não vazios");
    }
    PointCloud::new(
        indices.iter().map(|&i| cloud.positions()[i]).collect(),
        cloud
            .colors()
            .map(|c| indices.iter().map(|&i| c[i]).collect()),
    )
    .map_err(|_| "nuvem inválida")
}
pub fn box_selection(
    cloud: &PointCloud,
    active: &[bool],
    min: [f64; 3],
    max: [f64; 3],
) -> Result<Vec<bool>> {
    validate(cloud, active)?;
    if min.iter().chain(max.iter()).any(|v| !v.is_finite()) || (0..3).any(|a| min[a] > max[a]) {
        return Err("caixa XYZ inválida");
    }
    Ok(cloud
        .positions()
        .iter()
        .zip(active)
        .map(|(p, &a)| a && (0..3).all(|axis| p[axis] >= min[axis] && p[axis] <= max[axis]))
        .collect())
}
#[derive(Clone, Copy, PartialEq)]
struct Neighbour(f64, usize);
impl Eq for Neighbour {}
impl Ord for Neighbour {
    fn cmp(&self, o: &Self) -> Ordering {
        self.0.total_cmp(&o.0).then(self.1.cmp(&o.1))
    }
}
impl PartialOrd for Neighbour {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
struct Node {
    index: usize,
    axis: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
impl Node {
    fn build(ids: &mut [usize], points: &[[f64; 3]], depth: usize) -> Option<Box<Self>> {
        if ids.is_empty() {
            return None;
        }
        let axis = depth % 3;
        let mid = ids.len() / 2;
        ids.select_nth_unstable_by(mid, |&a, &b| {
            points[a][axis].total_cmp(&points[b][axis]).then(a.cmp(&b))
        });
        let (left, rest) = ids.split_at_mut(mid);
        let (pivot, right) = rest.split_first_mut()?;
        Some(Box::new(Self {
            index: *pivot,
            axis,
            left: Self::build(left, points, depth + 1),
            right: Self::build(right, points, depth + 1),
        }))
    }
    fn query(
        &self,
        points: &[[f64; 3]],
        index: usize,
        k: usize,
        radius2: f64,
        heap: &mut BinaryHeap<Neighbour>,
    ) {
        let p = points[index];
        let q = points[self.index];
        let delta = p[self.axis] - q[self.axis];
        let distance = (0..3).map(|a| (p[a] - q[a]).powi(2)).sum::<f64>();
        if self.index != index && distance <= radius2 {
            heap.push(Neighbour(distance, self.index));
            if heap.len() > k {
                heap.pop();
            }
        }
        let (near, far) = if delta <= 0. {
            (&self.left, &self.right)
        } else {
            (&self.right, &self.left)
        };
        if let Some(n) = near {
            n.query(points, index, k, radius2, heap);
        }
        let limit = if heap.len() == k {
            heap.peek().map(|n| n.0).unwrap_or(radius2).min(radius2)
        } else {
            radius2
        };
        if delta * delta <= limit
            && (heap.len() < k || delta * delta < limit)
            && let Some(n) = far
        {
            n.query(points, index, k, radius2, heap);
        }
    }
}
/// Returns candidate removals among active points. k excludes the query point.
pub fn statistical(cloud: &PointCloud, active: &[bool], k: usize, sigma: f64) -> Result<Vec<bool>> {
    validate(cloud, active)?;
    let mut ids: Vec<_> = active
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| v.then_some(i))
        .collect();
    if k == 0 || k > 64 || ids.len() <= k || !sigma.is_finite() || sigma <= 0. || sigma > 10. {
        return Err("SOR exige 1–64 vizinhos, mais pontos que k e sigma em (0,10]");
    }
    let tree = Node::build(&mut ids, cloud.positions(), 0).ok_or("nuvem vazia")?;
    let mut means = vec![0.; active.len()];
    let mut heap = BinaryHeap::with_capacity(k + 1);
    for &i in &ids {
        heap.clear();
        tree.query(cloud.positions(), i, k, f64::INFINITY, &mut heap);
        means[i] = heap.iter().map(|n| n.0.sqrt()).sum::<f64>() / k as f64;
    }
    let mean = ids.iter().map(|&i| means[i]).sum::<f64>() / ids.len() as f64;
    let sd =
        (ids.iter().map(|&i| (means[i] - mean).powi(2)).sum::<f64>() / ids.len() as f64).sqrt();
    Ok(active
        .iter()
        .enumerate()
        .map(|(i, &a)| a && means[i] > mean + sigma * sd)
        .collect())
}
pub fn radius(
    cloud: &PointCloud,
    active: &[bool],
    radius: f64,
    min_neighbours: usize,
) -> Result<Vec<bool>> {
    validate(cloud, active)?;
    if !radius.is_finite()
        || radius <= 0.
        || radius > 1e12
        || min_neighbours == 0
        || min_neighbours > 64
    {
        return Err("raio deve ser positivo e vizinhos entre 1–64");
    }
    let mut ids: Vec<_> = active
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| v.then_some(i))
        .collect();
    let tree = Node::build(&mut ids, cloud.positions(), 0).ok_or("nuvem ativa vazia")?;
    let mut result = vec![false; active.len()];
    let mut heap = BinaryHeap::with_capacity(min_neighbours + 1);
    for &i in &ids {
        heap.clear();
        tree.query(
            cloud.positions(),
            i,
            min_neighbours,
            radius * radius,
            &mut heap,
        );
        result[i] = heap.len() < min_neighbours;
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_remove_known_outlier_preserve_color_and_local_box() {
        let cloud = PointCloud::new(
            vec![
                [0., 0., 0.],
                [0.1, 0., 0.],
                [0., 0.1, 0.],
                [0.1, 0.1, 0.],
                [99., 99., 99.],
            ],
            Some(vec![[10, 20, 30]; 5]),
        )
        .unwrap();
        let active = vec![true; 5];
        assert_eq!(
            statistical(&cloud, &active, 2, 1.).unwrap(),
            vec![false, false, false, false, true]
        );
        assert_eq!(
            radius(&cloud, &active, 0.2, 2).unwrap(),
            vec![false, false, false, false, true]
        );
        let b = box_selection(&cloud, &active, [-1.; 3], [1.; 3]).unwrap();
        assert_eq!(b, vec![true, true, true, true, false]);
        let c = subset(&cloud, &[0, 2, 3]).unwrap();
        assert_eq!(c.colors().unwrap(), &[[10, 20, 30]; 3]);
        assert_eq!(c.positions()[1], cloud.positions()[2]);
        assert!(statistical(&cloud, &active, 5, 1.).is_err());
        assert!(radius(&cloud, &active, f64::NAN, 1).is_err());
        assert!(subset(&cloud, &[2, 1]).is_err());
        assert!(box_selection(&cloud, &active, [1.; 3], [0.; 3]).is_err());
    }
    #[test]
    fn kd_tree_matches_bruteforce_with_duplicates_and_inactive_points() {
        let mut p: Vec<_> = (0..137)
            .map(|i| {
                [
                    ((i * 17) % 41) as f64,
                    ((i * 31) % 29) as f64,
                    ((i * 3) % 13) as f64,
                ]
            })
            .collect();
        p.push(p[0]);
        let c = PointCloud::new(p, None).unwrap();
        let active: Vec<_> = (0..138).map(|i| i % 7 != 3).collect();
        let mut ids: Vec<_> = (0..138).filter(|&i| active[i]).collect();
        let t = Node::build(&mut ids, c.positions(), 0).unwrap();
        for &i in &ids {
            let mut h = BinaryHeap::new();
            t.query(c.positions(), i, 7, f64::INFINITY, &mut h);
            let mut expected: Vec<_> = ids
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| {
                    Neighbour(
                        (0..3)
                            .map(|a| (c.positions()[i][a] - c.positions()[j][a]).powi(2))
                            .sum(),
                        j,
                    )
                })
                .collect();
            expected.sort();
            assert_eq!(
                h.into_sorted_vec().iter().map(|n| n.0).collect::<Vec<_>>(),
                expected[..7].iter().map(|n| n.0).collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(test)]
mod degenerate_tests {
    use super::*;
    #[test]
    fn coincident_points_are_neighbours_not_outliers() {
        let c = PointCloud::new(vec![[1., 2., 3.]; 2000], None).unwrap();
        let active = vec![true; 2000];
        assert!(
            statistical(&c, &active, 16, 2.)
                .unwrap()
                .iter()
                .all(|&v| !v)
        );
        assert!(radius(&c, &active, 0.1, 16).unwrap().iter().all(|&v| !v));
    }
}
