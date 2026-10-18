use wow_adt::adt::parse_terrain_height_chunks;

fn tile() -> Vec<u8> {
    let mut chunk = vec![0; 128];
    chunk[0x68..0x6c].copy_from_slice(&12.5f32.to_le_bytes());
    chunk[0x6c..0x70].copy_from_slice(&(-30f32).to_le_bytes());
    chunk[0x70..0x74].copy_from_slice(&80f32.to_le_bytes());
    chunk[0x3c..0x3e].copy_from_slice(&2u16.to_le_bytes());
    // Rendering flags may name companion-file shadows; geometry reads only heights/holes.
    chunk[..4].copy_from_slice(&1u32.to_le_bytes());
    chunk.extend_from_slice(b"TVCM");
    chunk.extend_from_slice(&(145u32 * 4).to_le_bytes());
    for value in 0..145 {
        chunk.extend_from_slice(&(value as f32 * 0.25).to_le_bytes());
    }
    let mut tile = b"KNCM".to_vec();
    tile.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
    tile.extend(chunk);
    tile
}

#[test]
fn terrain_geometry_keeps_file_origin_holes_and_heights_without_render_payloads() {
    let chunks = parse_terrain_height_chunks(&tile()).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].origin, [12.5, -30.0, 80.0]);
    assert_eq!(chunks[0].heights[144], 36.0);
    assert_eq!(chunks[0].holes_low_res, 2);
    assert_eq!(chunks[0].holes_high_res, None);
}

#[test]
fn terrain_geometry_retains_high_resolution_holes() {
    let mut data = tile();
    data[8..12].copy_from_slice(&0x10000u32.to_le_bytes());
    data[8 + 0x14..8 + 0x1c].copy_from_slice(&0x8000_0000_0000_0001u64.to_le_bytes());
    assert_eq!(
        parse_terrain_height_chunks(&data).unwrap()[0].holes_high_res,
        Some(0x8000_0000_0000_0001)
    );
}
