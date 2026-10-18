use super::*;

/// Extract MD20 blob from chunked M2 data (test helper).

/// Build a minimal MD20 blob with `n` bones at offset 0x34.
fn md20_with_bones(bones: &[(i32, u32, i16, u16, [f32; 3])]) -> Vec<u8> {
    let bone_offset: u32 = 0x48; // right after the minimum header
    let bone_count = bones.len() as u32;
    let total = bone_offset as usize + bones.len() * 88;
    let mut md20 = vec![0u8; total];

    md20[0..4].copy_from_slice(b"MD20");
    md20[4..8].copy_from_slice(&264u32.to_le_bytes());
    // bones M2Array at 0x2C
    md20[0x2C..0x30].copy_from_slice(&bone_count.to_le_bytes());
    md20[0x30..0x34].copy_from_slice(&bone_offset.to_le_bytes());

    for (i, &(key_bone, flags, parent, submesh, pivot)) in bones.iter().enumerate() {
        let base = bone_offset as usize + i * 88;
        md20[base..base + 4].copy_from_slice(&key_bone.to_le_bytes());
        md20[base + 4..base + 8].copy_from_slice(&flags.to_le_bytes());
        md20[base + 8..base + 10].copy_from_slice(&parent.to_le_bytes());
        md20[base + 10..base + 12].copy_from_slice(&submesh.to_le_bytes());
        // pivot at offset 0x4C within bone
        md20[base + 0x4C..base + 0x50].copy_from_slice(&pivot[0].to_le_bytes());
        md20[base + 0x50..base + 0x54].copy_from_slice(&pivot[1].to_le_bytes());
        md20[base + 0x54..base + 0x58].copy_from_slice(&pivot[2].to_le_bytes());
    }

    md20
}

#[test]
fn parse_zero_bones() {
    // MD20 with count=0 at offset 0x2C
    let mut md20 = vec![0u8; 0x48];
    md20[0..4].copy_from_slice(b"MD20");
    md20[0x2C..0x30].copy_from_slice(&0u32.to_le_bytes());
    md20[0x30..0x34].copy_from_slice(&0x48u32.to_le_bytes());
    let bones = parse_bones(&md20).unwrap();
    assert!(bones.is_empty());
}

#[test]
fn parse_single_root_bone() {
    let md20 = md20_with_bones(&[(-1, 0, -1, 0, [1.0, 2.0, 3.0])]);
    let bones = parse_bones(&md20).unwrap();
    assert_eq!(bones.len(), 1);
    assert_eq!(bones[0].parent_bone_id, -1);
    assert_eq!(bones[0].pivot, [1.0, 2.0, 3.0]);
}

#[test]
fn parse_bone_hierarchy() {
    let md20 = md20_with_bones(&[
        (0, 0, -1, 0, [0.0, 0.0, 0.0]), // root
        (1, 0, 0, 0, [1.0, 0.0, 0.0]),  // child of root
        (2, 0, 1, 0, [2.0, 0.0, 0.0]),  // child of bone 1
    ]);
    let bones = parse_bones(&md20).unwrap();
    assert_eq!(bones.len(), 3);
    assert_eq!(bones[0].parent_bone_id, -1);
    assert_eq!(bones[1].parent_bone_id, 0);
    assert_eq!(bones[2].parent_bone_id, 1);
    assert!(validate_bone_hierarchy(&bones).is_ok());
}

#[test]
fn validate_detects_invalid_parent() {
    let md20 = md20_with_bones(&[
        (0, 0, 5, 0, [0.0, 0.0, 0.0]), // parent=5, but only 1 bone
    ]);
    let bones = parse_bones(&md20).unwrap();
    assert!(validate_bone_hierarchy(&bones).is_err());
}

#[test]
fn keyframe_binary_search() {
    // Empty
    assert!(find_keyframe_pair(&[], 100).is_none());

    // Single keyframe
    let (i, t) = find_keyframe_pair(&[0], 500).unwrap();
    assert_eq!(i, 0);
    assert_eq!(t, 0.0);

    // Two keyframes, before start
    let (i, t) = find_keyframe_pair(&[100, 200], 50).unwrap();
    assert_eq!(i, 0);
    assert_eq!(t, 0.0);

    // Two keyframes, midpoint
    let (i, t) = find_keyframe_pair(&[100, 200], 150).unwrap();
    assert_eq!(i, 0);
    assert!((t - 0.5).abs() < 0.01);

    // Two keyframes, at end
    let (i, t) = find_keyframe_pair(&[100, 200], 200).unwrap();
    assert_eq!(i, 1);
    assert_eq!(t, 0.0);

    // Multiple keyframes
    let (i, t) = find_keyframe_pair(&[0, 100, 200, 300], 250).unwrap();
    assert_eq!(i, 2);
    assert!((t - 0.5).abs() < 0.01);
}

#[test]
fn vec3_lerp_basic() {
    let a = [0.0, 0.0, 0.0];
    let b = [10.0, 20.0, 30.0];
    let mid = lerp_vec3(&a, &b, 0.5);
    assert!((mid[0] - 5.0).abs() < 0.001);
    assert!((mid[1] - 10.0).abs() < 0.001);
    assert!((mid[2] - 15.0).abs() < 0.001);
}

#[test]
fn rotation_unpack_identity() {
    // raw=-1 (negative path): (-1 + 32768) / 32767 = 32767/32767 = 1.0
    assert!(
        (unpack_quat_component(-1) - 1.0).abs() < 0.001,
        "raw=-1 should give ~1.0"
    );
    // raw=0 (non-negative path): (0 - 32767) / 32767 = -1.0
    assert!(
        (unpack_quat_component(0) - (-1.0)).abs() < 0.001,
        "raw=0 should give ~-1.0"
    );
    // raw=-32768 (negative path): (-32768 + 32768) / 32767 = 0.0
    assert!(
        (unpack_quat_component(-32768) - 0.0).abs() < 0.001,
        "raw=-32768 should give ~0.0"
    );
    // raw=32767 (non-negative path): (32767 - 32767) / 32767 = 0.0
    assert!(
        (unpack_quat_component(32767) - 0.0).abs() < 0.001,
        "raw=32767 should give 0.0"
    );
}

#[test]
fn slerp_identity() {
    let a = [0.0, 0.0, 0.0, 1.0];
    let b = [0.0, 0.0, 0.0, 1.0];
    let result = slerp(&a, &b, 0.5);
    assert!((result[3] - 1.0).abs() < 0.001);
    assert!(result[0].abs() < 0.001);
}
