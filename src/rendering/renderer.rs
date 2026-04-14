// src/rendering/renderer.rs
//! Game renderer with Kenney-style graphics

use super::camera::GameCamera;
use crate::game::*;
use crate::models::{draw_mesh_at_rot, draw_mesh_at_transform, ModelManager};
use glam::Quat;
use macroquad::models::Vertex;
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

/// City Kit `road-straight` tile: scale/step tuned so ~5 tiles cover segment length (~10).
const ROAD_TILE_SCALE: f32 = 2.35;
const ROAD_TILE_STEP: f32 = 2.35;
const ROAD_SURFACE_Y: f32 = 0.03;
/// Character mesh pivot so feet sit on the road surface.
const PLAYER_MESH_PIVOT_Y: f32 = 0.58;

/// Main menu / character-select preview: subtle idle motion (no full spin).
const PREVIEW_IDLE_BOB_AMP: f32 = 0.045;
const PREVIEW_IDLE_BOB_HZ: f32 = 1.2;
const PREVIEW_IDLE_YAW_AMP: f32 = 0.07;
const PREVIEW_IDLE_YAW_HZ: f32 = 1.2;
const PREVIEW_IDLE_ROLL_AMP: f32 = 0.065;
const PREVIEW_IDLE_ROLL_HZ: f32 = 1.2;
/// Camera sits on -Z looking toward +Z; rotate mesh so the character faces the viewer (Kenney export +X).
const PREVIEW_FACE_CAMERA_Y: f32 = PI;

/// Kenney humanoids export facing +X; gameplay runs along +Z.
fn quat_face_run_dir() -> Quat {
    Quat::from_rotation_y(0.0)
}

pub struct GameRenderer {
    camera: GameCamera,
    model_manager: ModelManager,
    menu_bg: Option<Texture2D>,
    game_bg: Option<Texture2D>,
    font: Option<Font>,
    hud_cache: HudCache,
    /// Reused each frame for mesh draw helpers to avoid per-call `Vec<Vertex>` allocations.
    scratch_verts: Vec<Vertex>,
    /// Which character the menu preview idle phase is aligned to (`None` until first draw).
    preview_phase_choice: Option<CharacterChoice>,
    /// `get_time()` when the current preview idle phase started (reset on character change).
    preview_phase_start_time: f64,
}

#[derive(Default)]
struct HudCache {
    last_score: u32,
    last_coins: u32,
    last_stars: u32,
    last_distance: i32,
    last_combo: u32,
    score_text: String,
    coins_text: String,
    stars_text: String,
    distance_text: String,
    combo_text: String,
}

impl HudCache {
    fn new() -> Self {
        Self {
            last_score: u32::MAX,
            last_coins: u32::MAX,
            last_stars: u32::MAX,
            last_distance: i32::MIN,
            last_combo: u32::MAX,
            score_text: String::new(),
            coins_text: String::new(),
            stars_text: String::new(),
            distance_text: String::new(),
            combo_text: String::new(),
        }
    }

    fn update(&mut self, game_state: &GameState) {
        let score = game_state.score.floor() as u32;
        if score != self.last_score {
            self.last_score = score;
            self.score_text.clear();
            self.score_text.push_str("Score: ");
            self.score_text.push_str(&score.to_string());
        }

        if game_state.coins != self.last_coins {
            self.last_coins = game_state.coins;
            self.coins_text.clear();
            self.coins_text.push_str("Coins: ");
            self.coins_text.push_str(&game_state.coins.to_string());
        }

        if game_state.stars != self.last_stars {
            self.last_stars = game_state.stars;
            self.stars_text.clear();
            self.stars_text.push_str("Stars: ");
            self.stars_text.push_str(&game_state.stars.to_string());
        }

        let distance = game_state.distance as i32;
        if distance != self.last_distance {
            self.last_distance = distance;
            self.distance_text.clear();
            self.distance_text.push_str("Distance: ");
            self.distance_text.push_str(&distance.to_string());
            self.distance_text.push('m');
        }

        if game_state.combo != self.last_combo {
            self.last_combo = game_state.combo;
            self.combo_text.clear();
            self.combo_text.push_str("Combo: ");
            self.combo_text.push_str(&game_state.combo.to_string());
            self.combo_text.push('x');
        }
    }
}

impl GameRenderer {
    pub fn new() -> Self {
        Self {
            camera: GameCamera::new(),
            model_manager: ModelManager::new(),
            menu_bg: None,
            game_bg: None,
            font: None,
            hud_cache: HudCache::new(),
            scratch_verts: Vec::new(),
            preview_phase_choice: None,
            preview_phase_start_time: 0.0,
        }
    }

    pub async fn load_models(&mut self) {
        self.model_manager.load_models().await;

        match load_texture("images/toon_dash_background.png").await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                self.menu_bg = Some(tex);
                info!("Loaded menu background texture");
            }
            Err(e) => info!("Could not load menu background: {:?}", e),
        }

        // Load game background image
        match load_texture("images/sky.png").await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                self.game_bg = Some(tex);
                info!("Loaded game background texture");
            }
            Err(e) => info!("Could not load game background: {:?}", e),
        }

        // Load custom font
        match load_ttf_font("fonts/LuckiestGuy-Regular.ttf").await {
            Ok(f) => {
                self.font = Some(f);
                info!("Loaded LuckiestGuy font");
            }
            Err(e) => info!("Could not load font: {:?} — using default", e),
        }
    }

    pub fn update(&mut self, player: &Player, dt: f32) {
        self.camera
            .update(player.position.x, player.position.y, player.position.z, dt);
    }

    pub fn set_speed_factor(&mut self, total_speed: f32) {
        self.camera.set_dynamic_tuning(total_speed);
    }

    // ── Main render dispatch ────────────────────────────────────────────
    pub fn render(
        &mut self,
        game_state: &GameState,
        track: &Track,
        player: &Player,
        obstacle_manager: &ObstacleManager,
        collectible_manager: &CollectibleManager,
        menu_nav: &MenuNavigator<MenuOption>,
        pause_nav: &MenuNavigator<PauseOption>,
        gameover_nav: &MenuNavigator<GameOverOption>,
        sub_screen: &MenuSubScreen,
        game_settings: &GameSettings,
        lifetime_stats: &LifetimeStats,
        character_choice: &CharacterChoice,
        select_char_focused: bool,
        quit_confirm_close_focused: bool,
        allow_menu_preview_render: bool,
    ) {
        // Clear screen
        clear_background(Color::from_rgba(60, 60, 60, 255));

        // Draw the static 2D game background (sky, hills, etc) behind everything
        if let Some(tex) = &self.game_bg {
            draw_texture_ex(
                tex,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
        }
        match game_state.screen {
            GameScreen::MainMenu => {
                self.render_main_menu(
                    menu_nav,
                    sub_screen,
                    game_settings,
                    lifetime_stats,
                    character_choice,
                    select_char_focused,
                    quit_confirm_close_focused,
                    allow_menu_preview_render,
                );
            }
            GameScreen::Playing => {
                self.render_game(
                    track,
                    player,
                    obstacle_manager,
                    collectible_manager,
                    game_state,
                );
            }
            GameScreen::Paused => {
                self.render_game(
                    track,
                    player,
                    obstacle_manager,
                    collectible_manager,
                    game_state,
                );
                self.render_pause_menu(pause_nav, game_state, sub_screen);

                if *sub_screen == MenuSubScreen::Options {
                    self.render_options_overlay(game_settings);
                }
            }
            GameScreen::GameOver => {
                self.render_game(
                    track,
                    player,
                    obstacle_manager,
                    collectible_manager,
                    game_state,
                );
                self.render_game_over(gameover_nav, game_state);
            }
        }
    }

    // ── Gameplay rendering ──────────────────────────────────────────────
    fn render_game(
        &mut self,
        track: &Track,
        player: &Player,
        obstacle_manager: &ObstacleManager,
        collectible_manager: &CollectibleManager,
        game_state: &GameState,
    ) {
        self.camera.apply();

        let player_z = player.position.z;
        let view_dist = 120.0;
        let frame_time = get_time() as f32;

        self.render_ground(player_z, view_dist);
        self.render_decorations(player_z, view_dist);

        for segment in track.get_visible(player_z, view_dist) {
            self.render_track_segment(segment);
        }

        for obstacle in obstacle_manager.get_visible(player_z, view_dist) {
            self.render_obstacle(obstacle);
        }

        for item in collectible_manager.get_visible(player_z, view_dist) {
            if !item.collected {
                self.render_collectible(item, frame_time);
            }
        }

        self.render_player(player);

        GameCamera::set_ui_camera();
        self.render_hud(game_state);
    }

    fn render_ground(&self, player_z: f32, view_dist: f32) {
        let ground_color = Color::from_rgba(72, 140, 72, 255);
        let zc = player_z + view_dist * 0.25;
        let zlen = view_dist * 1.55;
        let half_road = 1.5 * ROAD_TILE_STEP;
        let side_w = 22.0;

        draw_cube(
            vec3(-half_road - side_w * 0.5, -0.4, zc),
            vec3(side_w, 0.55, zlen),
            None,
            ground_color,
        );
        draw_cube(
            vec3(half_road + side_w * 0.5, -0.4, zc),
            vec3(side_w, 0.55, zlen),
            None,
            ground_color,
        );
    }

    fn render_decorations(&mut self, player_z: f32, view_dist: f32) {
        // Deterministic hash based on Z-segment
        let start_z = (player_z / ROAD_TILE_STEP).floor() as i32;
        let end_z = ((player_z + view_dist) / ROAD_TILE_STEP).ceil() as i32;
        let half_road = 1.5 * ROAD_TILE_STEP;

        let natural_decors = [
            "dec_tree",
            "dec_tree_pine",
            "dec_rocks",
            "dec_stones",
            "dec_mushrooms",
            "dec_flowers_tall",
            "dec_plant",
            "dec_grass",
        ];
        let urban_decors = ["dec_light", "dec_cone", "dec_clight", "dec_barrier"];

        for z_idx in start_z..=end_z {
            // Very simple deterministic linear congruential hash
            let hash1 = (z_idx.wrapping_mul(1234567)).wrapping_add(9876543) as u32;
            let hash2 = (z_idx.wrapping_mul(7654321)).wrapping_add(3456789) as u32;
            let hash3 = (z_idx.wrapping_mul(1357911)).wrapping_add(2468101) as u32;

            let z_world = z_idx as f32 * ROAD_TILE_STEP;

            // Maybe spawn a natural decoration on the left
            if hash1 % 100 < 40 {
                // 40% chance
                let decor_idx = (hash1 / 100) as usize % natural_decors.len();
                let x_offset = 2.0 + (hash1 % 15) as f32; // Spread 2 to 17 units away
                let rot =
                    Quat::from_rotation_y((hash1 % 360) as f32 * std::f32::consts::PI / 180.0);
                let scale = 1.0 + (hash1 % 10) as f32 * 0.05;
                if let Some(mesh) = self.model_manager.mesh(natural_decors[decor_idx]) {
                    draw_mesh_at_rot(
                        mesh,
                        vec3(-half_road - x_offset, -0.1, z_world),
                        2.0 * scale,
                        rot,
                        &mut self.scratch_verts,
                    );
                }
            }

            // Maybe spawn a natural decoration on the right
            if hash2 % 100 < 40 {
                // 40% chance
                let decor_idx = (hash2 / 100) as usize % natural_decors.len();
                let x_offset = 2.0 + (hash2 % 15) as f32; // Spread 2 to 17 units away
                let rot =
                    Quat::from_rotation_y((hash2 % 360) as f32 * std::f32::consts::PI / 180.0);
                let scale = 1.0 + (hash2 % 10) as f32 * 0.05;
                if let Some(mesh) = self.model_manager.mesh(natural_decors[decor_idx]) {
                    draw_mesh_at_rot(
                        mesh,
                        vec3(half_road + x_offset, -0.1, z_world),
                        2.0 * scale,
                        rot,
                        &mut self.scratch_verts,
                    );
                }
            }

            // Urban decorations (lights, cones) highly structured near edge of the road
            if hash3 % 100 < 15 {
                // 15% chance for a roadside element
                let decor_idx = (hash3 / 100) as usize % urban_decors.len();
                let is_left = hash3 % 2 == 0;
                let x_pos = if is_left {
                    -half_road - 0.5
                } else {
                    half_road + 0.5
                };

                // Base Y rotation to face the road (flipped by PI to face inward correctly)
                let rot = if is_left {
                    Quat::from_rotation_y(-std::f32::consts::PI / 2.0)
                } else {
                    Quat::from_rotation_y(std::f32::consts::PI / 2.0)
                };

                let decor_name = urban_decors[decor_idx];
                let mut scale = 4.0; // Doubled the default size for construction items

                // Make the streetlights significantly bigger as requested
                if decor_name == "dec_light" {
                    scale = 8.0;
                }

                if let Some(mesh) = self.model_manager.mesh(decor_name) {
                    draw_mesh_at_rot(
                        mesh,
                        vec3(x_pos, 0.0, z_world),
                        scale,
                        rot,
                        &mut self.scratch_verts,
                    );
                }
            }
        }
    }

    fn render_player(&mut self, player: &Player) {
        let pos = player.position.to_vec3();
        let color = self.model_manager.get_color("character");

        if let Some(mesh) = self.model_manager.mesh("character") {
            let mut scale = vec3(2.0, 2.0, 2.0);
            let mut rot = quat_face_run_dir();
            let mut pivot = pos + vec3(0.0, PLAYER_MESH_PIVOT_Y, 0.0);

            match player.state {
                PlayerState::Running => {
                    let bob = (player.animation_tick * 10.0).sin().abs() * 0.15;
                    let wobble = (player.animation_tick * 5.0).sin() * 0.08;
                    pivot.y += bob;
                    rot *= Quat::from_rotation_z(wobble);
                }
                PlayerState::Jumping => {
                    let flip_angle = player.jump_progress * std::f32::consts::TAU;
                    rot *= Quat::from_rotation_x(flip_angle);
                }
                PlayerState::Sliding => {
                    scale = vec3(1.2, 0.4, 1.2) * 1.1;
                    pivot.y -= 0.2;
                }
                PlayerState::Dead => {
                    rot *= Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                    pivot.y -= 0.3;
                }
            }

            draw_mesh_at_transform(mesh, pivot, scale, rot, &mut self.scratch_verts);
        } else {
            let (height, y_offset) = if player.is_sliding() {
                (0.5, 0.25)
            } else {
                (1.4, 0.7)
            };
            draw_cube(
                pos + vec3(0.0, y_offset, 0.0),
                vec3(0.8, height, 0.6),
                None,
                color,
            );
            if !player.is_sliding() {
                draw_sphere(
                    pos + vec3(0.0, 1.6, 0.0),
                    0.3,
                    None,
                    Color::from_rgba(255, 200, 160, 255),
                );
            }
        }
    }

    fn render_obstacle(&mut self, obstacle: &Obstacle) {
        let pos = obstacle.position.to_vec3();
        let border = Color::from_rgba(30, 30, 30, 255);

        match obstacle.obstacle_type {
            ObstacleType::LowBarrier => {
                if let Some(mesh) = self.model_manager.mesh("obstacle_low") {
                    draw_mesh_at_rot(
                        mesh,
                        pos + vec3(0.0, ROAD_SURFACE_Y, 0.0),
                        2.0,
                        Quat::from_rotation_y(0.0),
                        &mut self.scratch_verts,
                    );
                } else {
                    let color = self.model_manager.get_color("obstacle_low");
                    draw_cube(pos + vec3(0.0, 0.5, 0.0), vec3(1.4, 1.0, 0.8), None, color);
                    draw_cube(pos + vec3(0.0, 0.5, 0.0), vec3(1.5, 0.1, 0.9), None, border);
                }
            }
            ObstacleType::HighBarrier => {
                if let Some(mesh) = self.model_manager.mesh("obstacle_high") {
                    draw_mesh_at_rot(
                        mesh,
                        pos + vec3(0.0, 0.45, 0.0),
                        1.2,
                        Quat::from_rotation_y(0.0),
                        &mut self.scratch_verts,
                    );
                } else {
                    let color = self.model_manager.get_color("obstacle_high");
                    draw_cube(pos + vec3(0.0, 1.6, 0.0), vec3(1.4, 0.4, 0.8), None, color);
                    let pillar = Color::from_rgba(180, 120, 50, 255);
                    draw_cube(
                        pos + vec3(-0.5, 0.9, 0.0),
                        vec3(0.15, 1.8, 0.15),
                        None,
                        pillar,
                    );
                    draw_cube(
                        pos + vec3(0.5, 0.9, 0.0),
                        vec3(0.15, 1.8, 0.15),
                        None,
                        pillar,
                    );
                }
            }
            ObstacleType::FullBarrier => {
                if let Some(mesh) = self.model_manager.mesh("obstacle_full") {
                    draw_mesh_at_rot(
                        mesh,
                        pos + vec3(0.0, ROAD_SURFACE_Y, 0.0),
                        2.2,
                        Quat::from_rotation_y(0.0),
                        &mut self.scratch_verts,
                    );
                } else {
                    let color = self.model_manager.get_color("obstacle_full");
                    draw_cube(pos + vec3(0.0, 1.2, 0.0), vec3(1.0, 2.4, 0.6), None, color);
                    draw_cube(pos + vec3(0.0, 1.2, 0.0), vec3(1.1, 2.5, 0.1), None, border);
                }
            }
        }
    }

    fn render_collectible(&mut self, item: &Collectible, frame_time: f32) {
        let pos = item.position.to_vec3();
        let (mesh_key, default_color) = match item.ctype {
            CollectibleType::Coin => ("coin", self.model_manager.get_color("coin")),
            CollectibleType::Jewel => ("jewel", Color::from_rgba(180, 50, 255, 255)), // Jewel color fallback
            CollectibleType::Star => ("star", self.model_manager.get_color("star")),
        };

        let bob = (frame_time * 3.0 + pos.z * 0.5).sin() * 0.15;
        let pulse = 0.9 + (frame_time * 4.0).sin() * 0.1;

        let item_pos = pos + vec3(0.0, ROAD_SURFACE_Y + 0.35 + bob, 0.0);
        let spin = Quat::from_rotation_y(frame_time * 3.0);

        if let Some(mesh) = self.model_manager.mesh(mesh_key) {
            draw_mesh_at_rot(
                mesh,
                item_pos,
                2.8 * pulse,
                spin,
                &mut self.scratch_verts,
            );
        } else {
            draw_sphere(item_pos, 0.3 * pulse, None, default_color);
        }
    }

    fn render_track_segment(&mut self, segment: &TrackSegment) {
        let z = segment.z_position;
        let color = self.model_manager.get_color("track_segment");
        let rot = Quat::from_rotation_y(-FRAC_PI_2);

        if let Some(mesh) = self.model_manager.mesh("road_straight") {
            for ix in -1..=1 {
                for iz in 0..5 {
                    let ox = ix as f32 * ROAD_TILE_STEP;
                    let oz = z - 4.0 + iz as f32 * ROAD_TILE_STEP;
                    draw_mesh_at_rot(
                        mesh,
                        vec3(ox, ROAD_SURFACE_Y, oz),
                        ROAD_TILE_SCALE,
                        rot,
                        &mut self.scratch_verts,
                    );
                }
            }
        } else {
            draw_cube(vec3(0.0, -0.25, z), vec3(7.0, 0.5, 10.0), None, color);
        }
    }

    fn render_hud(&mut self, game_state: &GameState) {
        self.hud_cache.update(game_state);

        // Larger HUD panel for 1080p TVs
        draw_rectangle(20.0, 20.0, 340.0, 200.0, Color::from_rgba(0, 0, 0, 150));

        self.draw_font_text(
            &self.hud_cache.score_text,
            35.0,
            65.0,
            44,
            WHITE,
        );
        self.draw_font_text(
            &self.hud_cache.coins_text,
            35.0,
            110.0,
            38,
            Color::from_rgba(255, 200, 50, 255),
        );
        self.draw_font_text(
            &self.hud_cache.distance_text,
            35.0,
            150.0,
            32,
            Color::from_rgba(180, 180, 200, 255),
        );
        self.draw_font_text(
            &self.hud_cache.stars_text,
            35.0,
            188.0,
            32,
            Color::from_rgba(255, 235, 120, 255),
        );

        // ── Combo Multiplier Label ──────────────────────────────────────
        let sw = screen_width();
        let sh = screen_height();
        let combo_text = &self.hud_cache.combo_text;

        // Horizontal center, bottom of screen
        self.draw_font_text_centered(
            combo_text,
            sw / 2.0,
            sh - 36.0,
            65,
            if game_state.combo > 1 { YELLOW } else { WHITE },
        );
    }

    // ── MAIN MENU ───────────────────────────────────────────────────────
    fn render_main_menu(
        &mut self,
        menu_nav: &MenuNavigator<MenuOption>,
        sub_screen: &MenuSubScreen,
        game_settings: &GameSettings,
        lifetime_stats: &LifetimeStats,
        character_choice: &CharacterChoice,
        select_char_focused: bool,
        quit_confirm_close_focused: bool,
        allow_menu_preview_render: bool,
    ) {
        let sw = screen_width();
        let sh = screen_height();

        // Background image (fullscreen)
        if let Some(bg) = &self.menu_bg {
            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw, sh)),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(18, 32, 72, 255));
        }

        // ── Layout constants ────────────────────────────────────────────
        let btn_w = 360.0;
        let btn_h = 90.0;
        let left_col_x = (sw - btn_w) / 2.0;
        let menu_start_y = sh * 0.45;
        let btn_spacing = 135.0;

        draw_rectangle(25.0, 25.0, 370.0, 190.0, Color::from_rgba(0, 0, 0, 140));
        draw_rectangle_lines(
            25.0,
            25.0,
            370.0,
            190.0,
            2.0,
            Color::from_rgba(255, 255, 255, 100),
        );
        self.draw_font_text("LIFETIME", 42.0, 62.0, 34, WHITE);
        self.draw_font_text(
            &format!("High Score: {}", lifetime_stats.high_score),
            42.0,
            100.0,
            26,
            YELLOW,
        );
        self.draw_font_text(
            &format!("Coins Earned: {}", lifetime_stats.total_coins),
            42.0,
            132.0,
            24,
            Color::from_rgba(255, 210, 90, 255),
        );
        self.draw_font_text(
            &format!("Stars Earned: {}", lifetime_stats.total_stars),
            42.0,
            160.0,
            24,
            Color::from_rgba(255, 235, 120, 255),
        );
        self.draw_font_text(
            &format!("Distance: {}m", lifetime_stats.total_distance),
            42.0,
            188.0,
            24,
            Color::from_rgba(180, 180, 200, 255),
        );

        // ── Left column: 4 buttons ──────────────────────────────────────
        for (i, option) in menu_nav.options.iter().enumerate() {
            let text = match option {
                MenuOption::Play => "PLAY",
                MenuOption::HowToPlay => "HOW TO PLAY",
                MenuOption::Options => "OPTIONS",
                MenuOption::Quit => "QUIT",
            };
            let is_focused = i == menu_nav.selected
                && *sub_screen == MenuSubScreen::None
                && !select_char_focused;
            let btn_y = menu_start_y + (i as f32 * btn_spacing);
            Self::draw_ui_button(
                left_col_x,
                btn_y,
                btn_w,
                btn_h,
                text,
                is_focused,
                self.font.as_ref(),
            );
        }

        // ── Right panel: character showcase ─────────────────────────────
        // Match panel height to the total buttons stack
        let total_buttons_h = (menu_nav.options.len() as f32 - 1.0) * btn_spacing + btn_h;
        let panel_w = sw * 0.22;
        let panel_h = total_buttons_h;
        let panel_x = left_col_x + btn_w + 30.0;
        let panel_y = menu_start_y;

        // Semi-transparent background
        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::from_rgba(0, 0, 0, 80),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            3.0,
            Color::from_rgba(255, 255, 255, 60),
        );

        // Layout inside panel: character preview takes the top portion,
        // name sits below it, button at the bottom
        let select_btn_h_val = 42.0;
        let name_area_h = 35.0;
        let bottom_padding = 15.0;
        let char_preview_h = panel_h - select_btn_h_val - name_area_h - bottom_padding - 10.0;

        // 3D character idle preview (centered in upper area)
        if allow_menu_preview_render {
            self.render_character_preview(
                character_choice,
                panel_x,
                panel_y,
                panel_w,
                char_preview_h,
                0.6, // model_y
                2.0, // model_scale
            );
        }

        // Character name (below the 3D model)
        let name = character_choice.display_name();
        if allow_menu_preview_render {
            self.draw_font_text_centered(
                name,
                panel_x + panel_w / 2.0,
                panel_y + char_preview_h + 35.0,
                44,
                WHITE,
            );
        }

        // "SELECT CHARACTER" button (at bottom of panel)
        let select_btn_w = (panel_w - 20.0).max(220.0);
        let select_btn_h = 75.0;
        let select_btn_x = panel_x + (panel_w - select_btn_w) / 2.0;
        let select_btn_y = panel_y + panel_h - select_btn_h - 25.0;
        Self::draw_ui_button(
            select_btn_x,
            select_btn_y,
            select_btn_w,
            select_btn_h,
            "SELECT CHARACTER",
            select_char_focused,
            self.font.as_ref(),
        );

        // ── Overlays ────────────────────────────────────────────────────
        match sub_screen {
            MenuSubScreen::HowToPlay => self.render_how_to_play_overlay(),
            MenuSubScreen::Options => self.render_options_overlay(game_settings),
            MenuSubScreen::CharacterSelect => {
                self.render_character_select_overlay(character_choice)
            }
            MenuSubScreen::QuitConfirm => {
                self.render_quit_confirm_overlay(quit_confirm_close_focused)
            }
            MenuSubScreen::None => {}
        }
    }

    /// Render a front-facing idle 3D character preview inside a screen-space rectangle.
    fn render_character_preview(
        &mut self,
        choice: &CharacterChoice,
        panel_x: f32,
        panel_y: f32,
        panel_w: f32,
        panel_h: f32,
        model_y: f32,
        model_scale: f32,
    ) {
        let mesh_key = choice.mesh_key();
        if let Some(mesh) = self.model_manager.mesh(mesh_key) {
            let now = get_time();
            if self.preview_phase_choice != Some(*choice) {
                self.preview_phase_choice = Some(*choice);
                self.preview_phase_start_time = now;
            }
            let phase_t = (now - self.preview_phase_start_time) as f32;

            let bob_y = PREVIEW_IDLE_BOB_AMP * (phase_t * PREVIEW_IDLE_BOB_HZ * std::f32::consts::TAU).sin();
            let yaw = PREVIEW_IDLE_YAW_AMP * (phase_t * PREVIEW_IDLE_YAW_HZ * std::f32::consts::TAU).sin();
            let roll = PREVIEW_IDLE_ROLL_AMP * (phase_t * PREVIEW_IDLE_ROLL_HZ * std::f32::consts::TAU).sin();
            let rot = Quat::from_rotation_y(yaw)
                * Quat::from_rotation_z(roll)
                * Quat::from_rotation_y(PREVIEW_FACE_CAMERA_Y)
                * quat_face_run_dir();

            let vp_x = panel_x.max(0.0) as i32;
            let vp_y = (screen_height() - panel_y - panel_h).max(0.0) as i32;
            let vp_w = panel_w.max(1.0) as i32;
            let vp_h = panel_h.max(1.0) as i32;

            let aspect = panel_w / panel_h;
            set_camera(&Camera3D {
                position: vec3(0.0, 2.0, -3.5),
                target: vec3(0.0, 1.5, 0.0),
                up: Vec3::Y,
                fovy: 45.0_f32.to_radians(),
                aspect: Some(aspect),
                projection: Projection::Perspective,
                render_target: None,
                viewport: Some((vp_x, vp_y, vp_w, vp_h)),
                z_near: 0.1,
                z_far: 80.0,
            });

            draw_mesh_at_rot(
                mesh,
                vec3(0.0, model_y + bob_y, 0.0),
                model_scale,
                rot,
                &mut self.scratch_verts,
            );

            set_default_camera();
        }
    }

    // ── How to Play overlay ─────────────────────────────────────────────
    fn render_how_to_play_overlay(&self) {
        let sw = screen_width();
        let sh = screen_height();

        // Dimmed backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));

        // Box
        let box_w = sw * 0.55;
        let box_h = sh * 0.70;
        let box_x = (sw - box_w) / 2.0;
        let box_y = (sh - box_h) / 2.0;

        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            Color::from_rgba(30, 30, 50, 220),
        );
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            3.0,
            Color::from_rgba(255, 200, 50, 200),
        );

        // Title
        self.draw_font_text_centered(
            "HOW TO PLAY",
            sw / 2.0,
            box_y + 80.0,
            64,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Instructions
        let lines = [
            "LEFT / RIGHT  -  Change Lane",
            "UP  -  Jump (dodge low barriers)",
            "DOWN  -  Slide (dodge high barriers)",
            "ENTER  -  Select / Confirm",
            "BACK  -  Pause",
            "",
            "Collect coins for extra score!",
            "Full barriers must be dodged",
            "by switching lanes.",
            "",
            "You can cancel a slide into a",
            "jump, and a jump into a slide!",
        ];

        let start_y = box_y + 160.0;
        for (i, line) in lines.iter().enumerate() {
            self.draw_font_text_centered(line, sw / 2.0, start_y + i as f32 * 45.0, 32, WHITE);
        }

        // Close hint
        self.draw_font_text_centered(
            "Press ENTER or BACK to close",
            sw / 2.0,
            box_y + box_h - 20.0,
            18,
            Color::from_rgba(180, 180, 200, 255),
        );
    }

    // ── Options overlay ─────────────────────────────────────────────────
    fn render_options_overlay(&self, game_settings: &GameSettings) {
        let sw = screen_width();
        let sh = screen_height();

        // Dimmed backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));

        // Box (Larger for 1080p)
        let box_w = sw * 0.70;
        let box_h = sh * 0.85;
        let box_x = (sw - box_w) / 2.0;
        let box_y = (sh - box_h) / 2.0;

        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            Color::from_rgba(30, 30, 50, 220),
        );
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            4.0,
            Color::from_rgba(255, 200, 50, 200),
        );

        // Title
        self.draw_font_text_centered(
            "OPTIONS",
            sw / 2.0,
            box_y + 80.0,
            64,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Volume and Settings rows
        let row_labels = [
            "Master Volume",
            "Music Volume",
            "Effects Volume",
            "Game Speed",
        ];
        let row_values = [
            game_settings.master_volume,
            game_settings.music_volume,
            game_settings.effects_volume,
            game_settings.game_speed,
        ];
        let row_y_start = box_y + 160.0;

        for (i, (label, val)) in row_labels.iter().zip(row_values.iter()).enumerate() {
            let y = row_y_start + i as f32 * 180.0 + 20.0;
            let is_focused = game_settings.focused_row == i;
            let label_color = if is_focused {
                Color::from_rgba(58, 227, 58, 255)
            } else {
                WHITE
            };

            // Label
            self.draw_font_text_centered(label, sw / 2.0, y, 44, label_color);

            // Volume bar
            let bar_w = box_w * 0.65;
            let bar_h = 32.0;
            let bar_x = (sw - bar_w) / 2.0;
            let bar_y = y + 40.0;

            // Background
            draw_rectangle(
                bar_x,
                bar_y,
                bar_w,
                bar_h,
                Color::from_rgba(60, 60, 80, 255),
            );

            // Fill
            let fill_w = bar_w * (*val as f32 / 10.0);
            let fill_color = if is_focused {
                Color::from_rgba(58, 227, 58, 255)
            } else {
                Color::from_rgba(70, 130, 180, 255)
            };
            draw_rectangle(bar_x, bar_y, fill_w, bar_h, fill_color);

            // Border
            draw_rectangle_lines(
                bar_x,
                bar_y,
                bar_w,
                bar_h,
                3.0,
                Color::from_rgba(200, 200, 220, 180),
            );

            // Value text
            let val_text = if i == 3 {
                format!("{}%", val * 10)
            } else {
                format!("{}", val)
            };
            self.draw_font_text_centered(&val_text, sw / 2.0, bar_y + bar_h + 38.0, 32, WHITE);

            // Arrow hints
            if is_focused {
                self.draw_font_text(
                    "<",
                    bar_x - 50.0,
                    bar_y + 25.0,
                    44,
                    Color::from_rgba(58, 227, 58, 255),
                );
                self.draw_font_text(
                    ">",
                    bar_x + bar_w + 15.0,
                    bar_y + 25.0,
                    44,
                    Color::from_rgba(58, 227, 58, 255),
                );
            }
        }

        // Close hint
        self.draw_font_text_centered(
            "PRESS BACK TO CLOSE",
            sw / 2.0,
            box_y + box_h - 35.0,
            32,
            Color::from_rgba(180, 180, 200, 255),
        );
    }

    // ── Character Select overlay ────────────────────────────────────────
    fn render_character_select_overlay(&mut self, choice: &CharacterChoice) {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));

        let box_w = sw * 0.65;
        let box_h = sh * 0.70;
        let box_x = (sw - box_w) / 2.0;
        let box_y = (sh - box_h) / 2.0;

        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            Color::from_rgba(30, 30, 50, 220),
        );
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            4.0,
            Color::from_rgba(255, 200, 50, 200),
        );

        self.draw_font_text_centered(
            "CHOOSE CHARACTER",
            sw / 2.0,
            box_y + box_h * 0.10,
            64,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Preview area
        let preview_top = box_y + box_h * 0.25;
        let char_preview_h = box_h * 0.44;
        self.render_character_preview(
            choice,
            box_x,
            preview_top,
            box_w,
            char_preview_h,
            0.5,
            2.5,
        );

        // Name and hints
        let name_y = box_y + box_h * 0.78;
        let nav_y = box_y + box_h * 0.87;
        let hint_y = box_y + box_h * 0.94;
        self.draw_font_text_centered(
            choice.display_name(),
            sw / 2.0,
            name_y,
            55,
            WHITE,
        );
        self.draw_font_text_centered(
            "< PREVIOUS     |     NEXT >",
            sw / 2.0,
            nav_y,
            32,
            Color::from_rgba(180, 180, 200, 255),
        );
        self.draw_font_text_centered(
            "Press ENTER to Select",
            sw / 2.0,
            hint_y,
            32,
            Color::from_rgba(255, 200, 50, 255),
        );
    }

    fn render_quit_confirm_overlay(&self, close_focused: bool) {
        let sw = screen_width();
        let sh = screen_height();
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 180));

        let box_w = sw * 0.55;
        let box_h = sh * 0.38;
        let box_x = (sw - box_w) / 2.0;
        let box_y = (sh - box_h) / 2.0;
        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            Color::from_rgba(30, 30, 50, 235),
        );
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            4.0,
            Color::from_rgba(255, 200, 50, 200),
        );

        self.draw_font_text_centered(
            "ARE YOU SURE TO CLOSE THE GAME?",
            sw / 2.0,
            box_y + 85.0,
            48,
            WHITE,
        );

        let btn_w = box_w * 0.36;
        let btn_h = 86.0;
        let gap = box_w * 0.08;
        let total_w = btn_w * 2.0 + gap;
        let start_x = box_x + (box_w - total_w) / 2.0;
        let btn_y = box_y + box_h - 135.0;

        // Change this red to customize the "Close game" button color.
        let close_color = Color::from_rgba(200, 40, 40, 255);
        let close_highlight = Color::from_rgba(225, 70, 70, 255);
        // Change this green to customize the "Continue playing" button color.
        let continue_color = Color::from_rgba(40, 160, 40, 255);
        let continue_highlight = Color::from_rgba(70, 190, 70, 255);

        Self::draw_ui_button_with_colors(
            start_x,
            btn_y,
            btn_w,
            btn_h,
            "CLOSE GAME",
            close_focused,
            self.font.as_ref(),
            close_color,
            close_highlight,
        );
        Self::draw_ui_button_with_colors(
            start_x + btn_w + gap,
            btn_y,
            btn_w,
            btn_h,
            "CONTINUE PLAYING",
            !close_focused,
            self.font.as_ref(),
            continue_color,
            continue_highlight,
        );
    }

    // ── Pause Menu ──────────────────────────────────────────────────────
    fn render_pause_menu(
        &self,
        pause_nav: &MenuNavigator<PauseOption>,
        game_state: &GameState,
        sub_screen: &MenuSubScreen,
    ) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(0, 0, 30, 180),
        );
        self.draw_font_text_centered("PAUSED", cx, cy - 280.0, 90, WHITE);
        self.draw_font_text_centered(
            &format!("Score: {}", game_state.score.floor() as u32),
            cx,
            cy - 180.0,
            44,
            Color::from_rgba(200, 200, 220, 255),
        );

        let btn_w = 360.0;
        let btn_h = 90.0;
        let btn_x = cx - (btn_w / 2.0);
        let start_y = cy - 80.0;
        let btn_spacing = 135.0;

        for (i, option) in pause_nav.options.iter().enumerate() {
            let text = match option {
                PauseOption::Resume => "RESUME",
                PauseOption::Restart => "RESTART",
                PauseOption::Options => "OPTIONS",
                PauseOption::Quit => "MAIN MENU",
            };
            let is_focused = i == pause_nav.selected && *sub_screen == MenuSubScreen::None;

            Self::draw_ui_button(
                btn_x,
                start_y + (i as f32 * btn_spacing),
                btn_w,
                btn_h,
                text,
                is_focused,
                self.font.as_ref(),
            );
        }
    }

    // ── Game Over ───────────────────────────────────────────────────────
    fn render_game_over(
        &self,
        gameover_nav: &MenuNavigator<GameOverOption>,
        game_state: &GameState,
    ) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(50, 20, 20, 200),
        );
        self.draw_font_text_centered("GAME OVER", cx, cy - 280.0, 90, RED);
        self.draw_font_text_centered(
            &format!("Score: {}", game_state.score.floor() as u32),
            cx,
            cy - 180.0,
            48,
            WHITE,
        );
        self.draw_font_text_centered(
            &format!("High Score: {}", game_state.high_score.floor() as u32),
            cx,
            cy - 100.0,
            36,
            YELLOW,
        );
        self.draw_font_text_centered(
            &format!("Stars: {}", game_state.stars),
            cx,
            cy - 45.0,
            34,
            Color::from_rgba(255, 235, 120, 255),
        );
        self.draw_font_text_centered(
            &format!("Coins: {}", game_state.coins),
            cx,
            cy + 2.0,
            32,
            Color::from_rgba(255, 210, 90, 255),
        );

        let btn_w = 360.0;
        let btn_h = 90.0;
        let btn_x = cx - (btn_w / 2.0);
        let start_y = cy + 40.0;
        let btn_spacing = 135.0;

        for (i, option) in gameover_nav.options.iter().enumerate() {
            let text = match option {
                GameOverOption::Restart => "PLAY AGAIN",
                GameOverOption::Quit => "MAIN MENU",
            };
            let is_focused = i == gameover_nav.selected;

            Self::draw_ui_button(
                btn_x,
                start_y + (i as f32 * btn_spacing),
                btn_w,
                btn_h,
                text,
                is_focused,
                self.font.as_ref(),
            );
        }
    }

    // ── Camera snap ─────────────────────────────────────────────────────
    pub fn snap_camera(&mut self, player_x: f32, player_y: f32, player_z: f32) {
        self.camera.snap(player_x, player_y, player_z);
    }

    // ── Shared UI helpers ───────────────────────────────────────────────

    /// Draw text using the custom font (or fallback to default).
    fn draw_font_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
        let shadow_offset = (size as f32 * 0.1).max(2.0); // 10% of size, min 2px
                                                          // 1. Draw black shadow
        draw_text_ex(
            text,
            x + shadow_offset,
            y + shadow_offset,
            TextParams {
                font_size: size,
                font: self.font.as_ref(),
                color: BLACK,
                ..Default::default()
            },
        );
        // 2. Draw main text
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font_size: size,
                font: self.font.as_ref(),
                color,
                ..Default::default()
            },
        );
    }

    /// Draw text centered horizontally at `cx`.
    fn draw_font_text_centered(&self, text: &str, cx: f32, y: f32, size: u16, color: Color) {
        let dim = measure_text(text, self.font.as_ref(), size, 1.0);
        self.draw_font_text(text, cx - dim.width / 2.0, y, size, color);
    }

    /// Styled button with shadow, border, highlight, and centered text.
    pub fn draw_ui_button(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: &str,
        is_focused: bool,
        font: Option<&Font>,
    ) {
        let active_btn_color = Color::from_rgba(58, 227, 58, 255);
        let active_highlight = Color::from_rgba(100, 240, 100, 255);
        let idle_btn_color = Color::from_rgba(70, 130, 180, 255);
        let idle_highlight = Color::from_rgba(100, 160, 210, 255);
        let (btn_color, highlight) = if is_focused {
            (active_btn_color, active_highlight)
        } else {
            (idle_btn_color, idle_highlight)
        };
        Self::draw_ui_button_with_colors(x, y, w, h, text, is_focused, font, btn_color, highlight);
    }

    fn draw_ui_button_with_colors(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: &str,
        is_focused: bool,
        font: Option<&Font>,
        btn_color: Color,
        highlight: Color,
    ) {
        let dark_gray = Color::from_rgba(51, 51, 51, 255);

        // Shadow + border + face + highlight
        draw_rectangle(x + 6.0, y + 6.0, w, h, dark_gray);
        draw_rectangle(x - 4.0, y - 4.0, w + 8.0, h + 8.0, dark_gray);
        draw_rectangle(x, y, w, h, btn_color);
        draw_rectangle(x, y, w, h * 0.25, highlight);
        if is_focused {
            draw_rectangle_lines(
                x - 6.0,
                y - 6.0,
                w + 12.0,
                h + 12.0,
                4.0,
                Color::from_rgba(255, 255, 255, 230),
            );
        }

        // Text
        let font_size = h * 0.5;
        let text_params = TextParams {
            font_size: font_size as u16,
            font,
            color: WHITE,
            ..Default::default()
        };

        let text_dim = measure_text(text, font, font_size as u16, 1.0);
        let text_x = x + (w - text_dim.width) / 2.0;
        let text_y = y + (h + text_dim.height) / 2.0 - text_dim.offset_y / 2.0;

        // Shadow then text
        draw_text_ex(
            text,
            text_x + 2.0,
            text_y + 2.0,
            TextParams {
                color: dark_gray,
                ..text_params.clone()
            },
        );
        draw_text_ex(text, text_x, text_y, text_params);
    }

    /// Get the mesh key for the gameplay character based on selected choice.
    pub fn set_active_character(&mut self, choice: &CharacterChoice) {
        // Copy the selected character mesh into the "character" slot
        // so the gameplay renderer uses it automatically.
        let key = choice.mesh_key();
        if let Some(mesh) = self.model_manager.mesh(key) {
            let cloned = Mesh {
                vertices: mesh.vertices.clone(),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            };
            self.model_manager.set_mesh("character", cloned);
        }
    }
}

impl Default for GameRenderer {
    fn default() -> Self {
        Self::new()
    }
}
