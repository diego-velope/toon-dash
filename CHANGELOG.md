# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-04-14

### Added

- New collectible `CollectibleType::Star`, loaded from `assets/models/star.glb` and spawned under high barriers.
- Lifetime progression stats (`LifetimeStats`) with persistent totals for high score, coins earned, stars earned, and distance traveled.
- Main-menu lifetime stats panel and gameplay HUD `Stars` line.
- Game-over stats line for stars and coins collected in the run.

### Changed

- Menu/character-select preview animation now uses front-facing idle motion (bob/yaw/roll) instead of full continuous spin, with phase reset on character change.
- Character preview orientation adjusted to face the camera correctly in menu previews.
- Star spawning under high barriers now uses a 70% spawn chance.
- WASM/browser persistence wiring expanded in `web/index.html` with `mq_save_stats` and `mq_load_stats_*` bindings for lifetime stats.

### Fixed

- Star pickups no longer increase the combo multiplier; they award score only (`+500`) as intended.

## [0.1.0] - 2026-04-08

### Added

- `GameRenderer::scratch_verts` — a reusable `Vec<Vertex>` passed into GLTF mesh draw helpers so transformed vertex data is not allocated from scratch on every draw call.
- `TvAction::index()` — stable `usize` mapping for the six TV remote actions, backing fixed-size input state.
- `GameSettings::handle_options_input` — centralizes volume/speed slider and navigation for the options overlay from both the main menu and pause menu.

### Changed

- `draw_mesh_at`, `draw_mesh_at_rot`, and `draw_mesh_at_transform` in `src/models/gltf_mesh.rs` now take `scratch: &mut Vec<Vertex>`; `GameRenderer` passes `&mut self.scratch_verts` at call sites (several render helpers take `&mut self` where needed).
- `TvInputManager` stores current, previous, and latched press state as `[bool; 6]` instead of three `HashMap<TvAction, bool>` maps.
- TV WASM exports: removed redundant `#[no_mangle]` attributes where `#[export_name = "..."]` is already set (avoids `unused_attributes` warnings).
- Global TV input on WASM: replaced `static mut TV_INPUT_MANAGER` with an `UnsafeCell<Option<TvInputManager>>` behind a `Sync` wrapper, eliminating `static_mut_refs` warnings while keeping the same exported C symbols.

### Removed

- Unused `platform` module (`src/platform/`) and `pub mod platform` from `src/lib.rs`.
- Orphaned Rust files under `assets/` (`mod.rs`, `loader.rs`) that were never part of the crate build.
- Unused `web/bootstrap.js` (not referenced by `web/index.html`).

### Fixed

- Clean `cargo build --release --target wasm32-unknown-unknown` (with `build.sh` `RUSTFLAGS`) for the library crate: prior 14 warnings in `tv_input_manager.rs` are resolved.
