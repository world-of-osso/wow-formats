use wow_adt::adt_tex::{
    parse_water_attributes, read_liquid_instance_header, read_mh2o_chunk_header,
};

#[test]
fn chunk_projection_reads_only_its_twelve_authored_bytes() {
    let bytes: Vec<u8> = [0xfedc_ba98u32, 0, 0x1234_5678]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    let header = read_mh2o_chunk_header(&bytes, 0).unwrap();
    assert_eq!(header.instance_offset, 0xfedc_ba98);
    assert_eq!(header.layer_count, 0);
    assert_eq!(header.attributes_offset, 0x1234_5678);
    assert!(read_mh2o_chunk_header(&bytes[..11], 0).is_err());
}

#[test]
fn chunk_projection_keeps_directory_index_distinct_from_byte_offset() {
    let mut bytes = vec![0; 24];
    bytes[12..16].copy_from_slice(&3072u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&2u32.to_le_bytes());
    let header = read_mh2o_chunk_header(&bytes, 1).unwrap();
    assert_eq!(header.instance_offset, 3072);
    assert_eq!(header.layer_count, 2);
    assert_eq!(header.attributes_offset, 0);
}

#[test]
fn instance_projection_preserves_raw_float_bits_offsets_and_rectangle() {
    let mut bytes = vec![0xcc; 3];
    bytes.extend_from_slice(&5u16.to_le_bytes());
    bytes.extend_from_slice(&42u16.to_le_bytes());
    bytes.extend_from_slice(&(-0.0f32).to_bits().to_le_bytes());
    bytes.extend_from_slice(&0x7fc0_0025u32.to_le_bytes());
    bytes.extend_from_slice(&[2, 1, 3, 2]);
    bytes.extend_from_slice(&0x1122_3344u32.to_le_bytes());
    bytes.extend_from_slice(&0xfedc_ba98u32.to_le_bytes());
    let header = read_liquid_instance_header(&bytes, 3).unwrap();
    assert_eq!((header.liquid_type, header.liquid_object), (5, 42));
    assert_eq!(header.min_height.to_bits(), (-0.0f32).to_bits());
    assert_eq!(header.max_height.to_bits(), 0x7fc0_0025);
    assert_eq!(
        [
            header.x_offset,
            header.y_offset,
            header.width,
            header.height
        ],
        [2, 1, 3, 2]
    );
    assert_eq!(
        (header.exists_offset, header.vertex_offset),
        (0x1122_3344, 0xfedc_ba98)
    );
    assert_eq!(
        read_liquid_instance_header(&bytes[..26], 3).err().unwrap(),
        "SLiquidInstance out of bounds at 0x3 (payload len 0x1a)"
    );
}

#[test]
fn attribute_projection_keeps_both_64_bit_masks_and_requires_sixteen_bytes() {
    let mut bytes = vec![0xcc; 5];
    bytes.extend_from_slice(&0x0123_4567_89ab_cdefu64.to_le_bytes());
    bytes.extend_from_slice(&0x8000_0000_0000_0001u64.to_le_bytes());
    let attributes = parse_water_attributes(&bytes, 5).unwrap();
    assert_eq!(attributes.fishable, 0x0123_4567_89ab_cdef);
    assert_eq!(attributes.deep, 0x8000_0000_0000_0001);
    assert!(parse_water_attributes(&bytes[..20], 5).is_err());
}
