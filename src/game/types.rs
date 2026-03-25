//! Common game types and structures

use macroquad::prelude::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lane {
    Left = -1,
    #[default]
    Center = 0,
    Right = 1,
}

impl Lane {
    pub fn to_x(&self, lane_width: f32) -> f32 {
        match self {
            Lane::Left => lane_width,
            Lane::Center => 0.0,
            Lane::Right => -lane_width,
        }
    }

    pub fn neighbor(&self, direction: i32) -> Option<Self> {
        let current = *self as i32;
        let new_index = current + direction;

        match new_index {
            -1 => Some(Lane::Left),
            0 => Some(Lane::Center),
            1 => Some(Lane::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Position3D,
    pub max: Position3D,
}

impl BoundingBox {
    pub fn from_center(center: Position3D, half_size: Position3D) -> Self {
        Self {
            min: Position3D::new(
                center.x - half_size.x,
                center.y - half_size.y,
                center.z - half_size.z,
            ),
            max: Position3D::new(
                center.x + half_size.x,
                center.y + half_size.y,
                center.z + half_size.z,
            ),
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub lane_width: f32,
    pub player_speed: f32,
    pub jump_height: f32,
    pub jump_duration: f32,
    pub slide_duration: f32,
    pub spawn_distance: f32,
    pub despawn_distance: f32,
    pub coin_value: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            lane_width: 2.0,
            player_speed: 15.0,
            jump_height: 2.5,
            jump_duration: 0.6,
            slide_duration: 0.8,
            spawn_distance: 100.0,
            despawn_distance: 30.0,
            coin_value: 10,
        }
    }
}