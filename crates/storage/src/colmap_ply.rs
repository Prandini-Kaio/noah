//! Bounded COLMAP PLY interchange: ASCII/little-endian XYZ, optional normals/RGB,
//! triangular faces. Original binary assets remain available alongside normalized OBJ.
use crate::{MAX_PLY_BYTES, MAX_POINTS, Result};
use noah_geometry::{PointCloud, TriangleMesh};
const MAX_FACES: usize = 100_000;
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    F32,
    F64,
    U8,
    U32,
    I32,
}
impl Kind {
    fn from(s: &str) -> Result<Self> {
        match s {
            "float" | "float32" => Ok(Self::F32),
            "double" | "float64" => Ok(Self::F64),
            "uchar" | "uint8" => Ok(Self::U8),
            "uint" | "uint32" => Ok(Self::U32),
            "int" | "int32" => Ok(Self::I32),
            _ => Err("tipo PLY COLMAP não suportado".into()),
        }
    }
}
enum Data<'a> {
    Binary { bytes: &'a [u8], offset: usize },
    Ascii(std::str::SplitWhitespace<'a>),
}
impl Data<'_> {
    fn number(&mut self, k: Kind) -> Result<f64> {
        let v = match self {
            Self::Ascii(tokens) => {
                let s = tokens.next().ok_or("PLY truncado")?;
                match k {
                    Kind::F32 => s.parse::<f32>()? as f64,
                    Kind::F64 => s.parse()?,
                    Kind::U8 => s.parse::<u8>()? as f64,
                    Kind::U32 => s.parse::<u32>()? as f64,
                    Kind::I32 => s.parse::<i32>()? as f64,
                }
            }
            Self::Binary { bytes, offset } => {
                let n = match k {
                    Kind::U8 => 1,
                    Kind::F64 => 8,
                    _ => 4,
                };
                let end = offset.checked_add(n).ok_or("offset PLY inválido")?;
                let s = bytes.get(*offset..end).ok_or("PLY binário truncado")?;
                *offset = end;
                match k {
                    Kind::U8 => s[0] as f64,
                    Kind::F32 => f32::from_le_bytes(s.try_into()?) as f64,
                    Kind::F64 => f64::from_le_bytes(s.try_into()?),
                    Kind::U32 => u32::from_le_bytes(s.try_into()?) as f64,
                    Kind::I32 => i32::from_le_bytes(s.try_into()?) as f64,
                }
            }
        };
        if !v.is_finite() {
            return Err("coordenada/normal PLY não finita".into());
        }
        Ok(v)
    }
    fn end(&mut self) -> bool {
        match self {
            Self::Binary { bytes, offset } => *offset == bytes.len(),
            Self::Ascii(tokens) => tokens.next().is_none(),
        }
    }
}
struct Parsed {
    positions: Vec<[f64; 3]>,
    colors: Option<Vec<[u8; 3]>>,
    faces: Vec<[u32; 3]>,
    normals: bool,
}
fn parse(bytes: &[u8]) -> Result<Parsed> {
    if bytes.len() as u64 > MAX_PLY_BYTES {
        return Err("PLY COLMAP excede 64 MiB".into());
    }
    let mut header = Vec::new();
    let mut end = 0;
    let mut done = false;
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        end += line.len();
        if end > 65536 || line.len() > 4096 {
            return Err("header PLY COLMAP excede limites".into());
        }
        let line = std::str::from_utf8(line)?.trim_end_matches(['\r', '\n']);
        header.push(line);
        if line == "end_header" {
            done = true;
            break;
        }
    }
    if !done || header.first() != Some(&"ply") {
        return Err("PLY COLMAP sem header válido".into());
    }
    let binary = match header.get(1) {
        Some(&"format binary_little_endian 1.0") => true,
        Some(&"format ascii 1.0") => false,
        _ => return Err("PLY exige ASCII ou binary_little_endian 1.0".into()),
    };
    let mut vertices = None;
    let mut face_count = None;
    let mut properties = Vec::new();
    let mut face_layout = None;
    let mut in_faces = false;
    for line in &header[2..header.len() - 1] {
        if line.starts_with("comment ") || *line == "comment" {
            continue;
        }
        let w: Vec<_> = line.split_whitespace().collect();
        match w.as_slice() {
            ["element", "vertex", n] if vertices.is_none() && !in_faces => {
                let n: usize = n.parse()?;
                if n == 0 || n > MAX_POINTS {
                    return Err("contagem de vértices PLY fora do limite 1..1000000".into());
                }
                vertices = Some(n);
            }
            ["property", k, name] if vertices.is_some() && !in_faces => {
                properties.push((Kind::from(k)?, *name));
                if properties.len() > 10 {
                    return Err("propriedades PLY excedentes".into());
                }
            }
            ["element", "face", n] if vertices.is_some() && !in_faces => {
                let n: usize = n.parse()?;
                if n > MAX_FACES {
                    return Err("malha excede limite de 100 mil faces desta prévia".into());
                }
                face_count = Some(n);
                in_faces = true;
            }
            ["property", "list", count, index, name]
                if in_faces
                    && face_layout.is_none()
                    && matches!(*name, "vertex_indices" | "vertex_index") =>
            {
                let c = Kind::from(count)?;
                let i = Kind::from(index)?;
                if !matches!(c, Kind::U8 | Kind::U32) || !matches!(i, Kind::U32 | Kind::I32) {
                    return Err("lista de faces PLY inválida".into());
                }
                face_layout = Some((c, i));
            }
            _ => return Err("layout PLY COLMAP não suportado".into()),
        }
    }
    let count = vertices.ok_or("PLY sem vértices")?;
    let names: Vec<_> = properties.iter().map(|p| p.1).collect();
    let mut expected = vec!["x", "y", "z"];
    if names.get(3) == Some(&"nx") {
        expected.extend(["nx", "ny", "nz"]);
    }
    // Poisson emits a density scalar called value; it is not measurement uncertainty.
    if names.get(expected.len()) == Some(&"value") {
        expected.push("value");
    }
    if names.last() == Some(&"blue") {
        expected.extend(["red", "green", "blue"]);
    }
    let valid = names == expected;
    if !valid {
        return Err("PLY suporta XYZ, normal XYZ opcional e RGB opcional nessa ordem".into());
    }
    let rgb = names.last() == Some(&"blue");
    let normals = names.get(3) == Some(&"nx");
    let float_fields = if rgb {
        properties.len() - 3
    } else {
        properties.len()
    };
    if properties[..float_fields]
        .iter()
        .any(|p| !matches!(p.0, Kind::F32 | Kind::F64))
        || rgb && properties[float_fields..].iter().any(|p| p.0 != Kind::U8)
    {
        return Err("tipos de vértice PLY incompatíveis".into());
    }
    let nfaces = face_count.unwrap_or(0);
    if face_count.is_some() && face_layout.is_none() {
        return Err("PLY com faces sem lista de índices".into());
    }
    let mut data = if binary {
        Data::Binary {
            bytes: &bytes[end..],
            offset: 0,
        }
    } else {
        Data::Ascii(std::str::from_utf8(&bytes[end..])?.split_whitespace())
    };
    let mut positions = Vec::with_capacity(count);
    let mut colors = rgb.then(|| Vec::with_capacity(count));
    for _ in 0..count {
        let mut values = [0.; 10];
        for (i, p) in properties.iter().enumerate() {
            values[i] = data.number(p.0)?;
        }
        positions.push([values[0], values[1], values[2]]);
        if let Some(colors) = &mut colors {
            colors.push([
                values[float_fields] as u8,
                values[float_fields + 1] as u8,
                values[float_fields + 2] as u8,
            ]);
        }
    }
    let mut faces = Vec::with_capacity(nfaces);
    for _ in 0..nfaces {
        let (ck, ik) = face_layout.ok_or("lista ausente")?;
        if data.number(ck)? != 3. {
            return Err("apenas faces triangulares PLY são suportadas".into());
        }
        let mut f = [0; 3];
        for i in &mut f {
            let n = data.number(ik)?;
            if n < 0. || n >= count as f64 {
                return Err("índice de face PLY fora dos vértices".into());
            }
            *i = n as u32;
        }
        if f[0] == f[1] || f[1] == f[2] || f[0] == f[2] {
            return Err("face PLY com índices repetidos".into());
        }
        faces.push(f);
    }
    if !data.end() {
        return Err("dados excedentes após PLY COLMAP".into());
    }
    Ok(Parsed {
        positions,
        colors,
        faces,
        normals,
    })
}
pub fn parse_mesh(bytes: &[u8]) -> Result<TriangleMesh> {
    let p = parse(bytes)?;
    if p.faces.is_empty() {
        return Err("superfície sem faces; nuvem não é malha".into());
    }
    Ok(TriangleMesh::new(p.positions, p.faces)?)
}
pub fn parse_oriented_cloud(bytes: &[u8]) -> Result<PointCloud> {
    let p = parse(bytes)?;
    if !p.normals || !p.faces.is_empty() {
        return Err("nuvem densa deve ter normais e não conter faces".into());
    }
    Ok(PointCloud::new(p.positions, p.colors)?)
}
/// OBJ contains geometry only; input PLY normals/colors are preserved in the raw artifact.
pub fn encode_obj(mesh: &TriangleMesh) -> Result<Vec<u8>> {
    use std::fmt::Write;
    let mut s = String::from(
        "# NOAH normalized COLMAP surface; arbitrary SfM frame; unknown unit; no UV texture\n",
    );
    for p in mesh.positions() {
        if p.iter().any(|v| v.abs() > 1e12) {
            return Err("superfície fora do alcance visual +/-1e12".into());
        }
        writeln!(&mut s, "v {} {} {}", p[0], p[1], p[2])?;
        if s.len() as u64 > MAX_PLY_BYTES {
            return Err("OBJ normalizado excede 64 MiB".into());
        }
    }
    for f in mesh.triangles() {
        writeln!(&mut s, "f {} {} {}", f[0] + 1, f[1] + 1, f[2] + 1)?;
        if s.len() as u64 > MAX_PLY_BYTES {
            return Err("OBJ normalizado excede 64 MiB".into());
        }
    }
    Ok(s.into_bytes())
}
