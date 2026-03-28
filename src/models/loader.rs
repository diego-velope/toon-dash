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
    pub const JEWEL: &str = "models/jewel.glb";

    // Character selection variants
    pub const CHAR_OODI: &str = "models/character-oodi.glb";
    pub const CHAR_OOLI: &str = "models/character-ooli.glb";
    pub const CHAR_OOZI: &str = "models/character-oozi.glb";
    pub const CHAR_OOPI: &str = "models/character-oopi.glb";
    pub const CHAR_OOBI: &str = "models/character-oobi.glb";

    // Decorations (Platformer Kit)
    pub const DEC_FLOWERS_TALL: &str = "models/decoration/flowers-tall.glb";
    pub const DEC_GRASS: &str = "models/decoration/grass.glb";
    pub const DEC_MUSHROOMS: &str = "models/decoration/mushrooms.glb";
    pub const DEC_PLANT: &str = "models/decoration/plant.glb";
    pub const DEC_PIPE: &str = "models/decoration/pipe.glb";
    pub const DEC_ROCKS: &str = "models/decoration/rocks.glb";
    pub const DEC_STONES: &str = "models/decoration/stones.glb";
    pub const DEC_TREE_PINE_SMALL: &str = "models/decoration/tree-pine-small.glb";
    pub const DEC_TREE: &str = "models/decoration/tree.glb";

    // Decorations (City Roads Kit)
    pub const DEC_LIGHT_CURVED: &str = "models/decoration/light-curved.glb";
    pub const DEC_CONE: &str = "models/decoration/construction-cone.glb";
    pub const DEC_CONSTRUCTION_LIGHT: &str = "models/decoration/construction-light.glb";
    pub const DEC_BARRIER: &str = "models/decoration/construction-barrier.glb";
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

        crate::game::loading::set_progress(15.0);
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

        crate::game::loading::set_progress(20.0);
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

        crate::game::loading::set_progress(25.0);
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

        crate::game::loading::set_progress(45.0);
        // ── Core gameplay models ──────────────────────────────────────────
        self.try_load_glb("road_straight", paths::ROAD_STRAIGHT, atlas_roads.clone())
            .await;
        crate::game::loading::set_progress(50.0);
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
        crate::game::loading::set_progress(55.0);
        self.try_load_glb(
            "obstacle_high",
            paths::SPIKE_BLOCK,
            atlas_platformer.clone(),
        )
        .await;
        self.try_load_glb("obstacle_full", paths::POLES, atlas_platformer.clone())
            .await;
        self.try_load_glb("jewel", paths::JEWEL, atlas_platformer.clone())
            .await;

        crate::game::loading::set_progress(65.0);
        // ── Character selection variants ──────────────────────────────────
        self.try_load_glb("char_oodi", paths::CHAR_OODI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_ooli", paths::CHAR_OOLI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oozi", paths::CHAR_OOZI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oopi", paths::CHAR_OOPI, atlas_platformer.clone())
            .await;
        self.try_load_glb("char_oobi", paths::CHAR_OOBI, atlas_platformer.clone())
            .await;

        crate::game::loading::set_progress(75.0);
        // ── Decorations ──────────────────────────────────────────────────
        self.try_load_glb("dec_flowers_tall", paths::DEC_FLOWERS_TALL, atlas_platformer.clone()).await;
        self.try_load_glb("dec_grass", paths::DEC_GRASS, atlas_platformer.clone()).await;
        self.try_load_glb("dec_mushrooms", paths::DEC_MUSHROOMS, atlas_platformer.clone()).await;
        self.try_load_glb("dec_plant", paths::DEC_PLANT, atlas_platformer.clone()).await;
        self.try_load_glb("dec_pipe", paths::DEC_PIPE, atlas_platformer.clone()).await;
        self.try_load_glb("dec_rocks", paths::DEC_ROCKS, atlas_platformer.clone()).await;
        self.try_load_glb("dec_stones", paths::DEC_STONES, atlas_platformer.clone()).await;
        self.try_load_glb("dec_tree_pine", paths::DEC_TREE_PINE_SMALL, atlas_platformer.clone()).await;
        self.try_load_glb("dec_tree", paths::DEC_TREE, atlas_platformer.clone()).await;

        crate::game::loading::set_progress(80.0);
        self.try_load_glb("dec_light", paths::DEC_LIGHT_CURVED, atlas_roads.clone()).await;
        self.try_load_glb("dec_cone", paths::DEC_CONE, atlas_roads.clone()).await;
        self.try_load_glb("dec_clight", paths::DEC_CONSTRUCTION_LIGHT, atlas_roads.clone()).await;
        self.try_load_glb("dec_barrier", paths::DEC_BARRIER, atlas_roads).await;

        self.loaded = true;
        crate::game::loading::set_progress(85.0);
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
