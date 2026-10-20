//! Print wow-formats projections as one JSON line for project tools.
//!
//! Callers keep selection, validation and error policy: framing problems are
//! reported as data instead of failing the dump. Floats are the authored f32
//! values widened to f64; non-finite values use Python's `NaN`/`Infinity`
//! JSON extension so they round-trip exactly.

use std::{env, fs, process::ExitCode};

use wow_adt::adt::{TerrainHeightHeader, parse_mcvt, parse_terrain_height_header};
use wow_chunk::ChunkIter;

const USAGE: &str = "usage: wow-formats-dump adt-height <root.adt>";
const MCNK_HEADER_BYTES: usize = 128;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let [command, path] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    if command != "adt-height" {
        eprintln!("unknown command {command:?}; {USAGE}");
        return ExitCode::from(2);
    }
    let result = fs::read(path)
        .map_err(|error| error.to_string())
        .and_then(|data| adt_height_json(&data));
    match result {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("wow-formats-dump: {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Top-level chunks in file order, the first truncated frame, and unframed trailing bytes.
struct Frames<'a> {
    chunks: Vec<(usize, [u8; 4], &'a [u8])>,
    truncated: Option<(usize, [u8; 4])>,
    tail: usize,
}

fn read_frames(data: &[u8]) -> Frames<'_> {
    let mut chunks = Vec::new();
    let mut offset = 0;
    for chunk in ChunkIter::new(data) {
        let Ok((stored, payload)) = chunk else {
            let stored = data[offset..offset + 4].try_into().unwrap();
            return Frames {
                chunks,
                truncated: Some((offset, forward_tag(stored))),
                tail: 0,
            };
        };
        chunks.push((offset, forward_tag(*stored), payload));
        offset += 8 + payload.len();
    }
    Frames {
        chunks,
        truncated: None,
        tail: data.len() - offset,
    }
}

fn forward_tag(stored: [u8; 4]) -> [u8; 4] {
    [stored[3], stored[2], stored[1], stored[0]]
}

fn adt_height_json(data: &[u8]) -> Result<String, String> {
    let frames = read_frames(data);
    let mcnks = frames
        .chunks
        .iter()
        .filter(|(_, tag, _)| tag == b"MCNK")
        .map(|(offset, _, payload)| mcnk_json(*offset, payload))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!(
        r#"{{"mcnk":[{}],{}}}"#,
        mcnks.join(","),
        frame_status_json(&frames)
    ))
}

fn mcnk_json(offset: usize, payload: &[u8]) -> Result<String, String> {
    let bytes = payload.len();
    if bytes < MCNK_HEADER_BYTES {
        return Ok(format!(r#"{{"offset":{offset},"bytes":{bytes}}}"#));
    }
    let header = parse_terrain_height_header(payload)?;
    let subchunks = read_frames(&payload[MCNK_HEADER_BYTES..]);
    let mcvt: Vec<String> = subchunks
        .chunks
        .iter()
        .filter(|(_, tag, _)| tag == b"MCVT")
        .map(|(offset, _, payload)| mcvt_json(*offset, payload))
        .collect();
    Ok(format!(
        r#"{{"offset":{offset},"bytes":{bytes},{},"mcvt":[{}],{}}}"#,
        header_json(&header),
        mcvt.join(","),
        frame_status_json(&subchunks)
    ))
}

fn header_json(header: &TerrainHeightHeader) -> String {
    let holes_high_res = header
        .holes_high_res
        .map_or_else(|| "null".to_string(), |mask| mask.to_string());
    format!(
        r#""index":[{},{}],"origin":{},"area_id":{},"holes_low_res":{},"holes_high_res":{holes_high_res}"#,
        header.index_x,
        header.index_y,
        floats_json(&header.origin),
        header.area_id,
        header.holes_low_res,
    )
}

/// Every MCVT occurrence in order; heights only when the payload holds all 145 values.
fn mcvt_json(offset: usize, payload: &[u8]) -> String {
    let heights = parse_mcvt(payload).map_or_else(|_| "null".to_string(), |h| floats_json(&h));
    format!(
        r#"{{"offset":{offset},"bytes":{},"heights":{heights}}}"#,
        payload.len()
    )
}

fn frame_status_json(frames: &Frames) -> String {
    let truncated = frames.truncated.map_or_else(
        || "null".to_string(),
        |(offset, tag)| format!(r#"{{"offset":{offset},"tag":{tag:?}}}"#),
    );
    format!(r#""truncated":{truncated},"tail":{}"#, frames.tail)
}

fn floats_json(values: &[f32]) -> String {
    let items: Vec<String> = values.iter().map(|&value| float_json(value)).collect();
    format!("[{}]", items.join(","))
}

fn float_json(value: f32) -> String {
    let value = f64::from(value);
    if value.is_nan() {
        "NaN".to_string()
    } else if value.is_infinite() {
        if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
    } else {
        // Debug prints the shortest representation that round-trips the f64.
        format!("{value:?}")
    }
}
