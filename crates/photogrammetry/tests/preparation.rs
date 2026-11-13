use noah_photogrammetry::preparation::{analyze, prepare, provenance};
use std::{fs, path::PathBuf};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("noah-prep-{:032x}", noah_storage::new_id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn audit_duplicates_exposure_selection_resize_hashes_and_tamper() {
    let t = Temp::new();
    let photos = t.0.join("photos");
    fs::create_dir(&photos).unwrap();
    let image = image::RgbImage::from_pixel(800, 600, image::Rgb([0, 0, 0]));
    image.save(photos.join("a.png")).unwrap();
    image.save(photos.join("b.png")).unwrap();
    image::RgbImage::from_fn(800, 600, |x, y| {
        image::Rgb([((x + y) % 256) as u8, (x % 256) as u8, (y % 256) as u8])
    })
    .save(photos.join("c.jpg"))
    .unwrap();
    image::RgbImage::from_pixel(800, 600, image::Rgb([255, 255, 255]))
        .save(photos.join("d.png"))
        .unwrap();
    let a = analyze(&photos).unwrap();
    assert_eq!(a.photos.len(), 4);
    assert_eq!(a.photos[0].dark_fraction, 1.);
    assert_eq!(a.photos[1].duplicate_of.as_deref(), Some("a.png"));
    assert_eq!(a.photos[3].bright_fraction, 1.);
    let output = t.0.join("prepared");
    prepare(&a, &[true, false, true, true], 400, &output).unwrap();
    let p = image::open(output.join("images/0000.png")).unwrap();
    assert_eq!((p.width(), p.height()), (400, 300));
    assert!(provenance(&output.join("images")).unwrap().is_some());
    assert_eq!(
        analyze(&photos).unwrap().photos[0].sha256,
        a.photos[0].sha256
    );
    fs::write(output.join("images/0000.png"), b"changed").unwrap();
    assert!(provenance(&output.join("images")).is_err());
    assert!(prepare(&a, &[true; 4], 0, &output).is_err());
    fs::write(photos.join("a.png"), b"changed").unwrap();
    assert!(prepare(&a, &[true; 4], 0, &t.0.join("rejected")).is_err());
    assert!(!t.0.join("rejected").exists());
    assert!(analyze(&photos).is_err());
}
#[test]
fn preparation_validates_empty_truncated_limits_and_preserves_original_bytes() {
    let t = Temp::new();
    assert!(analyze(&t.0).is_err());
    let photos = t.0.join("images");
    fs::create_dir(&photos).unwrap();
    for i in 0..3 {
        image::RgbImage::from_pixel(12, 10, image::Rgb([i, 10, 20]))
            .save(photos.join(format!("{i}.png")))
            .unwrap();
    }
    let a = analyze(&photos).unwrap();
    assert!(prepare(&a, &[true, true, false], 0, &t.0.join("bad")).is_err());
    assert!(prepare(&a, &[true; 3], 1, &t.0.join("bad")).is_err());
    let out = t.0.join("ok");
    prepare(&a, &[true; 3], 0, &out).unwrap();
    for i in 0..3 {
        assert_eq!(
            fs::read(photos.join(format!("{i}.png"))).unwrap(),
            fs::read(out.join(format!("images/{i:04}.png"))).unwrap()
        );
    }
    fs::copy(photos.join("0.png"), out.join("images/unlisted.png")).unwrap();
    assert!(provenance(&out.join("images")).is_err());
}

#[test]
fn rejects_decoded_dimension_limit_before_analysis() {
    let t = Temp::new();
    image::RgbImage::new(12001, 1)
        .save(t.0.join("too-wide.png"))
        .unwrap();
    assert!(analyze(&t.0).is_err());
}
