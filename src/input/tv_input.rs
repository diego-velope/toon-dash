// src/input/tv_input.rs
//! TV Input Handler

use macroquad::prelude::*;

pub struct TvInput {
    // Current frame state
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    action: bool,
    back: bool,
    
    // Previous frame state (for "just pressed")
    prev_up: bool,
    prev_down: bool,
    prev_left: bool,
    prev_right: bool,
    prev_action: bool,
    prev_back: bool,
}

impl TvInput {
    pub fn new() -> Self {
        Self {
            up: false, down: false, left: false, right: false,
            action: false, back: false,
            prev_up: false, prev_down: false, prev_left: false, prev_right: false,
            prev_action: false, prev_back: false,
        }
    }

    pub fn update(&mut self) {
        // Save previous state
        self.prev_up = self.up;
        self.prev_down = self.down;
        self.prev_left = self.left;
        self.prev_right = self.right;
        self.prev_action = self.action;
        self.prev_back = self.back;

        // Read current state
        self.up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        self.down = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
        self.left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        self.right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        self.action = is_key_down(KeyCode::Enter) || is_key_down(KeyCode::Space);
        self.back = is_key_down(KeyCode::Escape) || is_key_down(KeyCode::Backspace);
    }

    pub fn is_up_just_pressed(&self) -> bool { self.up && !self.prev_up }
    pub fn is_down_just_pressed(&self) -> bool { self.down && !self.prev_down }
    pub fn is_left_just_pressed(&self) -> bool { self.left && !self.prev_left }
    pub fn is_right_just_pressed(&self) -> bool { self.right && !self.prev_right }
    pub fn is_action_just_pressed(&self) -> bool { self.action && !self.prev_action }
    pub fn is_back_just_pressed(&self) -> bool { self.back && !self.prev_back }
}

impl Default for TvInput {
    fn default() -> Self {
        Self::new()
    }
}