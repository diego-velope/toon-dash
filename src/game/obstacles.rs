//! Obstacle System for Toon Dash

use super::types::{BoundingBox, GameConfig, Lane, Position3D};
use super::track::TrackSegment;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    LowBarrier,
    HighBarrier,
    FullBarrier,
}

impl ObstacleType {
    // 🛠️ TWEAK HERE: The logical "height" of the hitbox for each obstacle. 
    // This determines if the player bumps their head when sliding under or hits their feet when jumping over.
    pub fn height(&self) -> f32 {
        match self {
            ObstacleType::LowBarrier => 1.0,
            ObstacleType::HighBarrier => 1.5,
            ObstacleType::FullBarrier => 2.5,
        }
    }

    // 🛠️ TWEAK HERE: The visual floating height offset of the obstacle. 
    // If your custom model floats above the ground or clips into it, adjust this value!
    pub fn y_offset(&self) -> f32 {
        match self {
            ObstacleType::LowBarrier => 0.5,
            ObstacleType::HighBarrier => 1.75,
            ObstacleType::FullBarrier => 1.25,
        }
    }

    pub fn avoidable_by_jump(&self) -> bool {
        matches!(self, ObstacleType::LowBarrier)
    }

    pub fn avoidable_by_slide(&self) -> bool {
        matches!(self, ObstacleType::HighBarrier)
    }
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub position: Position3D,
    pub obstacle_type: ObstacleType,
    pub lane: Lane,
    pub passed: bool,
}

pub struct ObstacleManager {
    obstacles: Vec<Obstacle>,
    rng: SmallRng,
    last_spawn_z: f32,
    min_spacing: f32,
}

impl Default for ObstacleManager {
    fn default() -> Self { Self::new() }
}

impl ObstacleManager {
    pub fn new() -> Self {
        Self {
            obstacles: Vec::with_capacity(50),
            rng: SmallRng::from_entropy(),
            last_spawn_z: 30.0,
            min_spacing: 20.0,
        }
    }

    pub fn reset(&mut self) {
        self.obstacles.clear();
        self.rng = SmallRng::from_entropy();
        self.last_spawn_z = 30.0;
    }

    pub fn spawn_from_segments(&mut self, segments: &[&TrackSegment], config: &GameConfig) {
        for segment in segments {
            if segment.z_position > self.last_spawn_z + self.min_spacing {
                if self.rng.gen::<f32>() < 0.5 {
                    self.spawn_obstacle(segment.z_position, config);
                    self.last_spawn_z = segment.z_position;
                }
            }
        }
    }

    fn spawn_obstacle(&mut self, z: f32, config: &GameConfig) {
        let r = self.rng.gen::<f32>();
        let obstacle_type = if r < 0.4 {
            ObstacleType::LowBarrier
        } else if r < 0.7 {
            ObstacleType::HighBarrier
        } else {
            ObstacleType::FullBarrier
        };

        let r2 = self.rng.gen::<f32>();
        let lane = if r2 < 0.33 {
            Lane::Left
        } else if r2 < 0.66 {
            Lane::Center
        } else {
            Lane::Right
        };

        self.obstacles.push(Obstacle {
            position: Position3D::new(
                lane.to_x(config.lane_width),
                obstacle_type.y_offset(),
                z,
            ),
            obstacle_type,
            lane,
            passed: false,
        });
    }

    pub fn update(&mut self, player_z: f32, config: &GameConfig) {
        let despawn_z = player_z - config.despawn_distance;
        self.obstacles.retain(|o| o.position.z > despawn_z);

        for obstacle in &mut self.obstacles {
            if obstacle.position.z < player_z && !obstacle.passed {
                obstacle.passed = true;
            }
        }
    }

    pub fn check_collision(
        &self,
        player_bbox: &BoundingBox,
        player_lane: Lane,
        is_jumping: bool,
        is_sliding: bool,
    ) -> Option<&Obstacle> {
        for obstacle in &self.obstacles {
            if obstacle.lane != player_lane { continue; }
            if obstacle.obstacle_type.avoidable_by_jump() && is_jumping { continue; }
            if obstacle.obstacle_type.avoidable_by_slide() && is_sliding { continue; }

            // 🛠️ TWEAK HERE: The core hitbox boundaries.
            // Position3D::new(width_half, height_half, depth_half)
            // Shrunk X from 0.8→0.65 and Z from 0.3→0.15 for "forgiveness zone":
            // visually grazing an obstacle won't kill you, only dead-center hits will.
            let obstacle_bbox = BoundingBox::from_center(
                obstacle.position,
                Position3D::new(0.65, obstacle.obstacle_type.height() / 2.0, 0.15),
            );

            if player_bbox.intersects(&obstacle_bbox) {
                return Some(obstacle);
            }
        }
        None
    }

    pub fn get_visible(&self, player_z: f32, view_dist: f32) -> Vec<&Obstacle> {
        self.obstacles.iter()
            .filter(|o| o.position.z > player_z - 20.0 && o.position.z < player_z + view_dist)
            .collect()
    }
}