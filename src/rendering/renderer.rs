// src/rendering/renderer.rs
//! Game renderer with Kenney-style graphics

use super::camera::GameCamera;
use crate::game::*;
use crate::models::{draw_mesh_at_rot, draw_mesh_at_transform, ModelManager};
use glam::Quat;
use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

/// City Kit `road-straight` tile: scale/step tuned so ~5 tiles cover segment length (~10).
const ROAD_TILE_SCALE: f32 = 2.35;
const ROAD_TILE_STEP: f32 = 2.35;
const ROAD_SURFACE_Y: f32 = 0.03;
/// Character mesh pivot so feet sit on the road surface.
const PLAYER_MESH_PIVOT_Y: f32 = 0.58;

/// Kenney humanoids export facing +X; gameplay runs along +Z.
fn quat_face_run_dir() -> Quat {
    Quat::from_rotation_y(0.0)
}

pub struct GameRenderer {
    camera: GameCamera,
    model_manager: ModelManager,
    menu_bg: Option<Texture2D>,
    font: Option<Font>,
}

impl GameRenderer {
    pub fn new() -> Self {
        Self {
            camera: GameCamera::new(),
            model_manager: ModelManager::new(),
            menu_bg: None,
            font: None,
        }
    }

    pub async fn load_models(&mut self) {
        self.model_manager.load_models().await;

        // Load main menu background image
        match load_texture("images/toon_dash_background.png").await {
            Ok(tex) => {
                tex.set_filter(FilterMode::Linear);
                self.menu_bg = Some(tex);
                info!("Loaded menu background texture");
            }
            Err(e) => info!("Could not load menu background: {:?}", e),
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

    // ── Main render dispatch ────────────────────────────────────────────
    pub fn render(
        &self,
        game_state: &GameState,
        track: &Track,
        player: &Player,
        obstacle_manager: &ObstacleManager,
        coin_manager: &CoinManager,
        menu_nav: &MenuNavigator<MenuOption>,
        pause_nav: &MenuNavigator<PauseOption>,
        gameover_nav: &MenuNavigator<GameOverOption>,
        sub_screen: &MenuSubScreen,
        audio: &AudioSettings,
        character_choice: &CharacterChoice,
        select_char_focused: bool,
    ) {
        clear_background(Color::from_rgba(135, 206, 250, 255)); // Sky blue

        match game_state.screen {
            GameScreen::MainMenu => {
                self.render_main_menu(
                    menu_nav,
                    sub_screen,
                    audio,
                    character_choice,
                    select_char_focused,
                );
            }
            GameScreen::Playing => {
                self.render_game(track, player, obstacle_manager, coin_manager, game_state);
            }
            GameScreen::Paused => {
                self.render_game(track, player, obstacle_manager, coin_manager, game_state);
                self.render_pause_menu(pause_nav, game_state);
            }
            GameScreen::GameOver => {
                self.render_game(track, player, obstacle_manager, coin_manager, game_state);
                self.render_game_over(gameover_nav, game_state);
            }
        }
    }

    // ── Gameplay rendering ──────────────────────────────────────────────
    fn render_game(
        &self,
        track: &Track,
        player: &Player,
        obstacle_manager: &ObstacleManager,
        coin_manager: &CoinManager,
        game_state: &GameState,
    ) {
        self.camera.apply();

        let player_z = player.position.z;
        let view_dist = 120.0;

        self.render_ground(player_z, view_dist);

        for segment in track.get_visible(player_z, view_dist) {
            self.render_track_segment(segment);
        }

        for obstacle in obstacle_manager.get_visible(player_z, view_dist) {
            self.render_obstacle(obstacle);
        }

        for coin in coin_manager.get_visible(player_z, view_dist) {
            if !coin.collected {
                self.render_coin(coin);
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

    fn render_player(&self, player: &Player) {
        let pos = player.position.to_vec3();
        let color = self.model_manager.get_color("character");

        if let Some(mesh) = self.model_manager.mesh("character") {
            let mut scale = vec3(2.0, 2.0, 2.0);
            let mut rot = quat_face_run_dir();
            let mut pivot = pos + vec3(0.0, PLAYER_MESH_PIVOT_Y, 0.0);

            match player.state {
                PlayerState::Running => {
                    let bob = (player.distance_traveled * 1.5).sin().abs() * 0.15;
                    let wobble = (player.distance_traveled * 0.75).sin() * 0.08;
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

            draw_mesh_at_transform(mesh, pivot, scale, rot);
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

    fn render_obstacle(&self, obstacle: &Obstacle) {
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
                    );
                } else {
                    let color = self.model_manager.get_color("obstacle_full");
                    draw_cube(pos + vec3(0.0, 1.2, 0.0), vec3(1.0, 2.4, 0.6), None, color);
                    draw_cube(pos + vec3(0.0, 1.2, 0.0), vec3(1.1, 2.5, 0.1), None, border);
                }
            }
        }
    }

    fn render_coin(&self, coin: &Coin) {
        let pos = coin.position.to_vec3();
        let color = self.model_manager.get_color("coin");

        let time = get_time() as f32;
        let bob = (time * 3.0 + pos.z * 0.5).sin() * 0.15;
        let pulse = 0.9 + (time * 4.0).sin() * 0.1;

        let coin_pos = pos + vec3(0.0, ROAD_SURFACE_Y + 0.35 + bob, 0.0);
        let spin = Quat::from_rotation_y(time * 3.0);

        if let Some(mesh) = self.model_manager.mesh("coin") {
            draw_mesh_at_rot(mesh, coin_pos, 2.8 * pulse, spin);
        } else {
            draw_sphere(coin_pos, 0.3 * pulse, None, color);
            draw_sphere(coin_pos, 0.2, None, Color::from_rgba(255, 230, 100, 255));
        }
    }

    fn render_track_segment(&self, segment: &TrackSegment) {
        let z = segment.z_position;
        let color = self.model_manager.get_color("track_segment");
        let rot = Quat::from_rotation_y(-FRAC_PI_2);

        if let Some(mesh) = self.model_manager.mesh("road_straight") {
            for ix in -1..=1 {
                for iz in 0..5 {
                    let ox = ix as f32 * ROAD_TILE_STEP;
                    let oz = z - 4.0 + iz as f32 * ROAD_TILE_STEP;
                    draw_mesh_at_rot(mesh, vec3(ox, ROAD_SURFACE_Y, oz), ROAD_TILE_SCALE, rot);
                }
            }
        } else {
            draw_cube(vec3(0.0, -0.25, z), vec3(7.0, 0.5, 10.0), None, color);
        }
    }

    fn render_hud(&self, game_state: &GameState) {
        draw_rectangle(10.0, 10.0, 180.0, 90.0, Color::from_rgba(0, 0, 0, 150));
        self.draw_font_text(
            &format!("Score: {}", game_state.score),
            20.0,
            38.0,
            24,
            WHITE,
        );
        self.draw_font_text(
            &format!("Coins: {}", game_state.coins),
            20.0,
            65.0,
            22,
            Color::from_rgba(255, 200, 50, 255),
        );
        self.draw_font_text(
            &format!("{}m", game_state.distance as i32),
            20.0,
            88.0,
            18,
            Color::from_rgba(180, 180, 200, 255),
        );
    }

    // ── MAIN MENU ───────────────────────────────────────────────────────
    fn render_main_menu(
        &self,
        menu_nav: &MenuNavigator<MenuOption>,
        sub_screen: &MenuSubScreen,
        audio: &AudioSettings,
        character_choice: &CharacterChoice,
        select_char_focused: bool,
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
        let btn_w = 200.0;
        let btn_h = 50.0;
        let left_col_x = (sw - btn_w) / 2.0;
        let menu_start_y = sh * 0.50;
        let btn_spacing = 70.0;

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

        // 3D character rotating (centered in upper area)
        self.render_character_preview(
            character_choice,
            panel_x,
            panel_y,
            panel_w,
            char_preview_h,
            0.6, // model_y
            2.0, // model_scale
        );

        // Character name (below the 3D model)
        let name = character_choice.display_name();
        self.draw_font_text_centered(
            name,
            panel_x + panel_w / 2.0,
            panel_y + char_preview_h + 20.0,
            26,
            WHITE,
        );

        // "SELECT CHARACTER" button (at bottom of panel)
        let select_btn_w = 225.0;
        let select_btn_h = 42.0;
        let select_btn_x = panel_x + (panel_w - select_btn_w) / 2.0;
        let select_btn_y = panel_y + panel_h - select_btn_h - 15.0;
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
            MenuSubScreen::Options => self.render_options_overlay(audio),
            MenuSubScreen::CharacterSelect => {
                self.render_character_select_overlay(character_choice)
            }
            MenuSubScreen::None => {}
        }
    }

    /// Render a rotating 3D character preview inside a screen-space rectangle.
    fn render_character_preview(
        &self,
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
            let vp_x = panel_x as i32;
            let vp_y = (screen_height() - panel_y - panel_h) as i32;
            let vp_w = panel_w as i32;
            let vp_h = panel_h as i32;

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

            let t = get_time() as f32;
            let spin = Quat::from_rotation_y(t * 0.5);
            draw_mesh_at_rot(
                mesh,
                vec3(0.0, model_y, 0.0),
                model_scale,
                spin * quat_face_run_dir(),
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
            box_y + 50.0,
            36,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Instructions
        let lines = [
            "LEFT / RIGHT  -  Change Lane",
            "UP  -  Jump (dodge low barriers)",
            "DOWN  -  Slide (dodge high barriers)",
            "ENTER  -  Select / Confirm",
            "ESC  -  Pause",
            "",
            "Collect coins for extra score!",
            "Full barriers must be dodged",
            "by switching lanes.",
            "",
            "You can cancel a slide into a",
            "jump, and a jump into a slide!",
        ];

        let start_y = box_y + 100.0;
        for (i, line) in lines.iter().enumerate() {
            self.draw_font_text_centered(line, sw / 2.0, start_y + i as f32 * 30.0, 20, WHITE);
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
    fn render_options_overlay(&self, audio: &AudioSettings) {
        let sw = screen_width();
        let sh = screen_height();

        // Dimmed backdrop
        draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));

        // Box
        let box_w = sw * 0.50;
        let box_h = sh * 0.45;
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
            "OPTIONS",
            sw / 2.0,
            box_y + 50.0,
            36,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Volume rows
        let row_labels = ["Master Volume", "Music Volume"];
        let row_values = [audio.master_volume, audio.music_volume];
        let row_y_start = box_y + 120.0;

        for (i, (label, val)) in row_labels.iter().zip(row_values.iter()).enumerate() {
            let y = row_y_start + i as f32 * 80.0;
            let is_focused = audio.focused_row == i;

            let label_color = if is_focused {
                Color::from_rgba(58, 227, 58, 255)
            } else {
                WHITE
            };

            // Label
            self.draw_font_text_centered(label, sw / 2.0, y, 24, label_color);

            // Volume bar
            let bar_w = box_w * 0.6;
            let bar_h = 20.0;
            let bar_x = (sw - bar_w) / 2.0;
            let bar_y = y + 15.0;

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
                2.0,
                Color::from_rgba(200, 200, 220, 180),
            );

            // Value text
            let val_text = format!("{}", val);
            self.draw_font_text_centered(
                &val_text,
                sw / 2.0,
                bar_y + bar_h + 22.0,
                20,
                label_color,
            );

            // Arrow hints when focused
            if is_focused {
                self.draw_font_text(
                    "<",
                    bar_x - 30.0,
                    bar_y + 16.0,
                    22,
                    Color::from_rgba(58, 227, 58, 255),
                );
                self.draw_font_text(
                    ">",
                    bar_x + bar_w + 12.0,
                    bar_y + 16.0,
                    22,
                    Color::from_rgba(58, 227, 58, 255),
                );
            }
        }

        // Close hint
        self.draw_font_text_centered(
            "Press BACK to close",
            sw / 2.0,
            box_y + box_h - 30.0,
            18,
            Color::from_rgba(180, 180, 200, 255),
        );
    }

    // ── Character Select overlay ────────────────────────────────────────
    fn render_character_select_overlay(&self, choice: &CharacterChoice) {
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

        // Title at the top
        let title_y = box_y + 50.0;
        self.draw_font_text_centered(
            "SELECT CHARACTER",
            sw / 2.0,
            title_y,
            36,
            Color::from_rgba(255, 200, 50, 255),
        );

        // Close hint at the bottom
        let close_y = box_y + box_h - 30.0;
        self.draw_font_text_centered(
            "Press ENTER to confirm | BACK to cancel",
            sw / 2.0,
            close_y,
            18,
            Color::from_rgba(180, 180, 200, 255),
        );

        // Available vertical space for character content:
        // from title_y + some padding to close_y - some padding
        let content_top = title_y + 20.0;
        let content_bottom = close_y - 25.0;
        let content_h = content_bottom - content_top;

        // Layout from bottom up: index (20px) + name (35px) + gap (10px) = 65px
        // Remaining space is for the 3D preview
        let index_h = 20.0;
        let name_h = 35.0;
        let gap = 10.0;
        let footer_h = index_h + name_h + gap;
        let preview_h = content_h - footer_h;

        // 3D character preview (centered, using full box width for the viewport)
        let preview_y = content_top;
        self.render_character_preview(
            choice, box_x, preview_y, box_w, preview_h,
            0.7, // model_y: overlay already looks perfect at 1.0
            2.0, // model_scale: large for the overlay
        );

        // Arrow hints — at the vertical center of the preview, at box edges
        let arrow_center_y = preview_y + preview_h / 2.0 + 10.0;
        self.draw_font_text(
            "<",
            box_x + 25.0,
            arrow_center_y,
            40,
            Color::from_rgba(58, 227, 58, 255),
        );
        let arrow_r = ">";
        let r_dim = measure_text(arrow_r, self.font.as_ref(), 40, 1.0);
        self.draw_font_text(
            arrow_r,
            box_x + box_w - 25.0 - r_dim.width,
            arrow_center_y,
            40,
            Color::from_rgba(58, 227, 58, 255),
        );

        // Character name
        let name_y = content_top + preview_h + gap;
        self.draw_font_text_centered(choice.display_name(), sw / 2.0, name_y, 30, WHITE);

        // Index indicator (e.g., "1 / 5")
        let idx_text = format!("{} / {}", choice.index() + 1, CharacterChoice::ALL.len());
        self.draw_font_text_centered(
            &idx_text,
            sw / 2.0,
            name_y + name_h,
            20,
            Color::from_rgba(180, 180, 200, 255),
        );
    }

    // ── Pause Menu ──────────────────────────────────────────────────────
    fn render_pause_menu(&self, pause_nav: &MenuNavigator<PauseOption>, game_state: &GameState) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(0, 0, 30, 180),
        );
        self.draw_font_text_centered("PAUSED", cx, cy - 80.0, 40, WHITE);
        self.draw_font_text_centered(
            &format!("Score: {}", game_state.score),
            cx,
            cy - 30.0,
            22,
            Color::from_rgba(200, 200, 220, 255),
        );

        for (i, option) in pause_nav.options.iter().enumerate() {
            let text = match option {
                PauseOption::Resume => "Resume",
                PauseOption::Restart => "Restart",
                PauseOption::Quit => "Quit",
            };
            let color = if i == pause_nav.selected {
                YELLOW
            } else {
                WHITE
            };
            self.draw_font_text_centered(text, cx, cy + 30.0 + (i as f32 * 38.0), 24, color);
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
        self.draw_font_text_centered("GAME OVER", cx, cy - 80.0, 40, RED);
        self.draw_font_text_centered(
            &format!("Score: {}", game_state.score),
            cx,
            cy - 30.0,
            26,
            WHITE,
        );
        self.draw_font_text_centered(
            &format!("High Score: {}", game_state.high_score),
            cx,
            cy + 5.0,
            20,
            YELLOW,
        );

        for (i, option) in gameover_nav.options.iter().enumerate() {
            let text = match option {
                GameOverOption::Restart => "Play Again",
                GameOverOption::Quit => "Main Menu",
            };
            let color = if i == gameover_nav.selected {
                YELLOW
            } else {
                WHITE
            };
            self.draw_font_text_centered(text, cx, cy + 60.0 + (i as f32 * 38.0), 24, color);
        }
    }

    // ── Camera snap ─────────────────────────────────────────────────────
    pub fn snap_camera(&mut self, player_x: f32, player_y: f32, player_z: f32) {
        self.camera.snap(player_x, player_y, player_z);
    }

    // ── Shared UI helpers ───────────────────────────────────────────────

    /// Draw text using the custom font (or fallback to default).
    fn draw_font_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
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
        let dark_gray = Color::from_rgba(51, 51, 51, 255);

        let (btn_color, highlight) = if is_focused {
            (
                Color::from_rgba(58, 227, 58, 255),
                Color::from_rgba(100, 240, 100, 255),
            )
        } else {
            (
                Color::from_rgba(70, 130, 180, 255),
                Color::from_rgba(100, 160, 210, 255),
            )
        };

        // Shadow + border + face + highlight
        draw_rectangle(x + 6.0, y + 6.0, w, h, dark_gray);
        draw_rectangle(x - 4.0, y - 4.0, w + 8.0, h + 8.0, dark_gray);
        draw_rectangle(x, y, w, h, btn_color);
        draw_rectangle(x, y, w, h * 0.25, highlight);

        // Text
        let font_size = 24.0;
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
