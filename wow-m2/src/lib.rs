//! Pure M2 format parsing namespace.
//!
//! Engine-independent M2/skin parsing; optional authored rendering data.

#[cfg(feature = "animation")]
#[path = "m2_anim.rs"]
pub mod m2_anim;
#[cfg(feature = "animation")]
#[path = "m2_attach.rs"]
pub mod m2_attach;
#[cfg(feature = "animation")]
#[path = "m2_bone_names.rs"]
pub mod m2_bone_names;
#[cfg(feature = "cameras")]
#[path = "m2_camera.rs"]
pub mod m2_camera;
#[path = "m2_collision.rs"]
pub mod m2_collision;
#[cfg(feature = "events")]
#[path = "m2_event.rs"]
pub mod m2_event;
#[cfg(feature = "lights")]
#[path = "m2_light.rs"]
pub mod m2_light;
#[cfg(feature = "particles")]
#[path = "m2_particle.rs"]
pub mod m2_particle;
#[cfg(feature = "ribbons")]
#[path = "m2_ribbon.rs"]
pub mod m2_ribbon;
#[cfg(feature = "animation")]
#[path = "m2_variation.rs"]
pub mod m2_variation;
#[path = "parser.rs"]
pub mod parser;
pub use parser::*;
pub use wow_chunk::{
    FIXED16_SCALE, fixed16_to_f32, read_f32, read_i16, read_u16, read_u32, read_vec3,
    unorm16_to_f32,
};

pub const MD20_VERSION_OFFSET: usize = 0x04;
pub const MD20_FLAGS_OFFSET: usize = 0x10;
pub const MD20_GLOBAL_SEQUENCES_COUNT_OFFSET: usize = 0x14;
pub const MD20_SEQUENCES_COUNT_OFFSET: usize = 0x1C;
pub const MD20_BONES_COUNT_OFFSET: usize = 0x2C;
pub const MD20_VERTICES_COUNT_OFFSET: usize = 0x3C;
pub const MD20_COLORS_COUNT_OFFSET: usize = 0x48;
pub const MD20_TEXTURES_COUNT_OFFSET: usize = 0x50;
pub const MD20_TRANSPARENCY_COUNT_OFFSET: usize = 0x58;
pub const MD20_TEXTURE_WEIGHTS_COUNT_OFFSET: usize = 0x60;
pub const MD20_MATERIALS_COUNT_OFFSET: usize = 0x70;
pub const MD20_TEXTURE_LOOKUP_COUNT_OFFSET: usize = 0x80;
pub const MD20_TEXTURE_UNIT_LOOKUP_COUNT_OFFSET: usize = 0x88;
pub const MD20_BOUNDING_BOX_MIN_OFFSET: usize = 0xA0;
pub const MD20_BOUNDING_BOX_MAX_OFFSET: usize = 0xAC;
pub const MD20_ATTACHMENTS_COUNT_OFFSET: usize = 0xF0;
pub const MD20_ATTACHMENT_LOOKUP_COUNT_OFFSET: usize = 0xF8;
pub const MD20_PARTICLE_EMITTERS_COUNT_OFFSET: usize = 0x128;

/// Parse the vertex bounding box from the MD20 header.
/// Returns (min, max) as `[f32; 3]` in WoW model-local coordinates.
pub fn parse_bounding_box(md20: &[u8]) -> ([f32; 3], [f32; 3]) {
    let min = read_vec3(md20, MD20_BOUNDING_BOX_MIN_OFFSET).unwrap_or([0.0; 3]);
    let max = read_vec3(md20, MD20_BOUNDING_BOX_MAX_OFFSET).unwrap_or([0.0; 3]);
    (min, max)
}

#[cfg(feature = "textures")]
pub mod m2_batch_data;
#[cfg(feature = "textures")]
pub mod m2_texture;
#[cfg(feature = "textures")]
pub mod m2_texture_composite_data;
