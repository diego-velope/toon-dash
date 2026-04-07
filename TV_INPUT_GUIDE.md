# TV Input Guide

## Purpose

Single source of truth for TV input implementation, integration, and maintenance.

It replaces and consolidates:
- `TV_INPUT_CHANGES.md`
- `TV_INPUT_HANDOFF_DOCUMENT.md`
- `TV_INPUT_IMPLEMENTATION_SUMMARY.md`
- `TV_INPUT_QUICK_REFERENCE.md`

## Input Model

Logical actions:
- `Up`
- `Down`
- `Left`
- `Right`
- `Action`
- `Back`

Core files:
- `src/tv_input_manager.rs`
- `src/input/tv_input.rs`
- `web/tv-pal.js`
- `web/index.html`

## Data Flow

1. Remote key event captured in `tv-pal.js`
2. Keycode mapped to logical action by platform
3. JS forwards action to `window.mq_handle_*`
4. Rust exported `mq_handle_*` updates `TvInputManager`
5. `TvInput::update()` reads manager state each frame
6. `main.rs` handles gameplay/menu transitions via `is_*_just_pressed()`

## Important Behavior: Frame Latch

`TvInputManager` includes a per-action press latch so very short taps are not missed between frames.

Why this matters:
- Some devices emit keydown and keyup within one frame window.
- Instantaneous state alone can miss taps.
- Latch keeps "was pressed this frame" truth until frame update clears it.

## JavaScript PAL Rules

- Keep keycode mappings platform-specific in one mapping table.
- Keep back handling DRY through shared helper logic.
- Use capture-phase listeners for TV remote reliability.
- Keep a debug path (`?debug`) to inspect real-device keycodes.

## WASM Export Contract

Required exported handlers:
- `mq_handle_up`
- `mq_handle_down`
- `mq_handle_left`
- `mq_handle_right`
- `mq_handle_action`
- `mq_handle_back`

Any rename requires synchronized changes in:
- Rust exports (`src/tv_input_manager.rs`)
- JS forward map (`web/tv-pal.js`)
- WASM hookup (`web/index.html`)

## Adding a New Action

1. Add enum variant in `TvAction`
2. Add manager state handling and export handler in Rust
3. Add mapping and forward map entry in `tv-pal.js`
4. Hook export in `index.html`
5. Read in `TvInput::update()`
6. Consume in `main.rs` or relevant gameplay/menu modules

## Debug Checklist

1. Load game with `?debug`
2. Confirm platform detected correctly
3. Confirm keydown/keyup logs appear for remote input
4. Confirm mapped action is not `UNMAPPED`
5. Confirm Rust handler is connected and called

## Related Docs

- `TV_DEVELOPMENT_GUIDE.md`
- `TV_TESTING_GUIDE.md`
- `README.md`
