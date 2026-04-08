# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
