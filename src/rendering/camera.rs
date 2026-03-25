// src/rendering/camera.rs
// Third-person chase camera for Toon Dash
//
// Player advances in +Z. Camera sits behind (lower Z) looking forward (higher Z).
// Macroquad uses a right-handed coordinate system (look_at_rh) — this is correct.
// IMPORTANT: Camera3D.fovy is in RADIANS.

use macroquad::prelude::*;

pub struct GameCamera {
    pub position:  Vec3,
    pub target:    Vec3,
    pub distance:  f32,
    pub height:    f32,
    /// FOV in degrees (converted to radians when applied).
    pub fov_deg:   f32,
    pub smoothing: f32,
    pub look_ahead: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            position:   Vec3::new(0.0, 8.0, -15.0),
            target:     Vec3::new(0.0, 0.0, 20.0),
            distance:   15.0,
            height:     8.0,
            fov_deg:    45.0,
            smoothing:  0.12,
            look_ahead: 25.0,
        }
    }
}

impl GameCamera {
    pub fn new() -> Self { Self::default() }

    pub fn snap(&mut self, player_x: f32, player_y: f32, player_z: f32) {
        self.position = Vec3::new(
            player_x,
            player_y + self.height,
            player_z - self.distance,
        );
        self.target = Vec3::new(
            player_x,
            player_y + 1.0,
            player_z + self.look_ahead,
        );
    }

    pub fn update(&mut self, player_x: f32, player_y: f32, player_z: f32, _dt: f32) {
        let desired_pos = Vec3::new(
            player_x * 0.3,
            player_y + self.height,
            player_z - self.distance,
        );

        let desired_target = Vec3::new(
            player_x * 0.5,
            player_y + 1.0,
            player_z + self.look_ahead,
        );

        self.position = self.position.lerp(desired_pos, self.smoothing);
        self.target   = self.target.lerp(desired_target, self.smoothing);
    }

    pub fn apply(&self) {
        let aspect = screen_width() / screen_height();
        set_camera(&Camera3D {
            position:      self.position,
            target:        self.target,
            up:            Vec3::Y,
            fovy:          self.fov_deg.to_radians(),
            aspect:        Some(aspect),
            projection:    Projection::Perspective,
            render_target: None,
            viewport:      None,
            z_near:        0.1,
            z_far:         500.0,
        });
    }

    pub fn set_ui_camera() {
        set_default_camera();
    }
}