#![cfg(unix)]
use noah_photogrammetry::{Config, model, reconstruct};
use noah_pipeline::{Status, inspect};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "noah-adapter-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const CAMERAS: &str = "1 SIMPLE_RADIAL 800 600 700 400 300 0\n";
const IMAGES: &str =
    "1 1 0 0 0 0 0 0 1 0000.jpg\n400 300 1\n2 1 0 0 0 1 0 0 1 0001.jpg\n400 300 1\n";
const POINTS: &str = "1 1 2 3 255 0 0 0.2 1 0 2 0\n";
#[test]
fn parser_checks_camera_poses_points_and_references() {
    let t = Temp::new();
    fs::write(t.0.join("cameras.txt"), CAMERAS).unwrap();
    fs::write(t.0.join("images.txt"), IMAGES).unwrap();
    fs::write(t.0.join("points3D.txt"), POINTS).unwrap();
    let r = model::parse(&t.0).unwrap();
    assert_eq!(r.views.len(), 2);
    assert_eq!(r.cloud.positions(), &[[1., 2., 3.]]);
    for invalid in [
        IMAGES.replace("1 1 0 0 0", "1 2 0 0 0"),
        IMAGES.replace("1 0001.jpg", "99 0001.jpg"),
        IMAGES.replace("400 300 1", "NaN 300 1"),
        IMAGES.replace("0001.jpg", "../other.jpg"),
    ] {
        fs::write(t.0.join("images.txt"), invalid).unwrap();
        assert!(model::parse(&t.0).is_err());
    }
    fs::write(t.0.join("images.txt"), IMAGES).unwrap();
    fs::write(
        t.0.join("points3D.txt"),
        POINTS.replace("1 1 2 3", "1 NaN 2 3"),
    )
    .unwrap();
    assert!(model::parse(&t.0).is_err());
}
fn setup(t: &Temp, mode: &str) -> Config {
    let images = t.0.join("photos");
    fs::create_dir(&images).unwrap();
    for i in 0..3 {
        fs::write(images.join(format!("{i}.jpg")), b"\xff\xd8\xfftest").unwrap();
    }
    let script = format!(
        r#"#!/usr/bin/env python3
import pathlib,sys,struct
mode={mode:?}
a=sys.argv[1:]
if a==['--help']:
 print('COLMAP 3.11.1 test-double');sys.exit()
def option(n):return pathlib.Path(a[a.index(n)+1])
if a[0]=='mapper' and mode!='missing':
 (option('--output_path')/'0').mkdir()
if a[0]=='model_converter':
 p=option('--output_path')
 (p/'cameras.txt').write_text({CAMERAS:?})
 (p/'images.txt').write_text({IMAGES:?})
 (p/'points3D.txt').write_text({POINTS:?} if mode!='bad' else 'invalid')
if a[0]=='stereo_fusion':
 header='ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty float nx\nproperty float ny\nproperty float nz\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n'
 option('--output_path').write_bytes(header.encode()+struct.pack('<6f3B',1,2,3,0,0,1,255,0,0))
"#
    );
    let engine = t.0.join("engine.py");
    fs::write(&engine, script).unwrap();
    fs::set_permissions(&engine, fs::Permissions::from_mode(0o755)).unwrap();
    Config {
        engine,
        images,
        output: t.0.join("run"),
        dense: mode == "dense",
        timeout: Duration::from_secs(3),
    }
}
#[test]
fn adapter_exports_sparse_and_dense_with_controlled_engine() {
    for mode in ["sparse", "dense"] {
        let t = Temp::new();
        let config = setup(&t, mode);
        let out = config.output.clone();
        reconstruct(config).unwrap();
        let m = inspect(&out).unwrap();
        assert_eq!(m.status, Status::Succeeded);
        assert!(out.join("exports/reconstruction.json").exists());
        let cloud =
            noah_storage::ply::parse(&fs::read(out.join("exports/sparse.ply")).unwrap()).unwrap();
        assert_eq!(cloud.positions(), &[[1., 2., 3.]]);
        if mode == "dense" {
            assert!(out.join("exports/dense.ply").exists());
        }
    }
}
#[test]
fn zero_exit_without_valid_outputs_is_failure() {
    for mode in ["missing", "bad"] {
        let t = Temp::new();
        let config = setup(&t, mode);
        let out = config.output.clone();
        assert!(reconstruct(config).is_err());
        let m = inspect(&out).unwrap();
        assert_eq!(m.status, Status::Failed);
        assert!(m.diagnostic.is_some());
    }
}
