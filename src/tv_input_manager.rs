// src/tv_input_manager.rs
//! TV Input Manager for WASM builds
//!
//! This module provides a platform-agnostic input layer for TV platforms.
//! It receives input events from JavaScript via wasm-bindgen and exposes
//! a simple API for the game to query input state.
//!
//! Supported platforms:
//! - Samsung Tizen (Back: 10009)
//! - LG webOS (Back: 461)
//! - Vizio (Back: 8)
//! - Fire TV / Android TV (Back: 8, Enter: 23)
//! - Browser (Back: Escape, Enter: Enter)

use std::cell::UnsafeCell;

/// Logical action types that map to physical TV buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvAction {
    Up,
    Down,
    Left,
    Right,
    Action, // Enter/Select/Confirm button
    Back,   // Back/Return button (required for TV certification)
}

impl TvAction {
    /// Stable index into the fixed-size input arrays (`[bool; 6]`).
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            TvAction::Up => 0,
            TvAction::Down => 1,
            TvAction::Left => 2,
            TvAction::Right => 3,
            TvAction::Action => 4,
            TvAction::Back => 5,
        }
    }
}

const ACTION_COUNT: usize = 6;

/// TV Input Manager
///
/// Stores the current state of all TV remote actions.
/// Updated via wasm-bindgen calls from JavaScript PAL.
pub struct TvInputManager {
    current_state: [bool; ACTION_COUNT],
    previous_state: [bool; ACTION_COUNT],
    /// Latch: true if the action was pressed at any point since the last update().
    /// This ensures a press+release that both occur within a single game frame
    /// (common on Samsung 2025 / Tizen 8.0 back button) is still detected.
    pressed_latch: [bool; ACTION_COUNT],
}

impl TvInputManager {
    /// Create a new TV input manager
    pub fn new() -> Self {
        Self {
            current_state: [false; ACTION_COUNT],
            previous_state: [false; ACTION_COUNT],
            pressed_latch: [false; ACTION_COUNT],
        }
    }

    /// Set the state of an action (called by wasm-bindgen)
    pub fn set_action(&mut self, action: TvAction, pressed: bool) {
        let i = action.index();
        self.current_state[i] = pressed;
        if pressed {
            self.pressed_latch[i] = true;
        }
    }

    /// Update to next frame (call at end of each frame, after state has been read)
    pub fn update(&mut self) {
        self.previous_state.copy_from_slice(&self.current_state);
        for v in self.pressed_latch.iter_mut() {
            *v = false;
        }
    }

    /// Check if an action is currently pressed (held state)
    pub fn is_action_pressed(&self, action: TvAction) -> bool {
        self.current_state[action.index()]
    }

    /// Check if an action was pressed at any point since the last update().
    ///
    /// Unlike is_action_pressed(), this returns true even when the key was
    /// pressed AND released within a single game frame — the latch retains
    /// the event until update() clears it. Use this for "just pressed"
    /// detection to avoid missing fast button taps (Samsung 2025 back key).
    pub fn was_action_pressed(&self, action: TvAction) -> bool {
        self.pressed_latch[action.index()]
    }

    /// Check if an action was just pressed this frame
    pub fn is_action_just_pressed(&self, action: TvAction) -> bool {
        let i = action.index();
        self.current_state[i] && !self.previous_state[i]
    }

    /// Clear all input state (useful when pausing/resuming)
    pub fn clear(&mut self) {
        self.current_state = [false; ACTION_COUNT];
        self.previous_state = [false; ACTION_COUNT];
        self.pressed_latch = [false; ACTION_COUNT];
    }
}

impl Default for TvInputManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MACROQUAD WASM INTEGRATION
// ============================================================================

/// Single-threaded global slot (WASM); avoids `static mut` reference warnings.
#[cfg(target_arch = "wasm32")]
struct TvInputGlobal(UnsafeCell<Option<TvInputManager>>);

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TvInputGlobal {}

#[cfg(target_arch = "wasm32")]
static TV_INPUT_GLOBAL: TvInputGlobal = TvInputGlobal(UnsafeCell::new(None));

/// Initialize the TV input manager (call from main on startup)
#[cfg(target_arch = "wasm32")]
pub fn init_tv_input_manager() {
    unsafe {
        *TV_INPUT_GLOBAL.0.get() = Some(TvInputManager::new());
    }
    log::info!("TV Input Manager initialized for WASM");
}

/// Get a reference to the global TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager() -> Option<&'static TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_ref() }
}

/// Get a mutable reference to the global TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager_mut() -> Option<&'static mut TvInputManager> {
    unsafe { (*TV_INPUT_GLOBAL.0.get()).as_mut() }
}

/// Simple handle functions for each TV action
/// These are exposed to JavaScript via Macroquad's plugin system
/// Using separate functions for each action to avoid string allocation

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_up"]
pub extern "C" fn mq_handle_up(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Up, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_down"]
pub extern "C" fn mq_handle_down(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Down, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_left"]
pub extern "C" fn mq_handle_left(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Left, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_right"]
pub extern "C" fn mq_handle_right(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Right, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_action"]
pub extern "C" fn mq_handle_action(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Action, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[export_name = "mq_handle_back"]
pub extern "C" fn mq_handle_back(pressed: i32) {
    unsafe {
        if let Some(manager) = (*TV_INPUT_GLOBAL.0.get()).as_mut() {
            manager.set_action(TvAction::Back, pressed != 0);
        }
    }
}

// ============================================================================
// NON-WASM IMPLEMENTATION (stubs)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub fn init_tv_input_manager() {
    // No-op on non-WASM platforms
}
