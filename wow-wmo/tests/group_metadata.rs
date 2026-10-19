use wow_wmo::parser::parse_root_group_file_ids;

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.iter().rev().copied().collect::<Vec<_>>();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn projected_group_list_needs_only_the_mohd_count_and_selected_gfid_entries() {
    let mut mohd = [0; 8];
    mohd[4..8].copy_from_slice(&2u32.to_le_bytes());
    let mut root = chunk(b"MOHD", &mohd);
    root.extend(chunk(b"GFID", &[17, 0, 0, 0, 23, 0, 0, 0, 31, 0, 0, 0]));
    root.extend_from_slice(b"TMOM\x10\0\0\0xx");
    assert_eq!(parse_root_group_file_ids(&root).unwrap(), [17, 23]);
}

#[test]
fn projected_group_list_preserves_missing_and_truncated_metadata_errors() {
    assert_eq!(
        parse_root_group_file_ids(&[]).unwrap_err(),
        "WMO root has no MOHD"
    );
    let mut mohd = [0; 8];
    mohd[4..8].copy_from_slice(&2u32.to_le_bytes());
    let mut root = chunk(b"MOHD", &mohd);
    assert_eq!(
        parse_root_group_file_ids(&root).unwrap_err(),
        "WMO root has no GFID"
    );
    root.extend(chunk(b"GFID", &17u32.to_le_bytes()));
    assert_eq!(
        parse_root_group_file_ids(&root).unwrap_err(),
        "read_u32 out of bounds at 0x4"
    );
}
