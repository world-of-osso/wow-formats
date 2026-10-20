use std::{fs, path::PathBuf, process::Command};

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = tag.iter().rev().copied().collect();
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

fn mcnk(index: (u32, u32), origin: [f32; 3], subchunks: &[u8]) -> Vec<u8> {
    let mut header = vec![0; 128];
    header[..4].copy_from_slice(&0x10000u32.to_le_bytes());
    header[4..8].copy_from_slice(&index.0.to_le_bytes());
    header[8..12].copy_from_slice(&index.1.to_le_bytes());
    header[0x14..0x1c].copy_from_slice(&0x8000_0000_0000_0001u64.to_le_bytes());
    header[0x34..0x38].copy_from_slice(&5u32.to_le_bytes());
    header[0x3c..0x3e].copy_from_slice(&0x21u16.to_le_bytes());
    for (slot, value) in origin.iter().enumerate() {
        header[0x68 + slot * 4..0x6c + slot * 4].copy_from_slice(&value.to_le_bytes());
    }
    header.extend_from_slice(subchunks);
    chunk(b"MCNK", &header)
}

fn heights(first: f32) -> Vec<u8> {
    let mut values = vec![0.25f32; 145];
    values[0] = first;
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn dump(name: &str, data: &[u8]) -> String {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, data).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_wow-formats-dump"))
        .arg("adt-height")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn rest_of_heights() -> String {
    vec!["0.25"; 144].join(",")
}

#[test]
fn projects_mcnk_header_and_every_mcvt_with_exact_widened_floats() {
    let mut sub = chunk(b"MCNR", &[0; 3]);
    sub.extend(chunk(b"MCVT", &heights(0.1)));
    sub.extend(chunk(b"MCVT", &heights(0.25)[..579]));
    let mut data = chunk(b"MVER", &18u32.to_le_bytes());
    data.extend(mcnk((3, 7), [-0.0, 1.5, 0.1], &sub));
    let expected = format!(
        concat!(
            r#"{{"mcnk":[{{"offset":12,"bytes":{},"index":[3,7],"origin":[-0.0,1.5,0.10000000149011612],"#,
            r#""area_id":5,"holes_low_res":33,"holes_high_res":9223372036854775809,"#,
            r#""mcvt":[{{"offset":11,"bytes":580,"heights":[0.10000000149011612,{}]}},"#,
            r#"{{"offset":599,"bytes":579,"heights":null}}],"truncated":null,"tail":0}}],"#,
            r#""truncated":null,"tail":0}}"#,
            "\n"
        ),
        128 + sub.len(),
        rest_of_heights()
    );
    assert_eq!(dump("projection.adt", &data), expected);
}

#[test]
fn non_finite_values_use_python_json_tokens() {
    let mut values = heights(f32::NAN);
    values[4..8].copy_from_slice(&f32::NEG_INFINITY.to_le_bytes());
    let data = mcnk((0, 0), [f32::INFINITY, 0.0, 0.0], &chunk(b"MCVT", &values));
    let json = dump("non-finite.adt", &data);
    assert!(json.contains(r#""origin":[Infinity,0.0,0.0]"#), "{json}");
    assert!(json.contains(r#""heights":[NaN,-Infinity,0.25"#), "{json}");
}

#[test]
fn framing_problems_are_reported_in_file_order_not_fatal() {
    let short = chunk(b"MCNK", &[0; 127]);
    let mut sub = chunk(b"MCVT", &heights(2.0));
    sub.extend_from_slice(b"XY");
    let good = mcnk((1, 2), [0.0; 3], &sub);
    let mut data = [short.clone(), good.clone()].concat();
    data.extend_from_slice(&chunk(b"MCNK", &[0; 16])[..12]);
    let json = dump("framing.adt", &data);
    let good_offset = short.len();
    let truncated_offset = good_offset + good.len();
    assert!(
        json.starts_with(r#"{"mcnk":[{"offset":0,"bytes":127},"#),
        "{json}"
    );
    assert!(
        json.contains(&format!(
            r#"{{"offset":{good_offset},"bytes":{},"#,
            128 + sub.len()
        )),
        "{json}"
    );
    assert!(json.contains(r#"],"truncated":null,"tail":2}"#), "{json}");
    assert!(
        json.ends_with(&format!(
            "\"truncated\":{{\"offset\":{truncated_offset},\"tag\":[77, 67, 78, 75]}},\"tail\":0}}\n"
        )),
        "{json}"
    );
}

#[test]
fn unreadable_file_and_bad_usage_fail_explicitly() {
    let missing = Command::new(env!("CARGO_BIN_EXE_wow-formats-dump"))
        .args(["adt-height", "/nonexistent/7.adt"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("/nonexistent/7.adt"));
    let usage = Command::new(env!("CARGO_BIN_EXE_wow-formats-dump"))
        .arg("wmo")
        .output()
        .unwrap();
    assert_eq!(usage.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&usage.stderr).contains("usage"));
}
