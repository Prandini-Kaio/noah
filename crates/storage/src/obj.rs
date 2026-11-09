//! OBJ triangular restrito; sem referências externas a material/textura.
use crate::{MAX_PLY_BYTES, MAX_POINTS, Result};
use noah_geometry::TriangleMesh;
pub fn parse(bytes: &[u8]) -> Result<TriangleMesh> {
    if bytes.len() as u64 > MAX_PLY_BYTES {
        return Err("OBJ excede 64 MiB".into());
    }
    let text = std::str::from_utf8(bytes)?;
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for line in text.lines() {
        if line.len() > 4096 {
            return Err("linha OBJ excede 4096 bytes".into());
        }
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["v", x, y, z] => {
                if positions.len() >= MAX_POINTS {
                    return Err("OBJ excede limite de vértices".into());
                }
                positions.push([x.parse()?, y.parse()?, z.parse()?]);
            }
            ["f", a, b, c] => {
                if triangles.len() >= 100_000 {
                    return Err("OBJ excede limite de faces".into());
                }
                let mut face = [0; 3];
                for (i, text) in [a, b, c].iter().enumerate() {
                    let index: u32 = text.parse()?;
                    face[i] = index.checked_sub(1).ok_or("OBJ exige índices positivos")?;
                }
                triangles.push(face);
            }
            _ => return Err("OBJ suporta apenas v x y z e f a b c com índices positivos".into()),
        }
    }
    if triangles.is_empty() {
        return Err("OBJ sem faces triangulares".into());
    }
    Ok(TriangleMesh::new(positions, triangles)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mesh_and_hostile_variants() {
        let good = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        assert_eq!(parse(good.as_bytes()).unwrap().triangles(), &[[0, 1, 2]]);
        for invalid in [
            good.replace("f 1 2 3", "f 1 2 4"),
            good.replace("f 1 2 3", "f -1 2 3"),
            good.replace("f 1 2 3", "f 1/1 2/2 3/3"),
            good.replace("v 0 0 0", "v NaN 0 0"),
            good.replace("f 1 2 3", "f 1 1 3"),
            format!("{good}mtllib outside.mtl\n"),
        ] {
            assert!(parse(invalid.as_bytes()).is_err());
        }
    }
}
