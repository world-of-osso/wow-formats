#![cfg(feature = "animation")]
use wow_m2::m2_anim::{first_bone_transform, parse_sequence_aliases_at};

#[test]
fn geometry_first_keys_keep_external_and_global_sources_distinct() {
    let mut blob = vec![0; 512];
    let mut external = vec![0; 64];
    // Bone 0 starts at 32; translation block 0x10. Keys, unlike timestamps, suffice.
    blob[64..68].copy_from_slice(&1u32.to_le_bytes());
    blob[48 + 2..48 + 4].copy_from_slice(&(-1i16).to_le_bytes());
    blob[48 + 12..48 + 16].copy_from_slice(&1u32.to_le_bytes());
    blob[48 + 16..48 + 20].copy_from_slice(&128u32.to_le_bytes());
    blob[128..132].copy_from_slice(&1u32.to_le_bytes());
    blob[132..136].copy_from_slice(&16u32.to_le_bytes());
    for (offset, value) in [(16, 1.0f32), (20, 2.0), (24, 3.0)] {
        external[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    let keys = first_bone_transform(&blob, 32, 0, 0, &external).unwrap();
    assert_eq!(keys.translation, Some([1.0, 2.0, 3.0]));
    assert_eq!(keys.rotation, None);
    assert_eq!(keys.scale, None);
    assert!(first_bone_transform(&blob, 32, 0, 0, &external[..20]).is_err());
    // A global track reads its key from the containing blob, not external bytes.
    blob[50..52].copy_from_slice(&0i16.to_le_bytes());
    blob[16..20].copy_from_slice(&4f32.to_le_bytes());
    blob[20..24].copy_from_slice(&5f32.to_le_bytes());
    blob[24..28].copy_from_slice(&6f32.to_le_bytes());
    assert_eq!(
        first_bone_transform(&blob, 32, 0, 5, &external)
            .unwrap()
            .translation,
        Some([4.0, 5.0, 6.0])
    );
}

#[test]
fn sequence_aliases_are_not_variation_links() {
    let mut data = vec![0; 144];
    data[16 + 0x3c..16 + 0x3e].copy_from_slice(&(-1i16).to_le_bytes());
    data[16 + 0x3e..16 + 0x40].copy_from_slice(&1u16.to_le_bytes());
    data[80 + 0x3e..80 + 0x40].copy_from_slice(&0u16.to_le_bytes());
    assert_eq!(parse_sequence_aliases_at(&data, 16, 2).unwrap(), [1, 0]);
    assert!(parse_sequence_aliases_at(&data[..143], 16, 2).is_err());
}
