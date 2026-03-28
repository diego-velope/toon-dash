//! Player Controller for Toon Dash

use super::types::{BoundingBox, GameConfig, Lane, Position3D};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerState {
    #[default]
    Running,
    Jumping,
    Sliding,
    Dead,
}

pub struct Player {
    pub position: Position3D,
    pub lane: Lane,
    pub state: PlayerState,
    pub distance_traveled: f32,
    pub jump_progress: f32,
    pub slide_progress: f32,
    pub target_lane: Lane,
    pub lane_change_progress: f32,
    pub animation_tick: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            position: Position3D::new(0.0, 0.0, 0.0),
            lane: Lane::Center,
            state: PlayerState::Running,
            distance_traveled: 0.0,
            jump_progress: 0.0,
            slide_progress: 0.0,
            target_lane: Lane::Center,
            lane_change_progress: 1.0,
            animation_tick: 0.0,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn jump(&mut self, _config: &GameConfig) -> bool {
        match self.state {
            PlayerState::Running => {
                self.state = PlayerState::Jumping;
                self.jump_progress = 0.0;
                true
            }
            // 🛠️ STATE CANCEL: If sliding, cancel slide and jump immediately
            PlayerState::Sliding => {
                self.state = PlayerState::Jumping;
                self.jump_progress = 0.0;
                self.slide_progress = 0.0;
                true
            }
            _ => false,
        }
    }

    pub fn slide(&mut self, _config: &GameConfig) -> bool {
        match self.state {
            PlayerState::Running => {
                self.state = PlayerState::Sliding;
                self.slide_progress = 0.0;
                true
            }
            // 🛠️ STATE CANCEL: If jumping, fast-fall and start sliding
            PlayerState::Jumping => {
                self.state = PlayerState::Sliding;
                self.slide_progress = 0.0;
                self.jump_progress = 0.0;
                self.position.y = 0.0; // Snap to ground instantly
                true
            }
            _ => false,
        }
    }

    pub fn change_lane(&mut self, direction: i32) -> bool {
        if self.lane_change_progress >= 1.0 {
            if let Some(new_lane) = self.lane.neighbor(direction) {
                self.target_lane = new_lane;
                self.lane_change_progress = 0.0;
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, world_dt: f32, anim_dt: f32, config: &GameConfig) {
        self.distance_traveled += config.player_speed * world_dt;
        // ── FIX: keep position.z in sync with distance traveled ──────────
        // Camera follows position.z — without this it stays at 0.0 forever.
        self.position.z = self.distance_traveled;
        self.animation_tick += anim_dt;

        match self.state {
            PlayerState::Jumping => {
                self.jump_progress += anim_dt / config.jump_duration;
                if self.jump_progress >= 1.0 {
                    self.jump_progress = 0.0;
                    self.state = PlayerState::Running;
                    self.position.y = 0.0;
                } else {
                    let t = self.jump_progress;
                    // 🛠️ ASYMMETRICAL JUMP: Fast rise, heavier fall.
                    // Remap t so the first half (rise) is stretched and the
                    // second half (fall) is compressed — removes floaty feeling.
                    let weighted_t = if t < 0.4 {
                        // Rise phase: map 0..0.4 → 0..0.5 (slower progress = more hang time going up)
                        t / 0.4 * 0.5
                    } else {
                        // Fall phase: map 0.4..1.0 → 0.5..1.0 (faster progress = snappy downfall)
                        0.5 + (t - 0.4) / 0.6 * 0.5
                    };
                    self.position.y = 4.0 * config.jump_height * weighted_t * (1.0 - weighted_t);
                }
            }
            PlayerState::Sliding => {
                self.slide_progress += anim_dt / config.slide_duration;
                if self.slide_progress >= 1.0 {
                    self.slide_progress = 0.0;
                    self.state = PlayerState::Running;
                }
            }
            PlayerState::Running | PlayerState::Dead => {}
        }

        // Lane change interpolation
        if self.lane_change_progress < 1.0 {
            self.lane_change_progress += world_dt * 5.0; // adjust speed as needed
            if self.lane_change_progress >= 1.0 {
                self.lane_change_progress = 1.0;
                self.lane = self.target_lane;
            }
        }

        // Update X position based on lane interpolation
        let start_x = self.lane.to_x(config.lane_width);
        let target_x = self.target_lane.to_x(config.lane_width);
        self.position.x = start_x + (target_x - start_x) * self.lane_change_progress;
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        let height = if self.state == PlayerState::Sliding {
            0.4
        } else {
            1.0
        };
        BoundingBox::from_center(
            self.position,
            // 🛠️ TWEAK: Increased from 0.5 to 0.55 (X) and 0.3 to 0.35 (Z) to match the larger model
            Position3D::new(0.55, height * 2.0, 0.35),
        )
    }

    pub fn is_airborne(&self) -> bool {
        self.position.y > 0.1
    }
    pub fn is_sliding(&self) -> bool {
        self.state == PlayerState::Sliding
    }
    pub fn is_alive(&self) -> bool {
        self.state != PlayerState::Dead
    }

    pub fn die(&mut self) {
        self.state = PlayerState::Dead;
    }
}
