use wow_adt::wdt::{parse_wdt_maid_file_ids, parse_wdt_tiles};

fn append(data: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
    data.extend_from_slice(tag);
    data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend_from_slice(payload);
}

#[test]
fn raw_maid_slots_remain_available_independently_of_main_activity() {
    let mut maid = vec![0; 4096 * 32];
    maid[65 * 32..65 * 32 + 4].copy_from_slice(&123u32.to_le_bytes());
    maid[65 * 32 + 4..65 * 32 + 8].copy_from_slice(&124u32.to_le_bytes());
    let mut data = Vec::new();
    append(&mut data, b"NIAM", &vec![0; 4096 * 8]);
    append(&mut data, b"DIAM", &maid);
    let raw = parse_wdt_maid_file_ids(&data).unwrap().unwrap();
    assert_eq!(raw.len(), 4096);
    assert_eq!((raw[65].root, raw[65].obj0), (123, 124));
    assert_eq!(raw[64].root, 0);
    assert!(parse_wdt_tiles(&data).unwrap().active.is_empty());
    assert!(parse_wdt_maid_file_ids(&[]).unwrap().is_none());
}

#[test]
fn truncated_maid_is_rejected() {
    let mut data = Vec::new();
    append(&mut data, b"DIAM", &[0; 31]);
    assert!(parse_wdt_maid_file_ids(&data).is_err());
}
