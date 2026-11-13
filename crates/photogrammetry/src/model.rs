//! Tradução limitada do modelo TXT COLMAP 3.11 para contratos internos.
use noah_core::reconstruction::{Camera, CameraView, Reconstruction};
use noah_geometry::PointCloud;
use noah_pipeline::{Result, controlled, read_bounded};
use std::{collections::HashSet, path::Path};
fn text(root: &Path, name: &str) -> Result<String> {
    let text = String::from_utf8(read_bounded(&controlled(root, name)?, 256 * 1024 * 1024)?)?;
    if text.lines().any(|line| line.len() > 2 * 1024 * 1024) {
        return Err("linha de modelo excede limite de 2 MiB".into());
    }
    Ok(text)
}
fn finite(text: &str) -> Result<f64> {
    let v: f64 = text.parse()?;
    if !v.is_finite() {
        return Err("valor não finito no modelo".into());
    }
    Ok(v)
}
fn positive(text: &str) -> Result<u64> {
    let id = text.parse()?;
    if id == 0 {
        return Err("ID zero no modelo".into());
    }
    Ok(id)
}
pub fn parse(root: &Path) -> Result<Reconstruction> {
    let mut cameras = Vec::new();
    let mut camera_ids = HashSet::new();
    for line in text(root, "cameras.txt")?
        .lines()
        .filter(|s| !s.starts_with('#') && !s.trim().is_empty())
    {
        if cameras.len() >= 500 {
            return Err("limite de câmeras excedido".into());
        }
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() < 5 {
            return Err("câmera truncada".into());
        }
        let expected = match f[1] {
            "SIMPLE_PINHOLE" => 3,
            "PINHOLE" => 4,
            "SIMPLE_RADIAL" => 4,
            "RADIAL" => 5,
            "OPENCV" | "OPENCV_FISHEYE" => 8,
            "FULL_OPENCV" => 12,
            "FOV" => 5,
            "SIMPLE_RADIAL_FISHEYE" => 4,
            "RADIAL_FISHEYE" => 5,
            "THIN_PRISM_FISHEYE" => 12,
            _ => return Err("modelo intrínseco não suportado".into()),
        };
        if f.len() != 4 + expected {
            return Err("parâmetros de câmera incompatíveis".into());
        }
        let id = positive(f[0])?;
        let width: u32 = f[2].parse()?;
        let height: u32 = f[3].parse()?;
        if !camera_ids.insert(id) || width == 0 || height == 0 || width > 100000 || height > 100000
        {
            return Err("ID/dimensões de câmera inválidos".into());
        }
        let parameters = f[4..]
            .iter()
            .map(|v| finite(v))
            .collect::<Result<Vec<_>>>()?;
        if parameters[0] <= 0.
            || (matches!(
                f[1],
                "PINHOLE"
                    | "OPENCV"
                    | "OPENCV_FISHEYE"
                    | "FULL_OPENCV"
                    | "FOV"
                    | "THIN_PRISM_FISHEYE"
            ) && parameters[1] <= 0.)
        {
            return Err("focal deve ser positiva".into());
        }
        cameras.push(Camera {
            id,
            model: f[1].into(),
            width,
            height,
            parameters,
        });
    }
    let mut track_views = HashSet::new();
    let mut points = Vec::new();
    let mut colors = Vec::new();
    let mut point_ids = Vec::new();
    let mut ids = HashSet::new();
    for line in text(root, "points3D.txt")?
        .lines()
        .filter(|s| !s.starts_with('#') && !s.trim().is_empty())
    {
        if points.len() >= 1_000_000 {
            return Err("limite de pontos excedido".into());
        }
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() < 8 || !(f.len() - 8).is_multiple_of(2) {
            return Err("ponto 3D truncado".into());
        }
        let id = positive(f[0])?;
        if !ids.insert(id) {
            return Err("ID de ponto duplicado".into());
        }
        if finite(f[7])? < 0. {
            return Err("erro de reprojeção negativo".into());
        }
        for pair in f[8..].as_chunks::<2>().0 {
            track_views.insert(positive(pair[0])?);
            pair[1].parse::<u64>()?;
        }
        points.push([finite(f[1])?, finite(f[2])?, finite(f[3])?]);
        colors.push([f[4].parse()?, f[5].parse()?, f[6].parse()?]);
        point_ids.push(id);
    }
    let mut views = Vec::new();
    let mut view_ids = HashSet::new();
    let mut names = HashSet::new();
    let images = text(root, "images.txt")?;
    let mut lines = images.lines();
    while let Some(line) = lines.next() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if views.len() >= 500 {
            return Err("limite de vistas excedido".into());
        }
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() != 10 {
            return Err("pose de imagem inválida".into());
        }
        let id = positive(f[0])?;
        let camera_id = positive(f[8])?;
        if !view_ids.insert(id)
            || !camera_ids.contains(&camera_id)
            || !names.insert(f[9].to_owned())
        {
            return Err("referência/identidade de vista inválida".into());
        }
        if Path::new(f[9]).components().count() != 1
            || f[9].contains(['/', '\\'])
            || matches!(f[9], "." | "..")
        {
            return Err("nome de imagem fora do snapshot".into());
        }
        let q = [finite(f[1])?, finite(f[2])?, finite(f[3])?, finite(f[4])?];
        if (q.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 1e-6 {
            return Err("quaternion de câmera não unitário".into());
        }
        let observations = lines.next().ok_or("observações de imagem ausentes")?;
        let fields: Vec<_> = observations.split_whitespace().collect();
        if !fields.len().is_multiple_of(3) {
            return Err("observações truncadas".into());
        }
        for triple in fields.as_chunks::<3>().0 {
            finite(triple[0])?;
            finite(triple[1])?;
            if triple[2] != "-1" && !ids.contains(&positive(triple[2])?) {
                return Err("observação referencia ponto ausente".into());
            }
        }
        views.push(CameraView {
            id,
            camera_id,
            image_name: f[9].into(),
            rotation_wxyz: q,
            translation: [finite(f[5])?, finite(f[6])?, finite(f[7])?],
        });
    }
    if !track_views.is_subset(&view_ids) {
        return Err("track referencia imagem ausente".into());
    }
    if views.len() < 2 || points.is_empty() {
        return Err("reconstrução vazia ou menos de duas imagens registradas".into());
    }
    Ok(Reconstruction {
        cameras,
        views,
        point_ids,
        cloud: PointCloud::new(points, Some(colors))?,
    })
}

/// Filter COLMAP TXT while preserving point IDs, image observation indices and tracks.
/// Removed observations become -1; cameras/poses and retained tracks stay unchanged.
pub fn filter_points(root: &Path, kept_ids: &std::collections::HashSet<u64>) -> Result<()> {
    let reconstruction = parse(root)?;
    let all_ids: std::collections::HashSet<_> = reconstruction.point_ids.iter().copied().collect();
    if kept_ids.is_empty() || !kept_ids.is_subset(&all_ids) {
        return Err("seleção de pontos não pertence ao modelo".into());
    }
    let points = text(root, "points3D.txt")?;
    let mut out = String::new();
    for line in points.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let id = line
            .split_whitespace()
            .next()
            .ok_or("ponto ausente")?
            .parse()?;
        if kept_ids.contains(&id) {
            out.push_str(line);
            out.push('\n');
        }
    }
    let images = text(root, "images.txt")?;
    let mut lines = images.lines();
    let mut image_out = String::new();
    while let Some(line) = lines.next() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        image_out.push_str(line);
        image_out.push('\n');
        let observations = lines.next().ok_or("observações ausentes")?;
        let f: Vec<_> = observations.split_whitespace().collect();
        for t in f.as_chunks::<3>().0 {
            image_out.push_str(t[0]);
            image_out.push(' ');
            image_out.push_str(t[1]);
            image_out.push(' ');
            if t[2] != "-1" && kept_ids.contains(&t[2].parse()?) {
                image_out.push_str(t[2]);
            } else {
                image_out.push_str("-1");
            }
            image_out.push(' ');
        }
        image_out.push('\n');
    }
    // Only called for a new, unregistered model copy in a new job.
    std::fs::write(root.join("points3D.txt"), out)?;
    std::fs::write(root.join("images.txt"), image_out)?;
    let filtered = parse(root)?;
    if filtered.point_ids.len() != kept_ids.len() {
        return Err("modelo filtrado inconsistente".into());
    }
    Ok(())
}
