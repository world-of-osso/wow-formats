# wow-formats

Engine-independent World of Warcraft binary readers extracted from `world-of-osso/game-engine` at `55e1f8687` (2026-10-18). One crate per format; no Godot, Bevy, asset fetching, or game-server dependencies.

| Crate | Contents | Optional features (enabled by default) |
| --- | --- | --- |
| `wow-chunk` | Little-endian byte readers and shared `ChunkIter` | None |
| `wow-m2` | MD20/MD21 model chunks, `.skin`, collision geometry, file references | `animation`, `attachments`, `materials`, `particles`, `ribbons`, `lights`, `cameras`, `events`, `textures`; `render` enables all |
| `wow-wmo` | WMO root/group chunks and FDID references | `render` (batch geometry), `fog`, `portals` |
| `wow-adt` | ADT root, `_tex0`, `_obj`, LOD, water and WDT metadata | `geometry` (terrain/water mesh helpers) |

Default features preserve the client's complete parser surface. Geometry consumers can use `default-features = false`; M2 rendering-data features that read animation tracks enable `animation`. Static attachment lookup uses only `attachments`; raw materials and skin batches use only `materials`. `textures` enables material metadata plus animation for render-batch sampling/composition. Geometry tools can enable `animation` separately when eye/head posing needs bone tracks, without compiling emitters, lights, cameras, events or texture composition. WMO/ADT raw chunk records remain available without optional presentation helpers. ADT and WMO reference models by FileDataID only; neither depends on `wow-m2` or the other format crate. Their only shared workspace dependency is `wow-chunk`. External dependencies are `binrw` and `glam` 0.32.

```toml
[dependencies]
wow-m2 = { git = "https://github.com/world-of-osso/wow-formats.git", rev = "<commit>", default-features = false }
```

`wow_m2::parser` exposes the original raw M2/skin readers; skin sections retain both the client's cumulative `triangle_start` and authored `raw_triangle_start` (indexStart plus level high bits). M2 first-bone-key readers preserve external/global sequence sources without requiring timestamps for a static pose. WDT exposes both the client's active-tile view and all raw MAID slots, leaving selection policy to consumers. The ADT height-only view keeps raw file origins/holes and does not require optional normals, texture payloads or companion shadows. M2 collision normals remain authored per-face geometry metadata, separate from the existing collision mesh.

The `m2_anim`, `m2_collision`, and other format namespaces retain their names. `wow_adt::{adt, adt_tex, adt_obj, wdt}` and `wow_wmo::parser` retain the original namespaces. Client model assembly, coordinate adapters and simulation policies are not part of these crates.

Runtime ground loaders can project only the fields they use: `wow_chunk::find_reversed_chunk` selects the first documented-tag chunk and validates only visited chunks; `wow_adt::wdt::parse_mphd_flags` and `parse_maid_root_obj0` read selected WDT fields; `wow_adt::adt_obj::{modf_flags,parse_modf_transform}` leave name/FDID refusal policy to the caller; `wow_wmo::parser::parse_root_group_file_ids` requires exactly the MOHD-named GFID entries without validating unrelated render chunks. These views do not make unused MAID companions, MODF bounds/sets or a full MOHD header prerequisites for runtime geometry.

## Provenance and scope

Clean-room extraction of the project's existing independently authored Rust parsers and synthetic tests. No decompiled or leaked Blizzard client source was opened, searched, reconstructed, or included during this extraction. Existing comments identify public format documentation (wowdev.wiki) and independently maintained open-source readers used by the original implementation. This extraction changes module paths, crate-boundary visibility and Cargo feature boundaries, not decoding behavior or malformed-input policy.

`extraction-inventory.json` records original client source paths and embedded test counts. Synthetic parser tests live here; authenticated fixtures requiring the client's `data/` tree remain in game-engine. No proprietary game assets are included.

## Tests

```text
cargo test --workspace
cargo test --workspace --no-default-features
cargo fmt --check
```
