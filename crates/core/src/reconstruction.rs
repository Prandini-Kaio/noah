//! Contratos neutros de reconstrução, sem dependência do motor.
use noah_geometry::PointCloud;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub id: u64,
    pub model: String,
    pub width: u32,
    pub height: u32,
    pub parameters: Vec<f64>,
}
/// Pose world→camera: quaternion wxyz unitário e translação.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraView {
    pub id: u64,
    pub camera_id: u64,
    pub image_name: String,
    pub rotation_wxyz: [f64; 4],
    pub translation: [f64; 3],
}
/// Referencial destro arbitrário, unidade desconhecida; vertical não declarada.
#[derive(Debug, Clone, PartialEq)]
pub struct Reconstruction {
    pub cameras: Vec<Camera>,
    pub views: Vec<CameraView>,
    pub point_ids: Vec<u64>,
    pub cloud: PointCloud,
}
