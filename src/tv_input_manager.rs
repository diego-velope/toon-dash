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

use std::collections::HashMap;

/// Logical action types that map to physical TV buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TvAction {
    Up,
    Down,
    Left,
    Right,
    Action,    // Enter/Select/Confirm button
    Back,      // Back/Return button (required for TV certification)
}

/// TV Input Manager
///
/// Stores the current state of all TV remote actions.
/// Updated via wasm-bindgen calls from JavaScript PAL.
pub struct TvInputManager {
    // Current frame state
    current_state: HashMap<TvAction, bool>,

    // Previous frame state (for "just pressed" detection)
    previous_state: HashMap<TvAction, bool>,

    // Latch: true if the action was pressed at any point since the last update().
    // This ensures a press+release that both occur within a single game frame
    // (common on Samsung 2025 / Tizen 8.0 back button) is still detected.
    pressed_latch: HashMap<TvAction, bool>,
}

impl TvInputManager {
    /// Create a new TV input manager
    pub fn new() -> Self {
        let mut current_state = HashMap::new();
        let mut previous_state = HashMap::new();
        let mut pressed_latch = HashMap::new();

        // Initialize all actions to false
        for action in [
            TvAction::Up,
            TvAction::Down,
            TvAction::Left,
            TvAction::Right,
            TvAction::Action,
            TvAction::Back,
        ] {
            current_state.insert(action, false);
            previous_state.insert(action, false);
            pressed_latch.insert(action, false);
        }

        Self {
            current_state,
            previous_state,
            pressed_latch,
        }
    }

    /// Set the state of an action (called by wasm-bindgen)
    pub fn set_action(&mut self, action: TvAction, pressed: bool) {
        *self.current_state.entry(action).or_insert(false) = pressed;
        if pressed {
            // Set the latch so that a press+release within a single game frame
            // is still detected by was_action_pressed() on that frame's read.
            *self.pressed_latch.entry(action).or_insert(false) = true;
        }
    }

    /// Update to next frame (call at end of each frame, after state has been read)
    pub fn update(&mut self) {
        // Copy current state to previous state
        for (action, state) in self.current_state.iter() {
            self.previous_state.insert(*action, *state);
        }
        // Clear the press latch now that this frame's reads are complete.
        // Any new presses arriving before the next update() call will re-set it.
        for val in self.pressed_latch.values_mut() {
            *val = false;
        }
    }

    /// Check if an action is currently pressed (held state)
    pub fn is_action_pressed(&self, action: TvAction) -> bool {
        *self.current_state.get(&action).unwrap_or(&false)
    }

    /// Check if an action was pressed at any point since the last update().
    ///
    /// Unlike is_action_pressed(), this returns true even when the key was
    /// pressed AND released within a single game frame — the latch retains
    /// the event until update() clears it. Use this for "just pressed"
    /// detection to avoid missing fast button taps (Samsung 2025 back key).
    pub fn was_action_pressed(&self, action: TvAction) -> bool {
        *self.pressed_latch.get(&action).unwrap_or(&false)
    }

    /// Check if an action was just pressed this frame
    pub fn is_action_just_pressed(&self, action: TvAction) -> bool {
        let current = *self.current_state.get(&action).unwrap_or(&false);
        let previous = *self.previous_state.get(&action).unwrap_or(&false);
        current && !previous
    }

    /// Clear all input state (useful when pausing/resuming)
    pub fn clear(&mut self) {
        for state in self.current_state.values_mut() {
            *state = false;
        }
        for state in self.previous_state.values_mut() {
            *state = false;
        }
        for state in self.pressed_latch.values_mut() {
            *state = false;
        }
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

/// Global TV input manager instance (only used in WASM builds)
#[cfg(target_arch = "wasm32")]
static mut TV_INPUT_MANAGER: Option<TvInputManager> = None;

/// Initialize the TV input manager (call from main on startup)
#[cfg(target_arch = "wasm32")]
pub fn init_tv_input_manager() {
    unsafe {
        TV_INPUT_MANAGER = Some(TvInputManager::new());
    }
    log::info!("TV Input Manager initialized for WASM");
}

/// Get a reference to the global TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager() -> Option<&'static TvInputManager> {
    unsafe { TV_INPUT_MANAGER.as_ref() }
}

/// Get a mutable reference to the global TV input manager
#[cfg(target_arch = "wasm32")]
pub fn get_tv_input_manager_mut() -> Option<&'static mut TvInputManager> {
    unsafe { TV_INPUT_MANAGER.as_mut() }
}

/// Simple handle functions for each TV action
/// These are exposed to JavaScript via Macroquad's plugin system
/// Using separate functions for each action to avoid string allocation

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_up"]
pub extern "C" fn mq_handle_up(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
            manager.set_action(TvAction::Up, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_down"]
pub extern "C" fn mq_handle_down(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
            manager.set_action(TvAction::Down, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_left"]
pub extern "C" fn mq_handle_left(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
            manager.set_action(TvAction::Left, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_right"]
pub extern "C" fn mq_handle_right(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
            manager.set_action(TvAction::Right, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_action"]
pub extern "C" fn mq_handle_action(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
            manager.set_action(TvAction::Action, pressed != 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[export_name = "mq_handle_back"]
pub extern "C" fn mq_handle_back(pressed: i32) {
    unsafe {
        if let Some(manager) = TV_INPUT_MANAGER.as_mut() {
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

// Export TvAction and TvInputManager for use in other modules
// pub use {TvAction, TvInputManager};
