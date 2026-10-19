//! WDT MPHD header flags that select how a map's `_tex0` terrain textures are stored and blended,
//! and the single WMO of a map made of one (a WMO-only dungeon such as the Stockade).

use std::collections::{BTreeMap, BTreeSet};

use wow_chunk::read_u32;

use super::adt::ChunkIter;
use super::adt_obj::{WmoPlacement, parse_modf};

/// wowdev WDT: `wdt_uses_global_map_obj`, the map is the one WMO its MODF places.
pub const MPHD_FLAG_GLOBAL_WMO: u32 = 0x1;

/// wowdev WDT: `adt_has_big_alpha`, "shader = 2".
const MPHD_FLAG_BIG_ALPHA: u32 = 0x4;
/// wowdev WDT: `adt_has_height_texturing`, "shader = 6"; also switches uncompressed MCAL to 4096 bytes.
const MPHD_FLAG_HEIGHT_TEXTURING: u32 = 0x80;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MphdFlags {
    pub raw: u32,
}

impl MphdFlags {
    /// Uncompressed MCAL layers are 4096 8-bit bytes instead of 2048 4-bit bytes.
    pub fn big_alpha(&self) -> bool {
        (self.raw & (MPHD_FLAG_BIG_ALPHA | MPHD_FLAG_HEIGHT_TEXTURING)) != 0
    }

    /// Terrain layers blend with `_h` height textures scaled by MTXP.
    pub fn height_texturing(&self) -> bool {
        (self.raw & MPHD_FLAG_HEIGHT_TEXTURING) != 0
    }
}

/// Flags in one selected MPHD payload; chunk selection and missing-chunk policy are separate.
pub fn parse_mphd_flags(payload: &[u8]) -> Result<MphdFlags, String> {
    Ok(MphdFlags {
        raw: read_u32(payload, 0)?,
    })
}

pub fn parse_wdt_mphd_flags(data: &[u8]) -> Result<MphdFlags, String> {
    for chunk in ChunkIter::new(data) {
        let (tag, payload) = chunk?;
        if tag == b"DHPM" {
            return parse_mphd_flags(payload);
        }
    }
    Err("WDT missing MPHD chunk".to_string())
}

/// The WMO of a global-WMO map (MPHD flag 0x1): the WDT's single MODF entry, named by
/// FileDataID or a legacy MWMO string index. `None` for a map of ADT tiles.
pub fn parse_wdt_global_wmo(data: &[u8]) -> Result<Option<WmoPlacement>, String> {
    if parse_wdt_mphd_flags(data)?.raw & MPHD_FLAG_GLOBAL_WMO == 0 {
        return Ok(None);
    }
    let mut names = &[][..];
    for chunk in ChunkIter::new(data) {
        let (tag, payload) = chunk?;
        if tag == b"OMWM" {
            names = payload;
        }
    }
    let offsets = mwmo_name_offsets(names);
    for chunk in ChunkIter::new(data) {
        let (tag, payload) = chunk?;
        if tag == b"FDOM" {
            return parse_global_modf(payload, names, &offsets).map(Some);
        }
    }
    Err("global-WMO WDT has no MODF".to_string())
}

fn parse_global_modf(
    payload: &[u8],
    names: &[u8],
    offsets: &[u32],
) -> Result<WmoPlacement, String> {
    let mut placement = parse_modf(payload, names, offsets)?
        .into_iter()
        .next()
        .ok_or("global-WMO WDT MODF is empty")?;
    placement.path = placement
        .path
        .map(|path| path.to_lowercase().replace('\\', "/"));
    Ok(placement)
}

fn mwmo_name_offsets(names: &[u8]) -> Vec<u32> {
    std::iter::once(0)
        .chain(
            names
                .iter()
                .enumerate()
                .filter(|(index, byte)| **byte == 0 && index + 1 < names.len())
                .map(|(index, _)| (index + 1) as u32),
        )
        .collect()
}

/// FileDataIDs in one MAID slot, indexed in MAIN order (second tile coordinate × 64 + first).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileFileIds {
    pub root: u32,
    pub obj0: u32,
    pub obj1: u32,
    pub tex0: u32,
    pub lod: u32,
    pub map_texture: u32,
    pub map_normal: u32,
    pub minimap: u32,
}

#[derive(Debug, Clone, Default)]
pub struct WdtTiles {
    pub active: BTreeSet<(u32, u32)>,
    /// None means this WDT uses named listfile companions, not MAID.
    pub maid: Option<BTreeMap<(u32, u32), TileFileIds>>,
}

impl WdtTiles {
    pub fn file_ids(&self, first: u32, second: u32) -> Option<&TileFileIds> {
        self.maid.as_ref()?.get(&(first, second))
    }
}

pub fn parse_wdt_tiles(data: &[u8]) -> Result<WdtTiles, String> {
    let mut main = None;
    let mut maid = None;
    for chunk in ChunkIter::new(data) {
        let (tag, payload) = chunk?;
        match tag {
            b"NIAM" => main = Some(payload),
            b"DIAM" => maid = Some(payload),
            _ => {}
        }
    }
    let main = main.ok_or("WDT missing MAIN chunk")?;
    if main.len() != 4096 * 8 {
        return Err(format!("WDT MAIN has {} bytes, expected 32768", main.len()));
    }
    if let Some(maid) = maid
        && maid.len() != 4096 * 32
    {
        return Err(format!(
            "WDT MAID has {} bytes, expected 131072",
            maid.len()
        ));
    }
    let mut tiles = WdtTiles {
        active: BTreeSet::new(),
        maid: maid.map(|_| BTreeMap::new()),
    };
    for index in 0..4096 {
        if read_u32(main, index * 8)? & 1 == 0 {
            continue;
        }
        let tile = ((index % 64) as u32, (index / 64) as u32);
        tiles.active.insert(tile);
        if let (Some(payload), Some(ids)) = (maid, tiles.maid.as_mut()) {
            let offset = index * 32;
            ids.insert(tile, parse_maid_entry(payload, offset)?);
        }
    }
    Ok(tiles)
}

/// Every authored MAID slot, including slots MAIN does not mark active.
/// Geometry-bake selection and the client's active-tile view are separate policies.
pub fn parse_wdt_maid_file_ids(data: &[u8]) -> Result<Option<Vec<TileFileIds>>, String> {
    let mut maid = None;
    for chunk in ChunkIter::new(data) {
        let (tag, payload) = chunk?;
        if tag == b"DIAM" {
            maid = Some(payload);
        }
    }
    let Some(payload) = maid else {
        return Ok(None);
    };
    if payload.len() != 4096 * 32 {
        return Err(format!(
            "WDT MAID has {} bytes, expected 131072",
            payload.len()
        ));
    }
    (0..4096)
        .map(|index| parse_maid_entry(payload, index * 32))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// Runtime geometry's two selected FileDataIDs per MAID slot, in authored order.
/// Unused companion fields and trailing bytes are not prerequisites for this view.
pub fn parse_maid_root_obj0(payload: &[u8]) -> Result<Vec<(u32, u32)>, String> {
    const TILE_COUNT: usize = 64 * 64;
    const ENTRY_BYTES: usize = 32;
    (0..TILE_COUNT)
        .map(|index| parse_maid_root_obj0_at(payload, index * ENTRY_BYTES))
        .collect()
}

fn parse_maid_root_obj0_at(payload: &[u8], offset: usize) -> Result<(u32, u32), String> {
    Ok((read_u32(payload, offset)?, read_u32(payload, offset + 4)?))
}

fn parse_maid_entry(payload: &[u8], offset: usize) -> Result<TileFileIds, String> {
    let (root, obj0) = parse_maid_root_obj0_at(payload, offset)?;
    Ok(TileFileIds {
        root,
        obj0,
        obj1: read_u32(payload, offset + 8)?,
        tex0: read_u32(payload, offset + 12)?,
        lod: read_u32(payload, offset + 16)?,
        map_texture: read_u32(payload, offset + 20)?,
        map_normal: read_u32(payload, offset + 24)?,
        minimap: read_u32(payload, offset + 28)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wdt_payload(chunks: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut data = Vec::new();
        for (tag, payload) in chunks {
            data.extend_from_slice(*tag);
            data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            data.extend_from_slice(payload);
        }
        data
    }

    #[test]
    fn reads_mphd_flags_after_version_chunk() {
        let mut mphd = 0x3cau32.to_le_bytes().to_vec();
        mphd.extend_from_slice(&[0; 28]);
        let data = wdt_payload(&[(b"REVM", 18u32.to_le_bytes().to_vec()), (b"DHPM", mphd)]);

        let flags = parse_wdt_mphd_flags(&data).expect("expected MPHD");

        assert_eq!(flags.raw, 0x3ca);
        assert!(flags.big_alpha());
        assert!(flags.height_texturing());
    }

    #[test]
    fn height_texturing_flag_implies_big_alpha_and_big_alpha_alone_does_not_imply_height() {
        assert!(MphdFlags { raw: 0x80 }.big_alpha());
        assert!(MphdFlags { raw: 0x4 }.big_alpha());
        assert!(!MphdFlags { raw: 0x4 }.height_texturing());
        assert!(!MphdFlags { raw: 0x2 }.big_alpha());
    }

    #[test]
    fn missing_mphd_is_an_error() {
        let data = wdt_payload(&[(b"REVM", 18u32.to_le_bytes().to_vec())]);

        assert!(parse_wdt_mphd_flags(&data).is_err());
    }
}
