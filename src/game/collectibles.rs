// src/game/collectibles.rs
//! Collectibles System for Toon Dash

use super::types::{GameConfig, Lane, Position3D};
use super::track::TrackSegment;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectibleType {
    Coin,
    Jewel,
}

#[derive(Debug, Clone)]
pub struct Collectible {
    pub ctype: CollectibleType,
    pub position: Position3D,
    pub lane: Lane,
    pub collected: bool,
}

pub struct CollectibleManager {
    pub items: Vec<Collectible>,
    rng: SmallRng,
    last_spawn_z: f32,
    min_spacing: f32,
}

impl Default for CollectibleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectibleManager {
    pub fn new() -> Self {
        Self {
            items: Vec::with_capacity(100),
            rng: SmallRng::from_entropy(),
            last_spawn_z: 20.0,
            min_spacing: 30.0,
        }
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.rng = SmallRng::from_entropy();
        self.last_spawn_z = 20.0;
    }

    pub fn spawn_from_segments<'a, I>(&mut self, segments: I, config: &GameConfig)
    where
        I: IntoIterator<Item = &'a TrackSegment>,
    {
        for segment in segments {
            if segment.z_position > self.last_spawn_z + self.min_spacing {
                // Spawn collectible line
                let num_items = self.rng.gen_range(1..=3);
                for i in 0..num_items {
                    let lane = match i {
                        0 => Lane::Left,
                        1 => Lane::Center,
                        _ => Lane::Right,
                    };

                    // 10% chance to spawn a jewel instead of a coin
                    let ctype = if self.rng.gen_bool(0.10) {
                        CollectibleType::Jewel
                    } else {
                        CollectibleType::Coin
                    };
                    
                    self.items.push(Collectible {
                        ctype,
                        position: Position3D::new(
                            lane.to_x(config.lane_width),
                            1.0,
                            segment.z_position + (i as f32 * 8.0), // Spaced out
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
        self.items.retain(|c| c.position.z > despawn_z);
    }

    /// Returns a tuple of (coins_collected, jewels_collected)
    pub fn check_collection(
        &mut self,
        player_lane: Lane,
        player_y: f32,
        player_z: f32,
    ) -> (u32, u32) {
        let mut coins_collected = 0;
        let mut jewels_collected = 0;
        
        for item in &mut self.items {
            if item.collected { continue; }
            if item.lane != player_lane { continue; }
            
            let z_dist = (item.position.z - player_z).abs();
            let y_dist = (item.position.y - player_y).abs();
            
            if z_dist < 1.0 && y_dist < 1.5 {
                item.collected = true;
                match item.ctype {
                    CollectibleType::Coin => coins_collected += 1,
                    CollectibleType::Jewel => jewels_collected += 1,
                }
            }
        }
        
        (coins_collected, jewels_collected)
    }

    pub fn get_visible(&self, player_z: f32, view_dist: f32) -> impl Iterator<Item = &Collectible> {
        self.items.iter()
            .filter(move |c| c.position.z > player_z - 20.0 && c.position.z < player_z + view_dist)
    }
}