// src/lib.rs
//! Toon Dash - Endless Runner for Samsung Tizen TV

pub mod game;
pub mod input;
pub mod models;
pub mod platform;
pub mod rendering;

pub use input::TvInput;
pub use game::{GameConfig, GameState, Player};
pub use rendering::{GameCamera, GameRenderer};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GAME_NAME: &str = "Toon Dash";
pub const TARGET_FPS: u32 = 30;