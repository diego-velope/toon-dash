# TV Development Guide (Chromium 80+)

## Purpose

This is the main development guide for building and shipping Toon Dash and future TV games on modern TV runtimes (Chromium 80+ baseline).

It replaces and consolidates:
- `RUST_WASM_TV_GAME_SCAFFOLD_GUIDE.md`
- `TV_GAME_QUICK_START_CHECKLIST.md`
- `TV_GAME_ARCHITECTURE_DIAGRAM.md`
- `TIZEN_COMPATIBILITY.md`
- `SAMSUNG_TIZEN_FIX.md`

Legacy Samsung/Tizen Chromium 69 compatibility notes were moved to `.cursor/rules/` for historical context and are not the target for this project.

## Quick Start

1. Build native for local gameplay iteration:
   - `cargo run`
2. Build WASM for TV/web:
   - `cargo build --release --target wasm32-unknown-unknown`
   - copy `target/wasm32-unknown-unknown/release/toon-dash.wasm` into `web/`
3. Serve `web/` and load on target TV shell/webview.

## Runtime Architecture

High-level flow:

1. TV remote event arrives in browser/webview
2. `web/pal/` normalizes platform keycodes to logical actions (`detect-platform.js` + `platforms/*` + `pal-core.js`)
3. `window.mq_handle_*` forwards into Rust WASM exports
4. `src/tv_input_manager.rs` stores action state
5. `src/input/tv_input.rs` exposes frame-safe input to game loop
6. `src/main.rs` consumes `is_*_just_pressed()` in menu/gameplay logic

## Platform Strategy (Chromium 80+)

- Target modern TV/browser engines (Chromium 80+).
- Keep key mapping flexible and platform-identified under `web/pal/platforms/`.
- Prefer one input abstraction (`TvAction`) for all TV targets.
- Treat Samsung Chromium 69 era constraints as legacy guidance only.

## Build and Deploy Notes

- Keep WASM exports stable (`mq_handle_up/down/left/right/action/back`).
- Ensure `web/index.html` loads `web/pal/*.js` (namespace → detect → platforms → `pal-core.js`) before the WASM bundle.
- Keep `mq_js_bundle.js` and `toon-dash.wasm` in sync after each build.

## Troubleshooting

If input is not working:

1. Verify platform detection logs in `[TV-PAL]` console output from `pal-core.js`
2. Verify WASM export connection logs in `web/index.html`
3. Test with `?debug` query parameter to log key events
4. Confirm back keycode emitted by actual target device

If game loads but visual/runtime issues occur:

1. Confirm target runtime is Chromium 80+
2. Validate WebGL context creation in the console
3. Validate WASM load and export binding order

## Related Docs

- `README.md`
- `TV_INPUT_GUIDE.md`
- `TV_TESTING_GUIDE.md`
