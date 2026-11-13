use noah_storage::colmap_ply;
fn header(format: &str, props: &str, n: usize, faces: usize) -> String {
    format!(
        "ply\nformat {format} 1.0\nelement vertex {n}\n{props}element face {faces}\nproperty list uchar int vertex_index\nend_header\n"
    )
}
const XYZ: &str = "property float x\nproperty float y\nproperty float z\n";
#[test]
fn ascii_and_binary_mesh_roundtrip_geometry() {
    let ascii = header("ascii", XYZ, 3, 1) + "0 0 0\n1 0 0\n0 1 0\n3 0 1 2\n";
    let mesh = colmap_ply::parse_mesh(ascii.as_bytes()).unwrap();
    assert_eq!(mesh.triangles(), &[[0, 1, 2]]);
    let mut binary = header("binary_little_endian", XYZ, 3, 1).into_bytes();
    for p in [[0f32, 0., 0.], [1., 0., 0.], [0., 1., 0.]] {
        for v in p {
            binary.extend(v.to_le_bytes());
        }
    }
    binary.push(3);
    for i in [0i32, 1, 2] {
        binary.extend(i.to_le_bytes());
    }
    let b = colmap_ply::parse_mesh(&binary).unwrap();
    assert_eq!(b.positions(), mesh.positions());
    let obj = colmap_ply::encode_obj(&b).unwrap();
    assert_eq!(noah_storage::obj::parse(&obj).unwrap(), mesh);
    for bad in [
        ascii.replace("3 0 1 2", "3 0 1 99"),
        ascii.replace("3 0 1 2", "3 0 -1 2"),
        ascii.replace("3 0 1 2", "3 0 0 2"),
        ascii.replace("3 0 1 2", "4 0 1 2 0"),
        ascii.replace("0 1 0", "NaN 1 0"),
        ascii.clone() + "extra",
    ] {
        assert!(colmap_ply::parse_mesh(bad.as_bytes()).is_err());
    }
    for n in [1, 4, 13] {
        assert!(colmap_ply::parse_mesh(&binary[..binary.len() - n]).is_err());
    }
}
#[test]
fn oriented_cloud_preserves_rgb_and_rejects_wrong_layout() {
    let h = "ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nproperty float nx\nproperty float ny\nproperty float nz\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n";
    let mut bytes = h.as_bytes().to_vec();
    for v in [1f32, 2., 3., 0., 0., 1.] {
        bytes.extend(v.to_le_bytes());
    }
    bytes.extend([12, 34, 56]);
    let cloud = colmap_ply::parse_oriented_cloud(&bytes).unwrap();
    assert_eq!(cloud.positions(), &[[1., 2., 3.]]);
    assert_eq!(cloud.colors().unwrap(), &[[12, 34, 56]]);
    assert!(colmap_ply::parse_mesh(&bytes).is_err());
    let mut nan = bytes.clone();
    nan[h.len()..h.len() + 4].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(colmap_ply::parse_oriented_cloud(&nan).is_err());
    let simple = header("ascii", XYZ, 1, 0) + "0 0 0\n";
    assert!(colmap_ply::parse_oriented_cloud(simple.as_bytes()).is_err());
}
#[test]
fn limits_headers_and_integer_types_are_strict() {
    for text in [
        header("ascii", XYZ, 1_000_001, 1),
        header("ascii", XYZ, 3, 100_001),
        header("binary_big_endian", XYZ, 3, 1),
        header("ascii", XYZ, 0, 0),
        header(
            "ascii",
            "property float x\nproperty float y\nproperty float confidence\n",
            3,
            1,
        ),
    ] {
        assert!(colmap_ply::parse_mesh(text.as_bytes()).is_err());
    }
    let good = header("ascii", XYZ, 3, 1) + "0 0 0 1 0 0 0 1 0 3 0 1 2";
    assert!(
        colmap_ply::parse_mesh(
            good.replace("list uchar int", "list float float")
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        colmap_ply::parse_mesh(good.replace("vertex_index", "texture_coords").as_bytes()).is_err()
    );
    assert!(
        colmap_ply::parse_mesh(
            good.replace("property float z", "property uchar z")
                .as_bytes()
        )
        .is_err()
    );
}

#[test]
fn poisson_density_scalar_is_preserved_only_in_raw_geometry_contract() {
    let props = "property float x\nproperty float y\nproperty float z\nproperty float value\nproperty uchar red\nproperty uchar green\nproperty uchar blue\n";
    let text = header("ascii", props, 3, 1)
        + "0 0 0 4 100 170 190\n1 0 0 4.1 100 170 190\n0 1 0 4.2 100 170 190\n3 0 1 2";
    assert_eq!(
        colmap_ply::parse_mesh(text.as_bytes()).unwrap().triangles(),
        &[[0, 1, 2]]
    );
    assert!(colmap_ply::parse_mesh(text.replace("4.1", "NaN").as_bytes()).is_err());
}
