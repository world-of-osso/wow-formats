use wow_m2::parser::{parent_animation_blacklist, parent_skeleton_fdid};

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.to_vec();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn skpd_reads_the_parent_file_id() {
    let mut payload = [0; 16];
    payload[8..12].copy_from_slice(&1_830_371u32.to_le_bytes());
    assert_eq!(
        parent_skeleton_fdid(&chunk(b"SKPD", &payload)).unwrap(),
        Some(1_830_371)
    );
    assert_eq!(
        parent_skeleton_fdid(&chunk(b"SKPD", &[0; 16])).unwrap(),
        None
    );
    assert_eq!(
        parent_skeleton_fdid(&chunk(b"SKL1", &[0; 16])).unwrap(),
        None
    );
    assert!(parent_skeleton_fdid(&chunk(b"SKPD", &[0; 11])).is_err());
}

#[test]
fn pabc_lists_only_banned_parent_animations() {
    let mut model = chunk(b"MD21", &[0; 8]);
    model.extend(chunk(b"PABC", &[1, 0, 2, 0, 0x2a, 1]));
    assert_eq!(parent_animation_blacklist(&model).unwrap(), [1, 2, 298]);
    assert!(parent_animation_blacklist(&chunk(b"PABC", &[1])).is_err());
    assert!(
        parent_animation_blacklist(&chunk(b"MD21", &[]))
            .unwrap()
            .is_empty()
    );
}
