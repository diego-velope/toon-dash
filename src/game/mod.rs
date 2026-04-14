//! Game logic modules for Toon Dash

mod collectibles;
pub mod loading;
pub mod obstacles;
pub mod persistence;
pub mod player;
pub mod state;
pub mod track;
pub mod types;

// Re-export everything
pub use persistence::{load_stats, save_stats};
pub use collectibles::{Collectible, CollectibleManager, CollectibleType};
pub use obstacles::{Obstacle, ObstacleManager, ObstacleType};
pub use player::{Player, PlayerState};
pub use state::{
    CharacterChoice, GameOverOption, GameScreen, GameSettings, GameState, LifetimeStats, MenuNavigator,
    MenuOption, MenuSubScreen, PauseOption, ShutdownFlow, ShutdownStage,
};
pub use track::{SegmentType, Track, TrackSegment};
pub use types::{BoundingBox, GameConfig, Lane, Position3D};
