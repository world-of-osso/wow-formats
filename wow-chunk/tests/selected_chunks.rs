use wow_chunk::find_reversed_chunk;

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.iter().rev().copied().collect::<Vec<_>>();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn selected_chunk_is_first_match_and_does_not_validate_unvisited_tail() {
    let mut bytes = chunk(b"MOHD", &[1, 2, 3]);
    bytes.extend(chunk(b"MOHD", &[4, 5]));
    bytes.extend_from_slice(b"TMOM\x10\0\0\0xx");
    assert_eq!(
        find_reversed_chunk(&bytes, b"MOHD").unwrap(),
        Some(&[1, 2, 3][..])
    );
    assert_eq!(find_reversed_chunk(&[], b"MOHD").unwrap(), None);
    assert_eq!(find_reversed_chunk(b"MOHD", b"MOHD").unwrap(), None);
}

#[test]
fn selected_chunk_reports_documented_tag_on_truncated_payload() {
    let mut bytes = chunk(b"MOHD", &[]);
    bytes.extend_from_slice(b"LIAF\x10\0\0\0xx");
    assert_eq!(
        find_reversed_chunk(&bytes, b"MAID").unwrap_err(),
        "FAIL chunk truncated"
    );
}
