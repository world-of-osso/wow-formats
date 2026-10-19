use wow_adt::adt::parse_mfbo_min_heights;

fn encode_planes(max: [i16; 9], min: [i16; 9]) -> Vec<u8> {
    max.into_iter()
        .chain(min)
        .flat_map(i16::to_le_bytes)
        .collect()
}

#[test]
fn minimum_projection_reads_signed_lower_corners_in_authored_order() {
    let minimum = [i16::MIN, -1237, -1, 0, 1, 300, 1000, 32766, i16::MAX];
    let bytes = encode_planes([1700, 1800, -1, 0, i16::MAX, i16::MIN, 17, 18, 19], minimum);
    assert_eq!(parse_mfbo_min_heights(&bytes).unwrap(), minimum);
}

#[test]
fn minimum_projection_retains_first_failing_word_errors_without_a_full_record_guard() {
    let bytes = encode_planes([1700; 9], [-1000; 9]);
    for (length, expected) in [
        (0, "read_u16 out of bounds at 0x12"),
        (18, "read_u16 out of bounds at 0x12"),
        (19, "read_u16 out of bounds at 0x12"),
        (20, "read_u16 out of bounds at 0x14"),
        (35, "read_u16 out of bounds at 0x22"),
    ] {
        assert_eq!(
            parse_mfbo_min_heights(&bytes[..length]).unwrap_err(),
            expected,
            "length {length}"
        );
    }
}

#[test]
fn minimum_projection_does_not_validate_upper_plane_values_or_unused_tail() {
    let mut bytes = encode_planes([i16::MIN; 9], [-2000; 9]);
    bytes.extend_from_slice(&[0xde, 0xad, 0xbe]);
    assert_eq!(parse_mfbo_min_heights(&bytes).unwrap(), [-2000; 9]);
}
