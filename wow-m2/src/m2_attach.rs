//! M2 attachment point parser.
//!
//! MD20 header M2Array offsets:
//!   0xD8–0xEF: collision indices, vertices, normals (three M2Arrays)
//!   0xF0: attachments (count + offset) → M2Attachment[n] (40 bytes each)
//!   0xF8: attachment_lookup (count + offset) → i16[n]
//!
//! M2Attachment layout (40 bytes):
//!   0x00: id (u32) — attachment lookup ID
//!   0x04: bone (u16) — bone index
//!   0x06: unknown (u16)
//!   0x08: position [f32; 3] — offset relative to bone, WoW coordinates
//!   0x14: activation byte AnimBlock (20 bytes)

#[cfg(feature = "animation")]
use super::read_f32;
use super::{
    MD20_ATTACHMENT_LOOKUP_COUNT_OFFSET, MD20_ATTACHMENTS_COUNT_OFFSET, read_u16, read_u32,
};
use wow_chunk::read_m2_array_header;

const ATTACHMENT_ENTRY_SIZE: usize = 40;
#[cfg(feature = "animation")]
const ATTACHMENT_ACTIVATION_OFFSET: usize = 20;
#[cfg(feature = "animation")]
const ATTACHMENT_ID_OFFSET: usize = 0;
#[cfg(feature = "animation")]
const ATTACHMENT_BONE_OFFSET: usize = 4;
const ATTACHMENT_POSITION_OFFSET: usize = 8;
const ATTACHMENT_LOOKUP_ENTRY_SIZE: usize = 2;

/// An attachment point on an M2 model (e.g., hand, back, shoulder).
#[cfg(feature = "animation")]
#[derive(Debug, Clone)]
pub struct M2Attachment {
    /// Attachment lookup ID (0=LeftWrist, 1=RightPalm, 2=LeftPalm, etc.).
    pub id: u32,
    /// Index of the bone this attachment is parented to.
    pub bone: u16,
    /// Position offset relative to the bone, in WoW coordinates.
    pub position: [f32; 3],
    /// Authored child animation/callback/shadow eligibility; absent samples are active.
    pub activation: super::m2_anim::AnimTrack<u8>,
}

#[cfg(feature = "animation")]
impl M2Attachment {
    pub fn active_at(
        &self,
        sequence: usize,
        time_ms: u32,
        globals: &[u32],
        global_ms: f64,
    ) -> bool {
        let (timeline, time) =
            super::m2_anim::global_track_time(&self.activation, globals, global_ms)
                .unwrap_or((sequence, time_ms));
        super::m2_anim::evaluate_u8_track(&self.activation, timeline, time).unwrap_or(1) != 0
    }
}

/// Parse M2Attachment entries from MD20 offset 0xF0.
#[cfg(feature = "animation")]
pub fn parse_attachments(md20: &[u8]) -> Result<Vec<M2Attachment>, String> {
    let (count, offset) = read_m2_array_header(md20, MD20_ATTACHMENTS_COUNT_OFFSET)?;
    let mut attachments = Vec::with_capacity(count);
    for i in 0..count {
        let base = offset + i * ATTACHMENT_ENTRY_SIZE;
        if base + ATTACHMENT_ENTRY_SIZE > md20.len() {
            return Err(format!("Attachment {i} out of bounds at {base:#x}"));
        }
        attachments.push(M2Attachment {
            id: read_u32(md20, base + ATTACHMENT_ID_OFFSET)?,
            bone: read_u16(md20, base + ATTACHMENT_BONE_OFFSET)?,
            position: [
                read_f32(md20, base + ATTACHMENT_POSITION_OFFSET)?,
                read_f32(md20, base + ATTACHMENT_POSITION_OFFSET + 4)?,
                read_f32(md20, base + ATTACHMENT_POSITION_OFFSET + 8)?,
            ],
            activation: super::m2_anim::parse_u8_track(md20, base + ATTACHMENT_ACTIVATION_OFFSET)?,
        });
    }
    Ok(attachments)
}

#[cfg(feature = "animation")]
pub fn parse_ska1_attachments(ska1: &[u8]) -> Result<Vec<M2Attachment>, String> {
    if ska1.len() < 16 {
        return Ok(Vec::new());
    }
    let count = read_u32(ska1, 0)? as usize;
    let offset = read_u32(ska1, 4)? as usize;
    let mut attachments = Vec::with_capacity(count);
    for i in 0..count {
        let base = offset + i * ATTACHMENT_ENTRY_SIZE;
        if base + ATTACHMENT_ENTRY_SIZE > ska1.len() {
            return Err(format!("SKA1 attachment {i} out of bounds at {base:#x}"));
        }
        attachments.push(M2Attachment {
            id: read_u32(ska1, base + ATTACHMENT_ID_OFFSET)?,
            bone: read_u16(ska1, base + ATTACHMENT_BONE_OFFSET)?,
            position: [
                read_f32(ska1, base + ATTACHMENT_POSITION_OFFSET)?,
                read_f32(ska1, base + ATTACHMENT_POSITION_OFFSET + 4)?,
                read_f32(ska1, base + ATTACHMENT_POSITION_OFFSET + 8)?,
            ],
            activation: super::m2_anim::parse_u8_track(ska1, base + ATTACHMENT_ACTIVATION_OFFSET)?,
        });
    }
    Ok(attachments)
}

/// Parse attachment lookup table from MD20 offset 0xF8 (array of i16).
pub fn parse_attachment_lookup(md20: &[u8]) -> Result<Vec<i16>, String> {
    let (count, offset) = read_m2_array_header(md20, MD20_ATTACHMENT_LOOKUP_COUNT_OFFSET)?;
    let mut lookup = Vec::with_capacity(count);
    for i in 0..count {
        let off = offset + i * ATTACHMENT_LOOKUP_ENTRY_SIZE;
        if off + 2 > md20.len() {
            break;
        }
        lookup.push(read_u16(md20, off)? as i16);
    }
    Ok(lookup)
}

pub fn parse_ska1_attachment_lookup(ska1: &[u8]) -> Result<Vec<i16>, String> {
    if ska1.len() < 16 {
        return Ok(Vec::new());
    }
    let count = read_u32(ska1, 8)? as usize;
    let offset = read_u32(ska1, 12)? as usize;
    let mut lookup = Vec::with_capacity(count);
    for i in 0..count {
        let off = offset + i * ATTACHMENT_LOOKUP_ENTRY_SIZE;
        if off + 2 > ska1.len() {
            break;
        }
        lookup.push(read_u16(ska1, off)? as i16);
    }
    Ok(lookup)
}

/// A static attachment offset selected through the MD20 attachment lookup.
/// Does not read activation tracks, which cannot affect the authored seat position.
pub fn attachment_position(md20: &[u8], attachment: u32) -> Result<[f32; 3], String> {
    let (count, lookup) = read_m2_array_header(md20, MD20_ATTACHMENT_LOOKUP_COUNT_OFFSET)?;
    if attachment as usize >= count {
        return Err("M2 seat attachment not in lookup".into());
    }
    let index = wow_chunk::read_i16(
        md20,
        lookup + attachment as usize * ATTACHMENT_LOOKUP_ENTRY_SIZE,
    )?;
    let index = usize::try_from(index).map_err(|_| "M2 seat attachment absent")?;
    let (count, offset) = read_m2_array_header(md20, MD20_ATTACHMENTS_COUNT_OFFSET)?;
    if index >= count {
        return Err("M2 attachment index out of range".into());
    }
    wow_chunk::read_vec3(
        md20,
        offset + index * ATTACHMENT_ENTRY_SIZE + ATTACHMENT_POSITION_OFFSET,
    )
}

#[cfg(all(test, feature = "animation"))]
#[path = "../tests/unit/m2_attach_tests.rs"]
mod tests;
