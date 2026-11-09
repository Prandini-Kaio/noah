//! Subconjunto PLY ASCII deliberadamente restrito; sem I/O no domínio.
use crate::{MAX_POINTS, Result, fail};
use noah_geometry::PointCloud;

pub fn parse(bytes: &[u8]) -> Result<PointCloud> {
    if bytes.len() as u64 > crate::MAX_PLY_BYTES {
        return fail("PLY excede 64 MiB");
    }
    let text = std::str::from_utf8(bytes)?;
    if text.lines().any(|line| line.len() > 4096) {
        return fail("linha PLY excede 4096 bytes");
    }
    let mut lines = text.lines();
    if lines.next() != Some("ply") || lines.next() != Some("format ascii 1.0") {
        return fail("PLY exige magic ply e format ascii 1.0");
    }
    let mut count = None;
    let mut properties = Vec::new();
    let mut header_len = 0;
    loop {
        let line = lines.next().ok_or("PLY sem end_header")?;
        header_len += line.len() + 1;
        if header_len > 65536 {
            return fail("header PLY excede 64 KiB");
        }
        if line == "end_header" {
            break;
        }
        if line.starts_with("comment ") || line == "comment" {
            continue;
        }
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            ["element", "vertex", n] if count.is_none() && properties.is_empty() => {
                let n: usize = n.parse()?;
                if n > MAX_POINTS {
                    return fail("PLY excede limite de pontos");
                }
                count = Some(n);
            }
            ["property", kind, name] if count.is_some() => properties.push((*kind, *name)),
            _ => return fail("elemento/propriedade PLY não suportado"),
        }
    }
    let count = count.ok_or("PLY sem element vertex")?;
    let xyz = properties.len() >= 3
        && properties[..3]
            .iter()
            .zip(["x", "y", "z"])
            .all(|((kind, name), expected)| {
                matches!(*kind, "float" | "double") && *name == expected
            });
    let rgb = properties.len() == 6
        && properties[3..] == [("uchar", "red"), ("uchar", "green"), ("uchar", "blue")];
    if !xyz || !(properties.len() == 3 || rgb) {
        return fail("PLY suporta apenas XYZ float/double e RGB uchar opcional, nessa ordem");
    }
    let mut points = Vec::with_capacity(count);
    let mut colors = rgb.then(|| Vec::with_capacity(count));
    for _ in 0..count {
        let fields: Vec<_> = lines
            .next()
            .ok_or("PLY truncado")?
            .split_whitespace()
            .collect();
        if fields.len() != properties.len() {
            return fail("quantidade inválida de campos PLY");
        }
        points.push([fields[0].parse()?, fields[1].parse()?, fields[2].parse()?]);
        if let Some(colors) = &mut colors {
            colors.push([fields[3].parse()?, fields[4].parse()?, fields[5].parse()?]);
        }
    }
    if lines.any(|line| !line.trim().is_empty()) {
        return fail("dados excedentes após vértices PLY");
    }
    Ok(PointCloud::new(points, colors)?)
}

pub fn encode(cloud: &PointCloud) -> Vec<u8> {
    let mut text = format!(
        "ply\nformat ascii 1.0\nelement vertex {}\nproperty double x\nproperty double y\nproperty double z\n",
        cloud.positions().len()
    );
    if cloud.colors().is_some() {
        text.push_str("property uchar red\nproperty uchar green\nproperty uchar blue\n");
    }
    text.push_str("end_header\n");
    for (i, [x, y, z]) in cloud.positions().iter().enumerate() {
        text.push_str(&format!("{x} {y} {z}"));
        if let Some(colors) = cloud.colors() {
            let [r, g, b] = colors[i];
            text.push_str(&format!(" {r} {g} {b}"));
        }
        text.push('\n');
    }
    text.into_bytes()
}
