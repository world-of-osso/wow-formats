use super::*;

#[test]
fn opacity_values_use_signed_fixed16() {
    let mut md20 = vec![0u8; 8];
    let emitter = [
        0, 0, 0, 0, 0, 0, 0, 0, // timestamps
        2, 0, 0, 0, // count
        0, 0, 0, 0, // offset placeholder
    ];
    let values_offset = md20.len();
    md20.extend_from_slice(&(-1_i16).to_le_bytes());
    md20.extend_from_slice(&(16384_i16).to_le_bytes());
    let mut emitter = emitter;
    emitter[12..16].copy_from_slice(&(values_offset as u32).to_le_bytes());

    let opacities = read_opacity_values(&md20, &emitter, 0);

    assert_eq!(opacities[0], 0.0);
    assert!((opacities[1] - (16384.0 / 32767.0)).abs() < 0.0001);
}

#[test]
fn color_keys_preserve_full_fake_animblock_timeline() {
    let mut md20 = vec![0u8; 96];
    let mut emitter = vec![0u8; 16];

    emitter[0..4].copy_from_slice(&(4u32).to_le_bytes());
    emitter[4..8].copy_from_slice(&(32u32).to_le_bytes());
    emitter[8..12].copy_from_slice(&(4u32).to_le_bytes());
    emitter[12..16].copy_from_slice(&(40u32).to_le_bytes());

    for (idx, time) in [0u16, 8192, 16384, 32767].into_iter().enumerate() {
        md20[32 + idx * 2..34 + idx * 2].copy_from_slice(&time.to_le_bytes());
    }
    let colors = [
        [1.0f32, 2.0, 3.0],
        [4.0f32, 5.0, 6.0],
        [7.0f32, 8.0, 9.0],
        [10.0f32, 11.0, 12.0],
    ];
    for (idx, color) in colors.into_iter().enumerate() {
        let base = 40 + idx * 12;
        md20[base..base + 4].copy_from_slice(&color[0].to_le_bytes());
        md20[base + 4..base + 8].copy_from_slice(&color[1].to_le_bytes());
        md20[base + 8..base + 12].copy_from_slice(&color[2].to_le_bytes());
    }

    let keys = read_color_keys(&md20, &emitter, 0);

    assert_eq!(keys.len(), 4);
    assert_eq!(keys[0], (0.0, [1.0, 2.0, 3.0]));
    assert!((keys[1].0 - (8192.0 / 32767.0)).abs() < 0.0001);
    assert_eq!(keys[1].1, [4.0, 5.0, 6.0]);
    assert!((keys[2].0 - (16384.0 / 32767.0)).abs() < 0.0001);
    assert_eq!(keys[2].1, [7.0, 8.0, 9.0]);
    assert_eq!(keys[3], (1.0, [10.0, 11.0, 12.0]));
}

#[test]
fn opacity_keys_preserve_full_fake_animblock_timeline() {
    let mut md20 = vec![0u8; 64];
    let mut emitter = vec![0u8; 16];

    emitter[0..4].copy_from_slice(&(4u32).to_le_bytes());
    emitter[4..8].copy_from_slice(&(32u32).to_le_bytes());
    emitter[8..12].copy_from_slice(&(4u32).to_le_bytes());
    emitter[12..16].copy_from_slice(&(40u32).to_le_bytes());

    for (idx, time) in [0u16, 8192, 16384, 32767].into_iter().enumerate() {
        md20[32 + idx * 2..34 + idx * 2].copy_from_slice(&time.to_le_bytes());
    }
    for (idx, value) in [0i16, 8192, 16384, 32767].into_iter().enumerate() {
        md20[40 + idx * 2..42 + idx * 2].copy_from_slice(&value.to_le_bytes());
    }

    let keys = read_opacity_keys(&md20, &emitter, 0);

    assert_eq!(keys.len(), 4);
    assert_eq!(keys[0], (0.0, 0.0));
    assert!((keys[1].0 - (8192.0 / 32767.0)).abs() < 0.0001);
    assert!((keys[1].1 - (8192.0 / 32767.0)).abs() < 0.0001);
    assert!((keys[2].0 - (16384.0 / 32767.0)).abs() < 0.0001);
    assert!((keys[2].1 - (16384.0 / 32767.0)).abs() < 0.0001);
    assert_eq!(keys[3], (1.0, 1.0));
}

#[test]
fn parses_head_tail_tracks_and_burst_multiplier() {
    let mut md20 = vec![0u8; 0x180];
    let mut emitter = vec![0u8; 0x178];

    let head_offset = 0x40usize;
    md20[head_offset..head_offset + 6].copy_from_slice(&[1, 0, 2, 0, 3, 0]);
    emitter[0x13C + 8..0x13C + 12].copy_from_slice(&(3u32).to_le_bytes());
    emitter[0x13C + 12..0x13C + 16].copy_from_slice(&(head_offset as u32).to_le_bytes());

    let tail_offset = 0x50usize;
    md20[tail_offset..tail_offset + 6].copy_from_slice(&[4, 0, 5, 0, 6, 0]);
    emitter[0x14C + 8..0x14C + 12].copy_from_slice(&(3u32).to_le_bytes());
    emitter[0x14C + 12..0x14C + 16].copy_from_slice(&(tail_offset as u32).to_le_bytes());

    emitter[0x170..0x174].copy_from_slice(&(1.75_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&emitter, &emitter).unwrap();
    fill_visual_values(&mut parsed, &md20, &emitter);

    assert_eq!(parsed.head_cell_track, [1, 2, 3]);
    assert_eq!(parsed.tail_cell_track, [4, 5, 6]);
    assert!((parsed.burst_multiplier - 1.75).abs() < 0.0001);
}

#[test]
fn parses_particle_type_and_head_or_tail() {
    let mut emitter = vec![0u8; 0x178];
    emitter[0x2A] = 1;
    emitter[0x2B] = 2;

    let parsed = parse_emitter_header(&emitter, &emitter).unwrap();

    assert_eq!(parsed.particle_type, 1);
    assert_eq!(parsed.head_or_tail, 2);
}

#[test]
fn parses_particle_and_recursion_model_filenames() {
    let mut md20 = vec![0u8; 0x80];
    let mut emitter = vec![0u8; 0x178];
    let particle_name = b"spells/torch_flame.m2\0";
    let recursion_name = b"spells/torch_child.m2\0";
    let particle_offset = 0x40usize;
    let recursion_offset = particle_offset + particle_name.len();
    md20[particle_offset..particle_offset + particle_name.len()].copy_from_slice(particle_name);
    md20[recursion_offset..recursion_offset + recursion_name.len()].copy_from_slice(recursion_name);

    emitter[0x18..0x1C].copy_from_slice(&(particle_name.len() as u32).to_le_bytes());
    emitter[0x1C..0x20].copy_from_slice(&(particle_offset as u32).to_le_bytes());
    emitter[0x20..0x24].copy_from_slice(&(recursion_name.len() as u32).to_le_bytes());
    emitter[0x24..0x28].copy_from_slice(&(recursion_offset as u32).to_le_bytes());

    let parsed = parse_emitter_header(&md20, &emitter).unwrap();

    assert_eq!(
        parsed.particle_model_filename.as_deref(),
        Some("spells/torch_flame.m2")
    );
    assert_eq!(
        parsed.child_emitters_model_filename.as_deref(),
        Some("spells/torch_child.m2")
    );
}

#[test]
fn particle_emitters_default_inherit_velocity_scale_to_identity() {
    let emitter = vec![0u8; 0x178];

    let parsed = parse_emitter_header(&emitter, &emitter).unwrap();

    assert!((parsed.inherit_velocity_scale - 1.0).abs() < 0.0001);
}

#[test]
fn parses_spin_fields_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_BASE_SPIN_OFFSET..EMITTER_BASE_SPIN_OFFSET + 4]
        .copy_from_slice(&(0.25_f32).to_le_bytes());
    md20[EMITTER_BASE_SPIN_VARIATION_OFFSET..EMITTER_BASE_SPIN_VARIATION_OFFSET + 4]
        .copy_from_slice(&(1.5_f32).to_le_bytes());
    md20[EMITTER_SPIN_OFFSET..EMITTER_SPIN_OFFSET + 4].copy_from_slice(&(0.75_f32).to_le_bytes());
    md20[EMITTER_SPIN_VARIATION_OFFSET..EMITTER_SPIN_VARIATION_OFFSET + 4]
        .copy_from_slice(&(0.5_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.base_spin - 0.25).abs() < 0.0001);
    assert!((parsed.base_spin_variation - 1.5).abs() < 0.0001);
    assert!((parsed.spin - 0.75).abs() < 0.0001);
    assert!((parsed.spin_variation - 0.5).abs() < 0.0001);
}

#[test]
fn parses_tail_length_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_TAIL_LENGTH_OFFSET..EMITTER_TAIL_LENGTH_OFFSET + 4]
        .copy_from_slice(&(2.5_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.tail_length - 2.5).abs() < 0.0001);
}

#[test]
fn parses_z_source_track_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    let mut emitter = vec![0u8; 0x1ec];
    let z_source_offset = 0x40usize;
    md20[z_source_offset..z_source_offset + 4].copy_from_slice(&(1.25_f32).to_le_bytes());
    emitter[EMITTER_Z_SOURCE_OFFSET + 12..EMITTER_Z_SOURCE_OFFSET + 16]
        .copy_from_slice(&(1u32).to_le_bytes());
    emitter[EMITTER_Z_SOURCE_OFFSET + 16..EMITTER_Z_SOURCE_OFFSET + 20]
        .copy_from_slice(&(z_source_offset as u32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &emitter).unwrap();
    fill_track_values(&mut parsed, &md20, &emitter);

    assert!((parsed.z_source - 1.25).abs() < 0.0001);
}

#[test]
fn parses_drag_from_272_suffix_float_slot() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_DRAG_OFFSET..EMITTER_DRAG_OFFSET + 4].copy_from_slice(&(0.35_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.drag - 0.35).abs() < 0.0001);
}

#[test]
fn parses_size_variation_fields_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_SCALE_VARIATION_OFFSET..EMITTER_SCALE_VARIATION_OFFSET + 4]
        .copy_from_slice(&(0.4_f32).to_le_bytes());
    md20[EMITTER_SCALE_VARIATION_Y_OFFSET..EMITTER_SCALE_VARIATION_Y_OFFSET + 4]
        .copy_from_slice(&(0.2_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.scale_variation - 0.4).abs() < 0.0001);
    assert!((parsed.scale_variation_y - 0.2).abs() < 0.0001);
}

#[test]
fn parses_lifespan_variation_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_LIFESPAN_VARIATION_OFFSET..EMITTER_LIFESPAN_VARIATION_OFFSET + 4]
        .copy_from_slice(&(0.4_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.lifespan_variation - 0.4).abs() < 0.0001);
}

#[test]
fn parses_emission_rate_variation_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_EMISSION_RATE_VARIATION_OFFSET..EMITTER_EMISSION_RATE_VARIATION_OFFSET + 4]
        .copy_from_slice(&(0.6_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.emission_rate_variation - 0.6).abs() < 0.0001);
}

#[test]
fn parses_uncompressed_gravity_as_negative_wow_z() {
    let mut md20 = vec![0u8; 0x1ec];
    let mut emitter = vec![0u8; 0x1ec];
    let gravity_offset = 0x40usize;
    md20[gravity_offset..gravity_offset + 4].copy_from_slice(&(2.0_f32).to_le_bytes());
    emitter[EMITTER_GRAVITY_OFFSET + 12..EMITTER_GRAVITY_OFFSET + 16]
        .copy_from_slice(&(1u32).to_le_bytes());
    emitter[EMITTER_GRAVITY_OFFSET + 16..EMITTER_GRAVITY_OFFSET + 20]
        .copy_from_slice(&(gravity_offset as u32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &emitter).unwrap();
    fill_track_values(&mut parsed, &md20, &emitter);

    assert!((parsed.gravity - 2.0).abs() < 0.0001);
    assert_eq!(parsed.gravity_vector, [0.0, 0.0, -2.0]);
}

#[test]
fn parses_compressed_gravity_vector() {
    let mut md20 = vec![0u8; 0x1ec];
    let mut emitter = vec![0u8; 0x1ec];
    emitter[EMITTER_FLAGS_OFFSET..EMITTER_FLAGS_OFFSET + 4]
        .copy_from_slice(&(0x0080_0000u32).to_le_bytes());
    let gravity_offset = 0x40usize;
    md20[gravity_offset..gravity_offset + 4].copy_from_slice(&[64, 0, 100, 0]);
    emitter[EMITTER_GRAVITY_OFFSET + 12..EMITTER_GRAVITY_OFFSET + 16]
        .copy_from_slice(&(1u32).to_le_bytes());
    emitter[EMITTER_GRAVITY_OFFSET + 16..EMITTER_GRAVITY_OFFSET + 20]
        .copy_from_slice(&(gravity_offset as u32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &emitter).unwrap();
    fill_track_values(&mut parsed, &md20, &emitter);

    assert!((parsed.gravity_vector[0] - 2.119324).abs() < 0.0001);
    assert!((parsed.gravity_vector[1] - 0.0).abs() < 0.0001);
    assert!((parsed.gravity_vector[2] - 3.670643).abs() < 0.0002);
}

#[test]
fn parses_twinkle_fields_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_TWINKLE_SPEED_OFFSET..EMITTER_TWINKLE_SPEED_OFFSET + 4]
        .copy_from_slice(&(2.0_f32).to_le_bytes());
    md20[EMITTER_TWINKLE_PERCENT_OFFSET..EMITTER_TWINKLE_PERCENT_OFFSET + 4]
        .copy_from_slice(&(0.75_f32).to_le_bytes());
    md20[EMITTER_TWINKLE_SCALE_MIN_OFFSET..EMITTER_TWINKLE_SCALE_MIN_OFFSET + 4]
        .copy_from_slice(&(0.5_f32).to_le_bytes());
    md20[EMITTER_TWINKLE_SCALE_MAX_OFFSET..EMITTER_TWINKLE_SCALE_MAX_OFFSET + 4]
        .copy_from_slice(&(1.5_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_visual_values(&mut parsed, &md20, &md20);

    assert!((parsed.twinkle_speed - 2.0).abs() < 0.0001);
    assert!((parsed.twinkle_percent - 0.75).abs() < 0.0001);
    assert!((parsed.twinkle_scale_min - 0.5).abs() < 0.0001);
    assert!((parsed.twinkle_scale_max - 1.5).abs() < 0.0001);
}

#[test]
fn parses_wind_fields_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_WIND_VECTOR_OFFSET..EMITTER_WIND_VECTOR_OFFSET + 4]
        .copy_from_slice(&(1.0_f32).to_le_bytes());
    md20[EMITTER_WIND_VECTOR_OFFSET + 4..EMITTER_WIND_VECTOR_OFFSET + 8]
        .copy_from_slice(&(2.0_f32).to_le_bytes());
    md20[EMITTER_WIND_VECTOR_OFFSET + 8..EMITTER_WIND_VECTOR_OFFSET + 12]
        .copy_from_slice(&(3.0_f32).to_le_bytes());
    md20[EMITTER_WIND_TIME_OFFSET..EMITTER_WIND_TIME_OFFSET + 4]
        .copy_from_slice(&(4.5_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert_eq!(parsed.wind_vector, [1.0, 2.0, 3.0]);
    assert!((parsed.wind_time - 4.5).abs() < 0.0001);
}

#[test]
fn parses_follow_fields_from_272_suffix() {
    let mut md20 = vec![0u8; 0x1ec];
    md20[EMITTER_FOLLOW_SPEED1_OFFSET..EMITTER_FOLLOW_SPEED1_OFFSET + 4]
        .copy_from_slice(&(2.0_f32).to_le_bytes());
    md20[EMITTER_FOLLOW_SCALE1_OFFSET..EMITTER_FOLLOW_SCALE1_OFFSET + 4]
        .copy_from_slice(&(0.25_f32).to_le_bytes());
    md20[EMITTER_FOLLOW_SPEED2_OFFSET..EMITTER_FOLLOW_SPEED2_OFFSET + 4]
        .copy_from_slice(&(6.0_f32).to_le_bytes());
    md20[EMITTER_FOLLOW_SCALE2_OFFSET..EMITTER_FOLLOW_SCALE2_OFFSET + 4]
        .copy_from_slice(&(0.75_f32).to_le_bytes());

    let mut parsed = parse_emitter_header(&md20, &md20).unwrap();
    fill_track_values(&mut parsed, &md20, &md20);

    assert!((parsed.follow_speed1 - 2.0).abs() < 0.0001);
    assert!((parsed.follow_scale1 - 0.25).abs() < 0.0001);
    assert!((parsed.follow_speed2 - 6.0).abs() < 0.0001);
    assert!((parsed.follow_scale2 - 0.75).abs() < 0.0001);
}

#[test]
fn scale_keys_preserve_full_fake_animblock_timeline() {
    let mut md20 = vec![0u8; 96];
    let mut emitter = vec![0u8; 16];

    emitter[0..4].copy_from_slice(&(4u32).to_le_bytes());
    emitter[4..8].copy_from_slice(&(32u32).to_le_bytes());
    emitter[8..12].copy_from_slice(&(4u32).to_le_bytes());
    emitter[12..16].copy_from_slice(&(40u32).to_le_bytes());

    for (idx, time) in [0u16, 8192, 16384, 32767].into_iter().enumerate() {
        md20[32 + idx * 2..34 + idx * 2].copy_from_slice(&time.to_le_bytes());
    }
    let scales = [[1.0f32, 2.0], [3.0f32, 4.0], [5.0f32, 6.0], [7.0f32, 8.0]];
    for (idx, scale) in scales.into_iter().enumerate() {
        let base = 40 + idx * 8;
        md20[base..base + 4].copy_from_slice(&scale[0].to_le_bytes());
        md20[base + 4..base + 8].copy_from_slice(&scale[1].to_le_bytes());
    }

    let keys = read_scale_keys(&md20, &emitter, 0);

    assert_eq!(keys.len(), 4);
    assert_eq!(keys[0], (0.0, [1.0, 2.0]));
    assert!((keys[1].0 - (8192.0 / 32767.0)).abs() < 0.0001);
    assert_eq!(keys[1].1, [3.0, 4.0]);
    assert!((keys[2].0 - (16384.0 / 32767.0)).abs() < 0.0001);
    assert_eq!(keys[2].1, [5.0, 6.0]);
    assert_eq!(keys[3], (1.0, [7.0, 8.0]));
}
