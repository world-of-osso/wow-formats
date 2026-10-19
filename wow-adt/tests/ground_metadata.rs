use wow_adt::{
    adt_obj::{modf_flags, parse_modf_transform},
    wdt::parse_maid_root_obj0,
};

#[test]
fn projected_maid_keeps_all_slots_without_requiring_unused_fields() {
    let mut payload = vec![0; 4095 * 32 + 8];
    payload[..4].copy_from_slice(&77u32.to_le_bytes());
    let last = 4095 * 32;
    payload[last..last + 4].copy_from_slice(&123u32.to_le_bytes());
    payload[last + 4..last + 8].copy_from_slice(&456u32.to_le_bytes());
    let slots = parse_maid_root_obj0(&payload).unwrap();
    assert_eq!(slots.len(), 4096);
    assert_eq!(slots[0], (77, 0));
    assert_eq!(slots[4095], (123, 456));
    payload.pop();
    assert!(parse_maid_root_obj0(&payload).is_err());
}

#[test]
fn projected_modf_preserves_raw_transform_and_optional_authored_scale() {
    let mut entry = vec![0; 58];
    entry[..4].copy_from_slice(&108631u32.to_le_bytes());
    entry[56..58].copy_from_slice(&8u16.to_le_bytes());
    for (offset, value) in [
        (8, 1.25f32),
        (12, -2.5),
        (16, 3.75),
        (20, 10.0),
        (24, 20.0),
        (28, 30.0),
    ] {
        entry[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    let transform = parse_modf_transform(&entry).unwrap();
    assert_eq!(transform.name_id, 108631);
    assert_eq!(transform.position, [1.25, -2.5, 3.75]);
    assert_eq!(transform.rotation, [10.0, 20.0, 30.0]);
    assert_eq!(transform.scale.to_bits(), 1f32.to_bits());
    entry[56..58].copy_from_slice(&12u16.to_le_bytes());
    assert!(parse_modf_transform(&entry).is_err());
    entry.resize(64, 0);
    entry[62..64].copy_from_slice(&2048u16.to_le_bytes());
    assert_eq!(
        parse_modf_transform(&entry).unwrap().scale.to_bits(),
        2f32.to_bits()
    );
}

#[test]
fn projected_modf_flags_are_available_before_geometry_or_path_policy() {
    let mut entry = vec![0; 58];
    entry[56..58].copy_from_slice(&4u16.to_le_bytes());
    assert_eq!(modf_flags(&entry).unwrap(), 4);
    assert!(parse_modf_transform(&entry).is_err());
    assert!(modf_flags(&entry[..57]).is_err());
}
