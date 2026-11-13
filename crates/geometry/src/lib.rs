//! Geometria em f64, referencial destro, vetores coluna e unidade explícita.
use std::{error::Error, fmt};

/// Unidade declarada pelo usuário; nunca inferida de coordenadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnitState {
    #[default]
    Unknown,
    Metre,
    Millimetre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGeometry;
impl fmt::Display for InvalidGeometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("coordenadas devem ser finitas e escala deve ser positiva")
    }
}
impl Error for InvalidGeometry {}

/// Nuvem imutável: cores, quando presentes, correspondem a todos os pontos.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloud {
    positions: Vec<[f64; 3]>,
    colors: Option<Vec<[u8; 3]>>,
}
impl PointCloud {
    pub fn new(
        positions: Vec<[f64; 3]>,
        colors: Option<Vec<[u8; 3]>>,
    ) -> Result<Self, InvalidGeometry> {
        if positions.iter().flatten().any(|v| !v.is_finite())
            || colors.as_ref().is_some_and(|c| c.len() != positions.len())
        {
            return Err(InvalidGeometry);
        }
        Ok(Self { positions, colors })
    }
    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }
    pub fn colors(&self) -> Option<&[[u8; 3]]> {
        self.colors.as_deref()
    }
}

/// Instância: escala uniforme positiva, rotação 3D, translação.
/// Ordem de aplicação: escala → rotação → translação. Não modifica a nuvem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    translation: [f64; 3],
    rotation: [[f64; 3]; 3],
    scale: f64,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            rotation: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            scale: 1.,
        }
    }
}
impl Transform {
    pub fn new(
        translation: [f64; 3],
        rotation_z: f64,
        scale: f64,
    ) -> Result<Self, InvalidGeometry> {
        Self::from_euler(translation, [0., 0., rotation_z], scale)
    }
    /// Euler XYZ em radianos: Rz * Ry * Rx.
    pub fn from_euler(
        translation: [f64; 3],
        angles: [f64; 3],
        scale: f64,
    ) -> Result<Self, InvalidGeometry> {
        if angles.iter().any(|v| !v.is_finite()) {
            return Err(InvalidGeometry);
        }
        let (sx, cx) = angles[0].sin_cos();
        let (sy, cy) = angles[1].sin_cos();
        let (sz, cz) = angles[2].sin_cos();
        Self::from_parts(
            translation,
            [
                [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
                [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
                [-sy, cy * sx, cy * cx],
            ],
            scale,
        )
    }
    /// Reconstrói uma rotação própria; tolerância de ortonormalidade 1e-10.
    pub fn from_parts(
        translation: [f64; 3],
        rotation: [[f64; 3]; 3],
        scale: f64,
    ) -> Result<Self, InvalidGeometry> {
        if translation
            .iter()
            .chain(rotation.iter().flatten())
            .any(|v| !v.is_finite())
            || !scale.is_finite()
            || scale <= 0.
        {
            return Err(InvalidGeometry);
        }
        for i in 0..3 {
            for j in 0..3 {
                let dot: f64 = (0..3).map(|k| rotation[i][k] * rotation[j][k]).sum();
                if (dot - if i == j { 1. } else { 0. }).abs() > 1e-10 {
                    return Err(InvalidGeometry);
                }
            }
        }
        let r = rotation;
        let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
        if (det - 1.).abs() > 1e-10 {
            return Err(InvalidGeometry);
        }
        Ok(Self {
            translation,
            rotation,
            scale,
        })
    }
    pub fn translation(&self) -> [f64; 3] {
        self.translation
    }
    pub fn rotation(&self) -> [[f64; 3]; 3] {
        self.rotation
    }
    pub fn scale(&self) -> f64 {
        self.scale
    }
    /// Aplica rhs primeiro, self depois, sem alterar os operandos.
    pub fn compose(&self, rhs: &Self) -> Result<Self, InvalidGeometry> {
        let mut rotation = [[0.; 3]; 3];
        for (i, row) in rotation.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                *value = (0..3)
                    .map(|k| self.rotation[i][k] * rhs.rotation[k][j])
                    .sum();
            }
        }
        Self::from_parts(
            self.apply(rhs.translation)?,
            rotation,
            self.scale * rhs.scale,
        )
    }
    /// Retorna erro também se uma operação finita produzir overflow.
    pub fn apply(&self, point: [f64; 3]) -> Result<[f64; 3], InvalidGeometry> {
        if point.iter().any(|v| !v.is_finite()) {
            return Err(InvalidGeometry);
        }
        let scaled = point.map(|v| v * self.scale);
        let result = std::array::from_fn(|i| {
            self.rotation[i]
                .iter()
                .zip(scaled)
                .map(|(a, b)| a * b)
                .sum::<f64>()
                + self.translation[i]
        });
        if result.iter().any(|v| !v.is_finite()) {
            return Err(InvalidGeometry);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_and_known_transform_preserve_source() {
        let cloud = PointCloud::new(vec![[1., 0., 2.]], Some(vec![[255, 0, 0]])).unwrap();
        assert_eq!(
            Transform::default().apply(cloud.positions()[0]).unwrap(),
            [1., 0., 2.]
        );
        let t = Transform::new([3., 4., 5.], std::f64::consts::FRAC_PI_2, 2.).unwrap();
        for (actual, expected) in t
            .apply(cloud.positions()[0])
            .unwrap()
            .iter()
            .zip([3., 6., 9.])
        {
            assert!((actual - expected).abs() < 1e-12);
        }
        assert_eq!(cloud.positions(), &[[1., 0., 2.]]);
        assert_eq!(cloud.colors(), Some([[255, 0, 0]].as_slice()));
    }
    #[test]
    fn invalid_values_and_incomplete_colors_fail() {
        assert!(PointCloud::new(vec![[f64::NAN, 0., 0.]], None).is_err());
        assert!(PointCloud::new(vec![[0.; 3]], Some(vec![])).is_err());
        for scale in [0., -1., f64::INFINITY] {
            assert!(Transform::new([0.; 3], 0., scale).is_err());
        }
        assert!(
            Transform::new([0.; 3], 0., 2.)
                .unwrap()
                .apply([f64::MAX; 3])
                .is_err()
        );
        assert!(Transform::default().apply([f64::INFINITY; 3]).is_err());
        assert!(PointCloud::new(vec![], None).is_ok());
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;
    #[test]
    fn axes_composition_and_invalid_matrices() {
        let q = std::f64::consts::FRAC_PI_2;
        let x = Transform::from_euler([0.; 3], [q, 0., 0.], 1.).unwrap();
        let y = Transform::from_euler([1., 2., 3.], [0., q, 0.], 2.).unwrap();
        let p = x.apply([0., 1., 0.]).unwrap();
        assert!((p[2] - 1.).abs() < 1e-12 && p[1].abs() < 1e-12);
        let expected = y.apply(x.apply([1., 2., 3.]).unwrap()).unwrap();
        let result = y.compose(&x).unwrap().apply([1., 2., 3.]).unwrap();
        for (a, b) in result.into_iter().zip(expected) {
            assert!((a - b).abs() < 1e-12);
        }
        assert!(
            Transform::from_parts([0.; 3], [[-1., 0., 0.], [0., 1., 0.], [0., 0., 1.]], 1.)
                .is_err()
        );
        assert!(Transform::from_euler([0.; 3], [f64::NAN, 0., 0.], 1.).is_err());
    }
}

/// Malha de triângulos validada, sem topologia/normal/material nesta etapa.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleMesh {
    positions: Vec<[f64; 3]>,
    triangles: Vec<[u32; 3]>,
}
impl TriangleMesh {
    pub fn new(
        positions: Vec<[f64; 3]>,
        triangles: Vec<[u32; 3]>,
    ) -> Result<Self, InvalidGeometry> {
        if positions.iter().flatten().any(|v| !v.is_finite())
            || triangles.iter().any(|face| {
                face.iter().any(|&i| i as usize >= positions.len())
                    || face[0] == face[1]
                    || face[1] == face[2]
                    || face[0] == face[2]
            })
        {
            return Err(InvalidGeometry);
        }
        Ok(Self {
            positions,
            triangles,
        })
    }
    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
}

pub mod cleaning;
