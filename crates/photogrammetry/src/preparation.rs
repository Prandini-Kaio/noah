//! Bounded photo audit and immutable, explicitly selected image preparation.
use noah_pipeline::{Result, directory, hash_file, read_bounded, regular, write_new};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Photo {
    pub name: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub laplacian_variance: f64,
    pub dark_fraction: f64,
    pub bright_fraction: f64,
    pub duplicate_of: Option<String>,
    pub warning: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Audit {
    pub schema_version: u32,
    pub source: PathBuf,
    pub photos: Vec<Photo>,
}
fn decode(bytes: &[u8]) -> Result<image::DynamicImage> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg)
    ) {
        return Err("somente JPEG/PNG".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12000);
    limits.max_image_height = Some(12000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode()?;
    if image.width() as u64 * image.height() as u64 > 32_000_000 {
        return Err("imagem excede 32 megapixels".into());
    }
    Ok(image)
}
pub fn analyze(root: &Path) -> Result<Audit> {
    directory(root)?;
    let root = fs::canonicalize(root)?;
    let mut paths = Vec::new();
    for entry in fs::read_dir(&root)? {
        let p = entry?.path();
        if p.extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        {
            regular(&p)?;
            paths.push(p);
            if paths.len() > 500 {
                return Err("máximo de 500 fotos".into());
            }
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err("pasta sem fotografias".into());
    }
    let mut total = 0;
    let mut photos = Vec::new();
    let mut seen = HashMap::new();
    for p in paths {
        let bytes = read_bounded(&p, 32 * 1024 * 1024)?;
        total += bytes.len();
        if total > 512 * 1024 * 1024 {
            return Err("fotos excedem 512 MiB".into());
        }
        let sha = noah_storage::content_sha256(&bytes);
        let image = decode(&bytes)?;
        let gray = image.thumbnail(512, 512).to_luma8();
        let (w, h) = gray.dimensions();
        let mut sum = 0.;
        let mut sum2 = 0.;
        let mut n = 0.;
        if w > 2 && h > 2 {
            for y in 1..h - 1 {
                for x in 1..w - 1 {
                    let at = |x, y| gray.get_pixel(x, y)[0] as f64;
                    let l =
                        at(x - 1, y) + at(x + 1, y) + at(x, y - 1) + at(x, y + 1) - 4. * at(x, y);
                    sum += l;
                    sum2 += l * l;
                    n += 1.;
                }
            }
        }
        let variance = if n > 0. {
            (sum2 / n - (sum / n).powi(2)).max(0.)
        } else {
            0.
        };
        let count = (w as u64 * h as u64) as f64;
        let dark = gray.pixels().filter(|p| p[0] <= 8).count() as f64 / count;
        let bright = gray.pixels().filter(|p| p[0] >= 247).count() as f64 / count;
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("nome inválido")?
            .to_owned();
        let duplicate = seen.get(&sha).cloned();
        seen.entry(sha.clone()).or_insert(name.clone());
        let mut warning = Vec::new();
        if variance < 40. {
            warning.push("pouco detalhe/desfoque possível".into());
        }
        if dark > 0.25 {
            warning.push("regiões muito escuras".into());
        }
        if bright > 0.25 {
            warning.push("regiões saturadas".into());
        }
        if image.width() < 640 || image.height() < 480 {
            warning.push("baixa resolução".into());
        }
        if duplicate.is_some() {
            warning.push("duplicata exata".into());
        }
        photos.push(Photo {
            name,
            sha256: sha,
            width: image.width(),
            height: image.height(),
            laplacian_variance: variance,
            dark_fraction: dark,
            bright_fraction: bright,
            duplicate_of: duplicate,
            warning,
        });
    }
    Ok(Audit {
        schema_version: 1,
        source: root,
        photos,
    })
}
/// Copies selected originals or resizes to PNG; never modifies source photos.
pub fn prepare(audit: &Audit, selected: &[bool], max_dimension: u32, output: &Path) -> Result<()> {
    if audit.schema_version != 1
        || selected.len() != audit.photos.len()
        || selected.iter().filter(|&&v| v).count() < 3
        || audit.photos.len() > 500
        || !(max_dimension == 0 || (320..=12000).contains(&max_dimension))
    {
        return Err("selecione ≥3 fotos; tamanho 0 (original) ou 320–12000".into());
    }
    // Preflight integrity before creating a destination. Rechecked on copied bytes below.
    for (p, &keep) in audit.photos.iter().zip(selected) {
        if keep {
            if Path::new(&p.name).components().count() != 1
                || p.name.contains(['/', '\\'])
                || matches!(p.name.as_str(), "." | "..")
            {
                return Err("nome de foto inválido".into());
            }
            if hash_file(&audit.source.join(&p.name), 32 * 1024 * 1024)?.0 != p.sha256 {
                return Err("foto alterada após análise; analise novamente".into());
            }
        }
    }
    fs::create_dir(output)?;
    fs::create_dir(output.join("images"))?;
    let mut entries = Vec::new();
    let mut total = 0;
    let result = (|| -> Result<()> {
        for (i, (p, &keep)) in audit.photos.iter().zip(selected).enumerate() {
            if !keep {
                continue;
            }
            let src = audit.source.join(&p.name);
            let bytes = read_bounded(&src, 32 * 1024 * 1024)?;
            let copied = noah_storage::content_sha256(&bytes);
            if copied != p.sha256 {
                return Err("foto alterada durante preparo".into());
            }
            let (name, prepared, w, h) = if max_dimension == 0 {
                let ext = if bytes.starts_with(b"\x89PNG") {
                    "png"
                } else {
                    "jpg"
                };
                (format!("{i:04}.{ext}"), bytes, p.width, p.height)
            } else {
                let image = decode(&bytes)?;
                let image = if image.width().max(image.height()) > max_dimension {
                    image.resize(
                        max_dimension,
                        max_dimension,
                        image::imageops::FilterType::Lanczos3,
                    )
                } else {
                    image
                };
                let (w, h) = (image.width(), image.height());
                let mut data = Cursor::new(Vec::new());
                image.write_to(&mut data, image::ImageFormat::Png)?;
                (format!("{i:04}.png"), data.into_inner(), w, h)
            };
            total += prepared.len();
            if prepared.len() > 32 * 1024 * 1024 || total > 512 * 1024 * 1024 {
                return Err("imagens preparadas excedem limites de captura".into());
            }
            write_new(&output.join("images").join(&name), &prepared)?;
            let hash = hash_file(&output.join("images").join(&name), 32 * 1024 * 1024)?.0;
            entries.push(serde_json::json!({"original":p,"prepared":name,"sha256":hash,"width":w,"height":h}));
        }
        write_new(
            &output.join("preparation.json"),
            &serde_json::to_vec_pretty(
                &serde_json::json!({"schema_version":1,"source":audit.source,"audit":audit,"selected":selected,"max_dimension":max_dimension,"images":entries,"photometric_correction":false,"orientation":"encoded pixels unchanged; no EXIF auto-rotation","note":"Heuristics only; resized PNG drops EXIF, intrinsics re-estimated by COLMAP; no calibration guarantee"}),
            )?,
        )?;
        Ok(())
    })();
    if let Err(e) = result {
        let _ = write_new(&output.join("FAILED.txt"), e.to_string().as_bytes());
        return Err(e);
    }
    Ok(())
}
/// Verify a prepared input folder and return provenance to attach to a new run.
pub fn provenance(images: &Path) -> Result<Option<Vec<u8>>> {
    let Some(parent) = images.parent() else {
        return Ok(None);
    };
    if images.file_name().and_then(|s| s.to_str()) != Some("images")
        || !parent.join("preparation.json").is_file()
    {
        return Ok(None);
    }
    let bytes = read_bounded(&parent.join("preparation.json"), 4 * 1024 * 1024)?;
    let meta: serde_json::Value = serde_json::from_slice(&bytes)?;
    if meta["schema_version"] != 1 {
        return Err("preparo incompatível".into());
    }
    let entries = meta["images"].as_array().ok_or("preparo sem entradas")?;
    if !(3..=500).contains(&entries.len()) {
        return Err("preparo inválido".into());
    }
    let mut actual = Vec::new();
    for entry in fs::read_dir(images)? {
        let p = entry?.path();
        if p.extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        {
            actual.push(p.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    let mut expected = Vec::new();
    for e in entries {
        let name = e["prepared"].as_str().ok_or("nome preparado inválido")?;
        if Path::new(name).components().count() != 1 || name.contains(['/', '\\']) {
            return Err("caminho preparado inválido".into());
        }
        if hash_file(&images.join(name), 32 * 1024 * 1024)?.0
            != e["sha256"].as_str().ok_or("hash preparado ausente")?
        {
            return Err("preparo alterado".into());
        }
        expected.push(name.to_string());
    }
    actual.sort();
    expected.sort();
    if actual != expected {
        return Err("conjunto de fotos preparado alterado".into());
    }
    Ok(Some(bytes))
}
