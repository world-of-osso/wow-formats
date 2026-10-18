use wow_m2::m2_collision::parse_collision_normals;

#[test]
fn collision_normals_preserve_authored_sign_and_require_one_per_triangle() {
    let mut data = vec![0; 272];
    data[0xd8..0xdc].copy_from_slice(&6u32.to_le_bytes());
    data[0xe8..0xec].copy_from_slice(&2u32.to_le_bytes());
    data[0xec..0xf0].copy_from_slice(&240u32.to_le_bytes());
    data[248..252].copy_from_slice(&(-1f32).to_le_bytes());
    data[260..264].copy_from_slice(&1f32.to_le_bytes());
    assert_eq!(
        parse_collision_normals(&data).unwrap(),
        [[0.0, 0.0, -1.0], [0.0, 0.0, 1.0]]
    );
    data[0xe8..0xec].copy_from_slice(&1u32.to_le_bytes());
    assert!(
        parse_collision_normals(&data)
            .unwrap_err()
            .contains("face normals")
    );
}

#[test]
fn collision_normals_reject_nonfinite_authored_values() {
    let mut data = vec![0; 272];
    data[0xd8..0xdc].copy_from_slice(&3u32.to_le_bytes());
    data[0xe8..0xec].copy_from_slice(&1u32.to_le_bytes());
    data[0xec..0xf0].copy_from_slice(&240u32.to_le_bytes());
    data[240..244].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(parse_collision_normals(&data).is_err());
}
