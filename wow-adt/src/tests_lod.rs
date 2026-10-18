use super::{LodHeader, LodLevel, LodQuadTreeNode, fixtures::append_subchunk, load_lod_adt};
use crate::adt::LodLiquidDirectory;

const TEST_M2_LOD_FLAGS: u32 = 0x55AA;
const TEST_WMO_LOD_FLAGS: u32 = 0xAA55;

#[test]
fn load_lod_adt_reads_synthetic_lod_chunks() {
    let payload = synthetic_lod_payload();

    let lod = load_lod_adt(&payload).expect("expected synthetic _lod.adt to parse");

    assert_eq!(lod.version, 18);
    assert_eq!(
        lod.header,
        LodHeader {
            flags: 7,
            bounds_min: [10.0, 30.0, 50.0],
            bounds_max: [20.0, 40.0, 60.0],
        }
    );
    assert_eq!(lod.heights, vec![1.5, 2.5, 3.5, 4.5]);
    assert_eq!(
        lod.levels,
        vec![
            LodLevel {
                vertex_step: 32.0,
                payload: [0, 12, 4, 24],
            },
            LodLevel {
                vertex_step: 16.0,
                payload: [12, 24, 8, 48],
            },
        ]
    );
    assert_eq!(
        lod.nodes,
        vec![
            LodQuadTreeNode {
                words16: [1, 2, 3, 4],
                words32: [5, 6, 7],
            },
            LodQuadTreeNode {
                words16: [8, 9, 10, 11],
                words32: [12, 13, 14],
            },
        ]
    );
    assert_eq!(lod.indices, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(lod.skirt_indices, vec![6, 7, 8, 9]);
    assert_eq!(
        lod.liquid_directory,
        Some(LodLiquidDirectory {
            raw: vec![0xAA, 0xBB, 0xCC, 0xDD],
        })
    );
    assert_eq!(lod.liquids.len(), 1);
    assert_eq!(lod.liquids[0].header.words, [1, 2, 3, 4, 5, 6]);
    assert_eq!(lod.liquids[0].indices, vec![10, 11, 12]);
    assert_eq!(
        lod.liquids[0].vertices,
        vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
    );
    assert_eq!(lod.m2_placements.len(), 1);
    assert_eq!(lod.m2_placements[0].id, 100);
    assert_eq!(lod.m2_placements[0].asset_id, 200);
    assert_eq!(lod.m2_placements[0].position, [1.0, 2.0, 3.0]);
    assert_eq!(lod.m2_placements[0].rotation, [4.0, 5.0, 6.0]);
    assert_eq!(lod.m2_placements[0].scale, 1.5);
    assert_eq!(lod.m2_placements[0].flags, TEST_M2_LOD_FLAGS);
    assert_eq!(lod.m2_visibility.len(), 1);
    assert_eq!(lod.m2_visibility[0].bounds_min, [10.0, 20.0, 30.0]);
    assert_eq!(lod.m2_visibility[0].bounds_max, [40.0, 50.0, 60.0]);
    assert_eq!(lod.m2_visibility[0].radius, 70.0);
    assert_eq!(lod.wmo_placements.len(), 1);
    assert_eq!(lod.wmo_visibility.len(), 1);
}

fn synthetic_lod_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    append_synthetic_lod_chunks(&mut payload);
    payload
}

fn append_synthetic_lod_chunks(payload: &mut Vec<u8>) {
    append_synthetic_lod_core_chunks(payload);
    append_synthetic_lod_liquid_chunks(payload);
    append_synthetic_lod_object_chunks(payload);
}

fn append_synthetic_lod_core_chunks(payload: &mut Vec<u8>) {
    append_subchunk(payload, b"REVM", 18u32.to_le_bytes().to_vec());
    append_subchunk(
        payload,
        b"DHLM",
        lod_header_payload(7, [10.0, 30.0, 50.0], [20.0, 40.0, 60.0]),
    );
    append_subchunk(payload, b"HVLM", lod_height_payload());
    append_subchunk(payload, b"LLLM", lod_levels_payload());
    append_subchunk(payload, b"DNLM", lod_nodes_payload());
    append_subchunk(payload, b"IVLM", lod_index_payload(&[0, 1, 2, 3, 4, 5]));
    append_subchunk(payload, b"ISLM", lod_index_payload(&[6, 7, 8, 9]));
}

fn append_synthetic_lod_liquid_chunks(payload: &mut Vec<u8>) {
    append_subchunk(payload, b"DLLM", vec![0xAA, 0xBB, 0xCC, 0xDD]);
    append_subchunk(
        payload,
        b"NLLM",
        lod_liquid_header_payload([1, 2, 3, 4, 5, 6]),
    );
    append_subchunk(payload, b"ILLM", lod_index_payload(&[10, 11, 12]));
    append_subchunk(
        payload,
        b"VLLM",
        lod_liquid_vertices_payload(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]),
    );
}

fn append_synthetic_lod_object_chunks(payload: &mut Vec<u8>) {
    append_synthetic_lod_object_chunk(
        payload,
        SyntheticLodObjectChunk {
            placement_id: *b"DDLM",
            visibility_id: *b"XDLM",
            unique_id: 100,
            name_id: 200,
            position: [1.0, 2.0, 3.0],
            rotation: [4.0, 5.0, 6.0],
            scale: 1.5,
            flags: TEST_M2_LOD_FLAGS,
            bbox_min: [10.0, 20.0, 30.0],
            bbox_max: [40.0, 50.0, 60.0],
            radius: 70.0,
        },
    );
    append_synthetic_lod_object_chunk(
        payload,
        SyntheticLodObjectChunk {
            placement_id: *b"DMLM",
            visibility_id: *b"XMLM",
            unique_id: 300,
            name_id: 400,
            position: [7.0, 8.0, 9.0],
            rotation: [10.0, 11.0, 12.0],
            scale: 2.5,
            flags: TEST_WMO_LOD_FLAGS,
            bbox_min: [15.0, 25.0, 35.0],
            bbox_max: [45.0, 55.0, 65.0],
            radius: 75.0,
        },
    );
}

struct SyntheticLodObjectChunk {
    placement_id: [u8; 4],
    visibility_id: [u8; 4],
    unique_id: u32,
    name_id: u32,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: f32,
    flags: u32,
    bbox_min: [f32; 3],
    bbox_max: [f32; 3],
    radius: f32,
}

fn append_synthetic_lod_object_chunk(payload: &mut Vec<u8>, chunk: SyntheticLodObjectChunk) {
    append_subchunk(
        payload,
        &chunk.placement_id,
        lod_object_placement_payload(
            chunk.unique_id,
            chunk.name_id,
            chunk.position,
            chunk.rotation,
            chunk.scale,
            chunk.flags,
        ),
    );
    append_subchunk(
        payload,
        &chunk.visibility_id,
        lod_object_visibility_payload(chunk.bbox_min, chunk.bbox_max, chunk.radius),
    );
}

fn lod_height_payload() -> Vec<u8> {
    [1.5f32, 2.5, 3.5, 4.5]
        .into_iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn lod_header_payload(flags: u32, bounds_min: [f32; 3], bounds_max: [f32; 3]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&flags.to_le_bytes());
    payload.extend_from_slice(&bounds_min[0].to_le_bytes());
    payload.extend_from_slice(&bounds_max[0].to_le_bytes());
    payload.extend_from_slice(&bounds_min[1].to_le_bytes());
    payload.extend_from_slice(&bounds_max[1].to_le_bytes());
    payload.extend_from_slice(&bounds_min[2].to_le_bytes());
    payload.extend_from_slice(&bounds_max[2].to_le_bytes());
    payload
}

fn lod_levels_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    append_lod_level(&mut payload, 32.0, [0, 12, 4, 24]);
    append_lod_level(&mut payload, 16.0, [12, 24, 8, 48]);
    payload
}

fn append_lod_level(payload: &mut Vec<u8>, vertex_step: f32, values: [u32; 4]) {
    payload.extend_from_slice(&vertex_step.to_le_bytes());
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
}

fn lod_nodes_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    append_lod_node(&mut payload, [1, 2, 3, 4], [5, 6, 7]);
    append_lod_node(&mut payload, [8, 9, 10, 11], [12, 13, 14]);
    payload
}

fn append_lod_node(payload: &mut Vec<u8>, words16: [u16; 4], words32: [u32; 3]) {
    for value in words16 {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    for value in words32 {
        payload.extend_from_slice(&value.to_le_bytes());
    }
}

fn lod_index_payload(indices: &[u16]) -> Vec<u8> {
    indices
        .iter()
        .flat_map(|index| index.to_le_bytes())
        .collect()
}

fn lod_liquid_header_payload(words: [u32; 6]) -> Vec<u8> {
    words
        .into_iter()
        .flat_map(|word| word.to_le_bytes())
        .collect()
}

fn lod_liquid_vertices_payload(vertices: &[[f32; 3]]) -> Vec<u8> {
    let mut payload = Vec::new();
    for vertex in vertices {
        for component in vertex {
            payload.extend_from_slice(&component.to_le_bytes());
        }
    }
    payload
}

fn lod_object_placement_payload(
    id: u32,
    asset_id: u32,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: f32,
    flags: u32,
) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&id.to_le_bytes());
    payload.extend_from_slice(&asset_id.to_le_bytes());
    for component in position {
        payload.extend_from_slice(&component.to_le_bytes());
    }
    for component in rotation {
        payload.extend_from_slice(&component.to_le_bytes());
    }
    payload.extend_from_slice(&scale.to_le_bytes());
    payload.extend_from_slice(&flags.to_le_bytes());
    payload
}

fn lod_object_visibility_payload(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    radius: f32,
) -> Vec<u8> {
    let mut payload = Vec::new();
    for component in bounds_min {
        payload.extend_from_slice(&component.to_le_bytes());
    }
    for component in bounds_max {
        payload.extend_from_slice(&component.to_le_bytes());
    }
    payload.extend_from_slice(&radius.to_le_bytes());
    payload
}
