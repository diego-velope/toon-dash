//! Procedural Track Generation for Toon Dash

use super::types::GameConfig;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Ground,
    ObstacleZone,
    CoinZone,
}

#[derive(Debug, Clone)]
pub struct TrackSegment {
    pub z_position: f32,
    pub segment_type: SegmentType,
}

pub struct Track {
    segments: Vec<TrackSegment>,
    pub segment_length: f32,
    rng: SmallRng,
    last_z: f32,
    difficulty: f32,
    pub track_width: f32,
}

impl Default for Track {
    fn default() -> Self { Self::new() }
}

impl Track {
    pub fn new() -> Self {
        Self {
            segments: Vec::with_capacity(100),
            segment_length: 10.0,
            rng: SmallRng::from_entropy(),
            last_z: 0.0,
            difficulty: 1.0,
            track_width: 8.0,
        }
    }

    pub fn reset(&mut self) {
        self.segments.clear();
        self.last_z = 0.0;
        self.difficulty = 1.0;
        self.rng = SmallRng::from_entropy();

        for i in 0..5 {
            self.segments.push(TrackSegment {
                z_position: i as f32 * self.segment_length,
                segment_type: SegmentType::Ground,
            });
        }
        self.last_z = 40.0; // CHANGED: this was 50.0, which caused a gap at z=50!
    }

    pub fn update(&mut self, player_z: f32, config: &GameConfig) {
        while self.last_z < player_z + config.spawn_distance {
            self.generate_segment();
        }

        self.segments.retain(|s| s.z_position > player_z - config.despawn_distance);
        self.difficulty = 1.0 + (player_z / 1000.0).min(1.0);
    }

    fn generate_segment(&mut self) {
        self.last_z += self.segment_length;

        let r1 = self.rng.gen::<f32>();
        let r2 = self.rng.gen::<f32>();

        let segment_type = if r1 < 0.3 * self.difficulty {
            SegmentType::ObstacleZone
        } else if r2 < 0.4 {
            SegmentType::CoinZone
        } else {
            SegmentType::Ground
        };

        self.segments.push(TrackSegment {
            z_position: self.last_z,
            segment_type,
        });
    }

    pub fn get_obstacle_zones(&self, player_z: f32, view_dist: f32) -> Vec<&TrackSegment> {
        self.segments.iter()
            .filter(|s| {
                s.segment_type == SegmentType::ObstacleZone
                    && s.z_position > player_z
                    && s.z_position < player_z + view_dist
            })
            .collect()
    }

    pub fn get_coin_zones(&self, player_z: f32, view_dist: f32) -> Vec<&TrackSegment> {
        self.segments.iter()
            .filter(|s| {
                s.segment_type == SegmentType::CoinZone
                    && s.z_position > player_z
                    && s.z_position < player_z + view_dist
            })
            .collect()
    }

    pub fn get_visible(&self, player_z: f32, view_dist: f32) -> Vec<&TrackSegment> {
        self.segments.iter()
            .filter(|s| s.z_position > player_z - 25.0 && s.z_position < player_z + view_dist)
            .collect()
    }

    pub fn difficulty(&self) -> f32 { self.difficulty }
}