//! Server geometry readers need no particle/light/camera/texture-composition modules.
use wow_m2::{m2_attach::attachment_position, parser};

#[test]
fn static_attachment_lookup_does_not_parse_the_activation_track() {
    let mut md20 = vec![0; 0x200];
    md20[0xf0..0xf4].copy_from_slice(&1u32.to_le_bytes());
    md20[0xf4..0xf8].copy_from_slice(&0x180u32.to_le_bytes());
    md20[0xf8..0xfc].copy_from_slice(&21u32.to_le_bytes());
    md20[0xfc..0x100].copy_from_slice(&0x100u32.to_le_bytes());
    md20[0x128..0x12a].copy_from_slice(&0i16.to_le_bytes());
    for (offset, value) in [(0x188, 1.25f32), (0x18c, -2.5), (0x190, 3.75)] {
        md20[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    // Invalid activation arrays are irrelevant to a static seat's authored offset.
    md20[0x198..0x19c].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(attachment_position(&md20, 20).unwrap(), [1.25, -2.5, 3.75]);
    assert!(attachment_position(&md20, 21).is_err());
    md20[0x128..0x12a].copy_from_slice(&(-1i16).to_le_bytes());
    assert!(attachment_position(&md20, 20).is_err());
}

#[test]
fn skin_retains_authored_high_index_bits_without_changing_cumulative_ranges() {
    let mut skin = vec![0; 44 + 48];
    skin[..4].copy_from_slice(b"SKIN");
    skin[28..32].copy_from_slice(&1u32.to_le_bytes());
    skin[32..36].copy_from_slice(&44u32.to_le_bytes());
    skin[46..48].copy_from_slice(&1u16.to_le_bytes());
    skin[52..54].copy_from_slice(&9u16.to_le_bytes());
    skin[54..56].copy_from_slice(&3u16.to_le_bytes());
    let parsed = parser::parse_skin_full(&skin).unwrap();
    assert_eq!(parsed.submeshes[0].triangle_start, 0);
    assert_eq!(parsed.submeshes[0].raw_triangle_start, 65_545);
    assert_eq!(parsed.submeshes[0].triangle_count, 3);
    assert!(parsed.batches.is_empty());
}
