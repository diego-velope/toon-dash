// src/platform/tizen.rs
//! Samsung Tizen TV Platform Support

/// Tizen-specific key codes
pub struct TizenKeys;

impl TizenKeys {
    pub const BACK: u32 = 10009;
    pub const ENTER: u32 = 13;
    pub const UP: u32 = 38;
    pub const DOWN: u32 = 40;
    pub const LEFT: u32 = 37;
    pub const RIGHT: u32 = 39;
}