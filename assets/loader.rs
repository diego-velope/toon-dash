//! Asset Loader for Kenney Platformer Kit Models

use macroquad::prelude::*;
use std::collections::HashMap;

/// Loaded GLTF model wrapper
#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

/// Simple mesh data
#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
    pub normals: Vec<Vec3>,
    pub uvs: Option<Vec<Vec2>>,
}

/// Material properties
#[derive(Clone)]
pub struct Material {
    pub base_color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

/// Asset loader and cache
pub struct AssetLoader {
    models: HashMap<String, Model>,
    textures: HashMap<String, Texture2D>,
    loaded: bool,
}

impl Default for AssetLoader {
    fn default() -> Self { Self::new() }
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            textures: HashMap::new(),
            loaded: false,
        }
    }

    /// Load all game assets (call once at startup)
    pub async fn load_all(&mut self) -> Result<(), String> {
        if self.loaded {
            return Ok(());
        }

        log::info!("Loading assets...");

        // Load player model (Kenney character)
        self.load_model("player", "assets/models/character_obj.obj").await.ok();
        
        // Load obstacles
        self.load_model("obstacle_low", "assets/models/barrier_low.obj").await.ok();
        self.load_model("obstacle_high", "assets/models/barrier_high.obj").await.ok();
        self.load_model("obstacle_full", "assets/models/barrier_full.obj").await.ok();
        
        // Load collectibles
        self.load_model("coin", "assets/models/coin.obj").await.ok();
        
        // Load environment
        self.load_model("ground", "assets/models/platform.obj").await.ok();
        self.load_model("track_segment", "assets/models/road.obj").await.ok();

        // Load fallback texture
        self.load_texture("white", "assets/textures/white.png").await.ok();

        self.loaded = true;
        log::info!("Assets loaded: {} models, {} textures", 
                  self.models.len(), self.textures.len());

        Ok(())
    }

    /// Load a GLTF/GLB/OBJ model
    pub async fn load_model(&mut self, name: &str, path: &str) -> Result<(), String> {
        match load_model(path).await {
            Ok(model) => {
                self.models.insert(name.to_string(), self.convert_model(model));
                log::debug!("Loaded model: {}", name);
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to load model '{}': {:?}", path, e);
                // Insert placeholder
                self.models.insert(name.to_string(), self.create_placeholder_model());
                Ok(())
            }
        }
    }

    /// Convert macroquad Model to our Model struct
    fn convert_model(&self, _model: macroquad::models::Model) -> Model {
        // For now, use placeholder - macroquad's Model is complex
        self.create_placeholder_model()
    }

    /// Create a simple placeholder model (cube)
    fn create_placeholder_model(&self) -> Model {
        Model {
            meshes: vec![Mesh {
                vertices: vec![
                    Vec3::new(-0.5, -0.5, -0.5),
                    Vec3::new( 0.5, -0.5, -0.5),
                    Vec3::new( 0.5,  0.5, -0.5),
                    Vec3::new(-0.5,  0.5, -0.5),
                    Vec3::new(-0.5, -0.5,  0.5),
                    Vec3::new( 0.5, -0.5,  0.5),
                    Vec3::new( 0.5,  0.5,  0.5),
                    Vec3::new(-0.5,  0.5,  0.5),
                ],
                indices: vec![
                    0, 1, 2, 2, 3, 0, // front
                    4, 5, 6, 6, 7, 4, // back
                    0, 4, 7, 7, 3, 0, // left
                    1, 5, 6, 6, 2, 1, // right
                    3, 7, 6, 6, 2, 3, // top
                    0, 4, 5, 5, 1, 0, // bottom
                ],
                normals: vec![Vec3::Z; 8],
                uvs: None,
            }],
            materials: vec![Material {
                base_color: WHITE,
                metallic: 0.0,
                roughness: 0.5,
            }],
        }
    }

    /// Load a texture
    pub async fn load_texture(&mut self, name: &str, path: &str) -> Result<(), String> {
        match load_texture(path).await {
            Ok(texture) => {
                self.textures.insert(name.to_string(), texture);
                log::debug!("Loaded texture: {}", name);
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to load texture '{}': {:?}", path, e);
                Err(format!("Texture load error: {:?}", e))
            }
        }
    }

    /// Get a loaded model
    pub fn get_model(&self, name: &str) -> Option<&Model> {
        self.models.get(name)
    }

    /// Get a loaded texture
    pub fn get_texture(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    /// Check if assets are loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}