//! Game logic modules for Toon Dash

mod player;
mod track;
mod obstacles;
mod coins;
mod state;
mod types;

// Re-export everything
pub use player::{Player, PlayerState};
pub use track::{Track, TrackSegment, SegmentType};
pub use obstacles::{Obstacle, ObstacleType, ObstacleManager};
pub use coins::{Coin, CoinManager};
pub use state::{GameState, GameScreen, MenuOption, PauseOption, GameOverOption, MenuNavigator, MenuSubScreen, AudioSettings, CharacterChoice};
pub use types::{Lane, Position3D, BoundingBox, GameConfig};