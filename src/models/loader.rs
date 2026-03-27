// src/models/loader.rs
//! Model loading: Kenney city roads (roads atlas) + platformer kit (platformer atlas).

use macroquad::file::load_file;
use macroquad::models::Mesh;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::mesh_from_glb_bytes;

/// Paths relative to `assets/` (`set_pc_assets_folder("assets")` on PC).
/// 🛠️ TWEAK HERE: If you download new models from Kenney or make your own,
/// drop them in your assets folder and update these strings!
pub mod paths {
    pub const COLORMAP_ROADS: &str = "models/textures/colormap_roads.png";
    pub const COLORMAP_PLATFORMER: &str = "models/textures/colormap_platformer.png";
    pub const ROAD_STRAIGHT: &str = "models/road-straight.glb";
    pub const CHARACTER_OODI: &str = "models/character-oodi.glb";
    pub const COIN_GOLD: &str = "models/coin-gold.glb";
    pub const FENCE_LOW_STRAIGHT: &str = "models/fence-low-straight.glb";
    pub const SPIKE_BLOCK: &str = "models/spike-block-wide.glb";
    pub const POLES: &str = "models/poles.glb";

    // Character selection variants (from kenney_platformer-kit)
    pub const CHAR_OODI: &str = "models/character-oodi.glb";
    pub const CHAR_OOLI: &str = "models/character-ooli.glb";
    pub const CHAR_OOZI: &str = "models/character-oozi.glb";
    pub const CHAR_OOPI: &str = "models/character-oopi.glb";
    pub const CHAR_OOBI: &str = "models/character-oobi.glb";
}

pub struct ModelManager {
    colors: HashMap<String, Color>,
    meshes: HashMap<String, Mesh>,
    loaded: bool,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            colors: HashMap::new(),
            meshes: HashMap::new(),
            loaded: false,
        }
    }

    pub async fn load_models(&mut self) {
        if self.loaded {
            return;
        }

        info!("Loading model colors...");

        self.colors
            .insert("character".to_string(), Color::from_rgba(70, 130, 220, 255));
        self.colors
            .insert("coin".to_string(), Color::from_rgba(255, 200, 50, 255));
        self.colors.insert(
            "obstacle_low".to_string(),
            Color::from_rgba(220, 80, 80, 255),
        );
        self.colors.insert(
            "obstacle_high".to_string(),
            Color::from_rgba(220, 140, 60, 255),
        );
        self.colors.insert(
            "obstacle_full".to_string(),
            Color::from_rgba(150, 80, 180, 255),
        );
        self.colors.insert(
            "track_segment".to_string(),
            Color::from_rgba(100, 100, 120, 255),
        );

        let atlas_roads = match load_texture(paths::COLORMAP_ROADS).await {
            Ok(t) => {
                info!("Loaded roads atlas {}", paths::COLORMAP_ROADS);
                Some(t)
            }
            Err(e) => {
                info!(
                    "Could not load {} ({e:?}) — road mesh may look wrong",
                    paths::COLORMAP_ROADS
                );
                None
            }
        };

        let atlas_platformer = match load_texture(paths::COLORMAP_PLATFORMER).await {
            Ok(t) => {
                info!("Loaded platformer atlas {}", paths::COLORMAP_PLATFORMER);
                Some(t)
            }
            Err(e) => {
                info!(
                    "Could not load {} ({e:?}) — character/coins/obstacles may look wrong",
                    paths::COLORMAP_PLATFORMER
                );
                None
            }
        };

        // ── Core gameplay models ──────────────────────────────────────────
        self.try_load_glb("road_straight", paths::ROAD_STRAIGHT, atlas_roads)
            .await;
        self.try_load_glb("character", paths::CHARACTER_OODI, atlas_platformer.clone())
            .await;
        self.try_load_glb("coin", paths::COIN_GOLD, atlas_platformer.clone())
            .await;
        self.try_load_glb(
            "obstacle_low",
            paths::FENCE_LOW_STRAIGHT,
            atlas_platformer.clone(),
        )
        .await;
        self.try_load_glb(
            "obstacle_high",
            paths::SPIKE_BLOCK,
            atlas_platformer.clone(),
        )
        .await;
        self.try_load_glb("obstacle_full", paths::POLES, atlas_platformer.clone())
            .await;

        // ── Character selection variants ──────────────────────────────────
        self.try_load_glb("char_oodi", paths::CHAR_OODI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_ooli", paths::CHAR_OOLI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oozi", paths::CHAR_OOZI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oopi", paths::CHAR_OOPI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oobi", paths::CHAR_OOBI, atlas_platformer)
            .await;

        self.loaded = true;
        info!("Model assets ready ({} mesh(es)).", self.meshes.len());
    }

    // 🛠️ EXPERT TWEAK: This function maps GLB files to Atlases (Textures).
    // If your models look grey or blank, verify you passed in the correct PNG atlas Option!
    async fn try_load_glb(&mut self, key: &str, path: &str, atlas: Option<Texture2D>) {
        match load_file(path).await {
            Ok(bytes) => match mesh_from_glb_bytes(&bytes, atlas) {
                Ok(mesh) => {
                    self.meshes.insert(key.to_string(), mesh);
                    info!("GLB mesh {:?} ← {}", key, path);
                }
                Err(e) => info!("GLB parse {:?} failed: {} — using primitives", key, e),
            },
            Err(e) => info!("Could not read {}: {:?} — using primitives", path, e),
        }
    }

    pub fn get_color(&self, name: &str) -> Color {
        *self.colors.get(name).unwrap_or(&WHITE)
    }

    pub fn mesh(&self, name: &str) -> Option<&Mesh> {
        self.meshes.get(name)
    }

    pub fn set_mesh(&mut self, name: &str, mesh: Mesh) {
        self.meshes.insert(name.to_string(), mesh);
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}
