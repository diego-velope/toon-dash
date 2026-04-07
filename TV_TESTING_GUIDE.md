# TV Testing Guide

## Purpose

Unified test plan for TV input and runtime behavior on Chromium 80+ targets.

It replaces and consolidates:
- `TV_INPUT_TESTING_CHECKLIST.md`
- `SAMSUNG_TV_TESTING_GUIDE.md`

## Test Matrix

Minimum targets:
- Samsung TV (modern Tizen web runtime)
- Fire TV (Android TV shell/webview)
- Chromecast with Google TV (Android TV shell/webview)

Optional:
- LG webOS
- Vizio

## Pre-Test Setup

1. Build and deploy latest `web/` assets and WASM.
2. Start test with `?debug` to collect key telemetry.
3. Confirm platform detection log is correct.
4. Confirm TV input handlers are connected after WASM load.

## Functional Input Tests

For each target platform:

1. Press each navigation key once:
   - Up, Down, Left, Right, Action, Back
2. Verify each press triggers exactly one expected in-game action.
3. Verify no repeated fast-tap requirement for Back.
4. Verify keyup does not cancel taps before frame processing.

## Back Button Reliability Tests

Run all on each platform:

1. Single quick press on each major screen:
   - Main menu
   - Overlay/dialog
   - Gameplay
   - Pause
2. Rapid multi-press sequence
3. Press-and-hold behavior
4. Verify no browser/navigation escape unless intended

Expected:
- Back is consistently captured and routed to game behavior.
- No requirement to spam button for detection.

## Regression Tests

1. D-pad menu navigation still smooth
2. Action/confirm still works
3. No accidental double-triggering from duplicate listeners
4. No new console errors during normal play loop

## Log Capture Template

- Device model:
- OS/runtime version:
- User agent:
- Detected platform from logs:
- Back keycodes observed:
- Any unmapped keys:
- Repro steps for failures:

## Pass Criteria

- Input latency acceptable for gameplay
- Back behavior consistent on all screens
- No critical console errors
- No input dead zones

## Related Docs

- `TV_INPUT_GUIDE.md`
- `TV_DEVELOPMENT_GUIDE.md`
- `README.md`
