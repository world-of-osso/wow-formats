use wow_adt::adt::{parse_mcvt, parse_terrain_height_header};

fn header() -> Vec<u8> {
    let mut data = vec![0; 128];
    data[..4].copy_from_slice(&0x41u32.to_le_bytes());
    data[4..8].copy_from_slice(&9u32.to_le_bytes());
    data[8..12].copy_from_slice(&14u32.to_le_bytes());
    data[0x34..0x38].copy_from_slice(&77u32.to_le_bytes());
    data[0x3c..0x3e].copy_from_slice(&0x20u16.to_le_bytes());
    for (offset, bits) in [
        (0x68, (-0.0f32).to_bits()),
        (0x6c, (-543.25f32).to_bits()),
        (0x70, 18.5f32.to_bits()),
    ] {
        data[offset..offset + 4].copy_from_slice(&bits.to_le_bytes());
    }
    data
}

#[test]
fn header_projects_raw_origin_area_and_low_holes_without_render_payloads() {
    let parsed = parse_terrain_height_header(&header()).unwrap();
    assert_eq!(
        parsed.origin.map(f32::to_bits),
        [
            (-0.0f32).to_bits(),
            (-543.25f32).to_bits(),
            18.5f32.to_bits()
        ]
    );
    assert_eq!((parsed.index_x, parsed.index_y), (9, 14));
    assert_eq!(parsed.area_id, 77);
    assert_eq!(parsed.holes_low_res, 0x20);
    assert_eq!(parsed.holes_high_res, None);
}

#[test]
fn header_keeps_high_mask_without_widening_or_erasing_the_low_mask() {
    let mut data = header();
    data[..4].copy_from_slice(&0x10000u32.to_le_bytes());
    data[0x14..0x1c].copy_from_slice(&0x8000_0000_0000_c0c0u64.to_le_bytes());
    let parsed = parse_terrain_height_header(&data).unwrap();
    assert_eq!(parsed.holes_high_res, Some(0x8000_0000_0000_c0c0));
    assert_eq!(parsed.holes_low_res, 0x20);
}

#[test]
fn header_rejects_short_payload_with_existing_size_error() {
    assert_eq!(
        parse_terrain_height_header(&[0; 127]).err().unwrap(),
        "MCNK too small: 127 bytes"
    );
}

#[test]
fn mcvt_reads_exact_145_float_bits_and_ignores_unused_tail() {
    let bits: Vec<u32> = (0..145)
        .map(|n| {
            if n == 0 {
                (-0.0f32).to_bits()
            } else {
                (n as f32 * 0.25).to_bits()
            }
        })
        .collect();
    let mut bytes: Vec<_> = bits.iter().flat_map(|word| word.to_le_bytes()).collect();
    bytes.extend_from_slice(&999f32.to_le_bytes());
    assert_eq!(parse_mcvt(&bytes).unwrap().map(f32::to_bits).to_vec(), bits);
    assert_eq!(
        parse_mcvt(&bytes[..579]).unwrap_err(),
        "MCVT too small: 579 bytes (need 580)"
    );
}
