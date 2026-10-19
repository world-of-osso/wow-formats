//! Shared little-endian readers and WoW chunk iteration.
mod read_bytes;
pub use read_bytes::*;

pub struct ChunkIter<'a> {
    data: &'a [u8],
    off: usize,
}

impl<'a> ChunkIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, off: 0 }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Result<(&'a [u8; 4], &'a [u8]), String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.off + 8 > self.data.len() {
            return None;
        }
        let tag: &[u8; 4] = self.data[self.off..self.off + 4].try_into().unwrap();
        let size = match read_u32(self.data, self.off + 4) {
            Ok(s) => s as usize,
            Err(e) => return Some(Err(e)),
        };
        let payload_start = self.off + 8;
        let payload_end = payload_start + size;
        if payload_end > self.data.len() {
            return Some(Err(format!(
                "chunk {:?} truncated at {:#x}",
                std::str::from_utf8(tag).unwrap_or("????"),
                self.off,
            )));
        }
        self.off = payload_end;
        Some(Ok((tag, &self.data[payload_start..payload_end])))
    }
}

/// First ADT/WDT/WMO chunk with the documented forward tag (stored reversed).
/// Only visited chunks are validated; a selected chunk does not validate its tail.
pub fn find_reversed_chunk<'a>(data: &'a [u8], tag: &[u8; 4]) -> Result<Option<&'a [u8]>, String> {
    let stored_tag = [tag[3], tag[2], tag[1], tag[0]];
    let mut offset = 0;
    for chunk in ChunkIter::new(data) {
        let (found, payload) = chunk.map_err(|_| {
            let stored = &data[offset..offset + 4];
            let name = [stored[3], stored[2], stored[1], stored[0]];
            format!("{} chunk truncated", String::from_utf8_lossy(&name))
        })?;
        if found == &stored_tag {
            return Ok(Some(payload));
        }
        offset += 8 + payload.len();
    }
    Ok(None)
}
