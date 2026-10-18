use super::*;

#[test]
fn a7_truncated_activation_track_is_rejected_in_md20_and_ska1() {
    let mut md20 = vec![0; 0x100 + 39];
    md20[0xf0..0xf4].copy_from_slice(&1_u32.to_le_bytes());
    md20[0xf4..0xf8].copy_from_slice(&0x100_u32.to_le_bytes());
    assert!(
        parse_attachments(&md20).is_err(),
        "truncated byte track was discarded"
    );
    let mut ska1 = vec![0; 16 + 39];
    ska1[0..4].copy_from_slice(&1_u32.to_le_bytes());
    ska1[4..8].copy_from_slice(&16_u32.to_le_bytes());
    assert!(
        parse_ska1_attachments(&ska1).is_err(),
        "SKA1 truncated track was discarded"
    );
}

fn activation_fixture(ska1: bool) -> Vec<u8> {
    let mut bytes = vec![0; 400];
    let header = if ska1 { 0 } else { 0xf0 };
    bytes[header..header + 4].copy_from_slice(&1_u32.to_le_bytes());
    bytes[header + 4..header + 8].copy_from_slice(&256_u32.to_le_bytes());
    bytes[256..260].copy_from_slice(&19_u32.to_le_bytes());
    bytes[278..280].copy_from_slice(&(-1_i16).to_le_bytes());
    for (offset, value) in [
        (280, 2),
        (284, 320),
        (288, 2),
        (292, 336),
        (320, 3),
        (324, 352),
        (328, 1),
        (332, 368),
        (336, 3),
        (340, 364),
        (344, 1),
        (348, 372),
        (352, 0),
        (356, 100),
        (360, 200),
        (368, 0),
    ] {
        bytes[offset..offset + 4].copy_from_slice(&u32::to_le_bytes(value));
    }
    bytes[364..367].copy_from_slice(&[1, 0, 1]);
    bytes[372] = 1;
    bytes
}

#[test]
fn a7_activation_keys_sequence_switch_global_and_missing_controls() {
    for ska1 in [false, true] {
        let bytes = activation_fixture(ska1);
        let mut attachments = if ska1 {
            parse_ska1_attachments(&bytes)
        } else {
            parse_attachments(&bytes)
        }
        .unwrap();
        let attachment = &mut attachments[0];
        assert_eq!(attachment.id, 19);
        for (time, active) in [
            (0, true),
            (99, true),
            (100, false),
            (199, false),
            (200, true),
            (250, true),
        ] {
            assert_eq!(
                attachment.active_at(0, time, &[], 0.0),
                active,
                "SKA1={ska1}, time={time}"
            );
            assert!(
                attachment.active_at(1, time, &[], 0.0),
                "always-active sequence switch"
            );
        }
        attachment.activation.global_sequence = 0;
        assert!(
            !attachment.active_at(1, 0, &[300], 100.0),
            "global ignores local sequence/time"
        );
        assert!(
            attachment.active_at(1, 100, &[300], 300.0),
            "global wraps to active0"
        );
        attachment.activation.global_sequence = -1;
        attachment.activation.sequences.clear();
        assert!(
            attachment.active_at(0, 100, &[], 0.0),
            "missing track starts active1"
        );
    }
}

#[test]
fn root_attachment_arrays_are_distinct_from_collision_arrays() {
    let mut md20 = vec![0; 0x200];
    md20[..4].copy_from_slice(b"MD20");
    md20[4..8].copy_from_slice(&274_u32.to_le_bytes());
    md20[0xd8..0xdc].copy_from_slice(&1_u32.to_le_bytes());
    md20[0xdc..0xe0].copy_from_slice(&0x140_u32.to_le_bytes());
    md20[0xe0..0xe4].copy_from_slice(&1_u32.to_le_bytes());
    md20[0xe4..0xe8].copy_from_slice(&0x142_u32.to_le_bytes());
    md20[0xf0..0xf4].copy_from_slice(&1_u32.to_le_bytes());
    md20[0xf4..0xf8].copy_from_slice(&0x180_u32.to_le_bytes());
    md20[0xf8..0xfc].copy_from_slice(&2_u32.to_le_bytes());
    md20[0xfc..0x100].copy_from_slice(&0x1a8_u32.to_le_bytes());
    md20[0x180..0x184].copy_from_slice(&19_u32.to_le_bytes());
    md20[0x184..0x186].copy_from_slice(&7_u16.to_le_bytes());
    for (index, coordinate) in [1.0_f32, -2.0, 3.5].into_iter().enumerate() {
        let offset = 0x188 + index * 4;
        md20[offset..offset + 4].copy_from_slice(&coordinate.to_le_bytes());
    }
    md20[0x1a8..0x1aa].copy_from_slice(&(-1_i16).to_le_bytes());
    md20[0x1aa..0x1ac].copy_from_slice(&0_i16.to_le_bytes());
    let attachments = parse_attachments(&md20).unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].id, 19);
    assert_eq!(attachments[0].bone, 7);
    assert_eq!(attachments[0].position, [1.0, -2.0, 3.5]);
    assert_eq!(parse_attachment_lookup(&md20).unwrap(), [-1, 0]);
}
