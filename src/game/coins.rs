// src/game/coins.rs
//! Coin System for Toon Dash

use super::types::{GameConfig, Lane, Position3D};
use super::track::TrackSegment;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Coin {
    pub position: Position3D,
    pub lane: Lane,
    pub collected: bool,
}

pub struct CoinManager {
    coins: Vec<Coin>,
    rng: SmallRng,
    last_spawn_z: f32,
    min_spacing: f32,
}

impl Default for CoinManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinManager {
    pub fn new() -> Self {
        Self {
            coins: Vec::with_capacity(100),
            rng: SmallRng::from_entropy(),
            last_spawn_z: 20.0,
            min_spacing: 8.0,
        }
    }

    pub fn reset(&mut self) {
        self.coins.clear();
        self.rng = SmallRng::from_entropy();
        self.last_spawn_z = 20.0;
    }

    pub fn spawn_from_segments(&mut self, segments: &[&TrackSegment], config: &GameConfig) {
        for segment in segments {
            if segment.z_position > self.last_spawn_z + self.min_spacing {
                // Spawn coin line
                let num_coins = self.rng.gen_range(1..=3);
                for i in 0..num_coins {
                    let lane = match i {
                        0 => Lane::Left,
                        1 => Lane::Center,
                        _ => Lane::Right,
                    };
                    
                    self.coins.push(Coin {
                        position: Position3D::new(
                            lane.to_x(config.lane_width),
                            1.0,
                            segment.z_position + (i as f32 * 2.0),
                        ),
                        lane,
                        collected: false,
                    });
                }
                self.last_spawn_z = segment.z_position;
            }
        }
    }

    pub fn update(&mut self, player_z: f32, config: &GameConfig) {
        let despawn_z = player_z - config.despawn_distance;
        self.coins.retain(|c| c.position.z > despawn_z);
    }

    pub fn check_collection(
        &mut self,
        player_lane: Lane,
        player_y: f32,
        player_z: f32,
    ) -> u32 {
        let mut collected_count = 0;
        
        for coin in &mut self.coins {
            if coin.collected { continue; }
            if coin.lane != player_lane { continue; }
            
            let z_dist = (coin.position.z - player_z).abs();
            let y_dist = (coin.position.y - player_y).abs();
            
            if z_dist < 1.0 && y_dist < 1.5 {
                coin.collected = true;
                collected_count += 1;
            }
        }
        
        collected_count
    }

    pub fn get_visible(&self, player_z: f32, view_dist: f32) -> Vec<&Coin> {
        self.coins.iter()
            .filter(|c| c.position.z > player_z - 20.0 && c.position.z < player_z + view_dist)
            .collect()
    }
}