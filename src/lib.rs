// src/lib.rs
//! Toon Dash - Endless Runner for Samsung Tizen TV

pub mod game;
pub mod input;
pub mod models;
pub mod platform;
pub mod rendering;

// TV Input Manager for WASM builds (platform abstraction layer)
#[cfg(target_arch = "wasm32")]
pub mod tv_input_manager;

pub use input::TvInput;
pub use game::{GameConfig, GameState, Player};
pub use rendering::{GameCamera, GameRenderer};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GAME_NAME: &str = "Toon Dash";
pub const TARGET_FPS: u32 = 30;

// Export TV input handler functions for WASM builds
// These are called by JavaScript TV PAL (`web/pal/pal-core.js`) via Macroquad's plugin system
#[cfg(target_arch = "wasm32")]
pub use tv_input_manager::{
    mq_handle_up, mq_handle_down, mq_handle_left, mq_handle_right,
    mq_handle_action, mq_handle_back, init_tv_input_manager
};