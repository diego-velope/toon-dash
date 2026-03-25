// src/rendering/renderer.rs
//! Game renderer with Kenney-style graphics

use macroquad::prelude::*;
use crate::game::*;
use crate::models::{draw_mesh_at, draw_mesh_at_rot, ModelManager};
use glam::Quat;
use std::f32::consts::FRAC_PI_2;
use super::camera::GameCamera;

/// City Kit `road-straight` tile: scale/step tuned so ~5 tiles cover segment length (~10) and ix -4..4 spans lane width.
const ROAD_TILE_SCALE: f32 = 2.35;
const ROAD_TILE_STEP: f32 = 2.35;
const ROAD_SURFACE_Y: f32 = 0.03;
/// Character mesh pivot so feet read on the road surface at `ROAD_SURFACE_Y` with `ROAD_TILE_SCALE`.
const PLAYER_MESH_PIVOT_Y: f32 = 0.58;
const PLAYER_SLIDE_CENTER_Y: f32 = 0.32;

/// Kenney humanoids export facing +X; gameplay runs along +Z.
fn quat_face_run_dir() -> Quat {
    Quat::from_rotation_y(-FRAC_PI_2)
}

pub struct GameRenderer {
    camera: GameCamera,
    model_manager: ModelManager,
}

impl GameRenderer {
    pub fn new() -> Self {
        Self {
            camera: GameCamera::new(),
            model_manager: ModelManager::new(),
        }
    }

    pub async fn load_models(&mut self) {
        self.model_manager.load_models().await;
    }

    pub fn update(&mut self, player: &Player, dt: f32) {
        self.camera.update(
            player.position.x,
            player.position.y,
            player.position.z,
            dt,
        );
    }

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
    ) {
        clear_background(Color::from_rgba(135, 206, 250, 255)); // Sky blue

        match game_state.screen {
            GameScreen::MainMenu => {
                self.render_main_menu(menu_nav);
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
        // Same half-width as road ix -4..=4 at ROAD_TILE_STEP (leave gap so curbs read clearly).
        let half_road = 4.5 * ROAD_TILE_STEP;
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
            if player.is_sliding() {
                let (height, y_off) = (0.5, PLAYER_SLIDE_CENTER_Y);
                draw_cube(
                    pos + vec3(0.0, y_off, 0.0),
                    vec3(0.8, height, 0.6),
                    None,
                    color,
                );
            } else {
                let pivot = pos + vec3(0.0, PLAYER_MESH_PIVOT_Y, 0.0);
                draw_mesh_at_rot(mesh, pivot, 0.42, quat_face_run_dir());
            }
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
                draw_cube(
                    pos + vec3(0.0, 1.6, 0.0),
                    vec3(0.2, 0.15, 0.15),
                    None,
                    Color::from_rgba(40, 40, 60, 255),
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
                        0.52,
                        Quat::from_rotation_y(-FRAC_PI_2),
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
                        0.48,
                        Quat::from_rotation_y(-FRAC_PI_2),
                    );
                } else {
                    let color = self.model_manager.get_color("obstacle_high");
                    draw_cube(pos + vec3(0.0, 1.6, 0.0), vec3(1.4, 0.4, 0.8), None, color);
                    let pillar = Color::from_rgba(180, 120, 50, 255);
                    draw_cube(pos + vec3(-0.5, 0.9, 0.0), vec3(0.15, 1.8, 0.15), None, pillar);
                    draw_cube(pos + vec3(0.5, 0.9, 0.0), vec3(0.15, 1.8, 0.15), None, pillar);
                }
            }
            ObstacleType::FullBarrier => {
                if let Some(mesh) = self.model_manager.mesh("obstacle_full") {
                    draw_mesh_at_rot(
                        mesh,
                        pos + vec3(0.0, ROAD_SURFACE_Y, 0.0),
                        0.48,
                        Quat::from_rotation_y(-FRAC_PI_2),
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

        if let Some(mesh) = self.model_manager.mesh("coin") {
            draw_mesh_at(mesh, coin_pos, 2.8 * pulse);
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
            for ix in -4..=4 {
                for iz in 0..5 {
                    let ox = ix as f32 * ROAD_TILE_STEP;
                    let oz = z - 4.0 + iz as f32 * ROAD_TILE_STEP;
                    draw_mesh_at_rot(
                        mesh,
                        vec3(ox, ROAD_SURFACE_Y, oz),
                        ROAD_TILE_SCALE,
                        rot,
                    );
                }
            }
        } else {
            draw_cube(
                vec3(0.0, -0.25, z),
                vec3(7.0, 0.5, 10.0),
                None,
                color,
            );
        }
    }

    fn render_hud(&self, game_state: &GameState) {
        // Fondo semitransparente
        draw_rectangle(10.0, 10.0, 180.0, 90.0, Color::from_rgba(0, 0, 0, 150));
        
        draw_text(&format!("Score: {}", game_state.score), 20.0, 38.0, 24.0, WHITE);
        draw_text(&format!("Coins: {}", game_state.coins), 20.0, 65.0, 22.0, Color::from_rgba(255, 200, 50, 255));
        draw_text(&format!("{}m", game_state.distance as i32), 20.0, 88.0, 18.0, Color::from_rgba(180, 180, 200, 255));
    }

    // ... (Mantén los métodos de menú exactamente igual que en tu código anterior)
    // render_main_menu, render_pause_menu, render_game_over ...
    // Si no los tienes en este archivo, copia los métodos privados del archivo original.

    fn render_main_menu(&self, menu_nav: &MenuNavigator<MenuOption>) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(18, 32, 72, 255),
        );

        if let Some(mesh) = self.model_manager.mesh("character") {
            let aspect = screen_width() / screen_height();
            set_camera(&Camera3D {
                position: vec3(2.4, 1.35, -2.9),
                target: vec3(0.0, 0.75, 0.0),
                up: Vec3::Y,
                fovy: 45.0_f32.to_radians(),
                aspect: Some(aspect),
                projection: Projection::Perspective,
                render_target: None,
                viewport: None,
                z_near: 0.1,
                z_far: 80.0,
            });
            let t = get_time() as f32;
            let spin = Quat::from_rotation_y(t * 0.5);
            draw_mesh_at_rot(
                mesh,
                vec3(0.0, 0.0, 0.0),
                0.7,
                spin * quat_face_run_dir(),
            );
            set_default_camera();
        }

        draw_text("TOON DASH", cx - 120.0, cy - 120.0, 60.0, Color::from_rgba(255, 200, 50, 255));
        draw_text("Endless Runner", cx - 80.0, cy - 60.0, 25.0, WHITE);

        for (i, option) in menu_nav.options.iter().enumerate() {
            let text = match option {
                MenuOption::Play => "Play",
                MenuOption::Quit => "Quit",
            };
            let color = if i == menu_nav.selected { YELLOW } else { WHITE };
            draw_text(text, cx - 40.0, cy + 20.0 + (i as f32 * 45.0), 28.0, color);
        }

        draw_text("D-Pad: Navigate | Enter: Select", cx - 150.0, cy + 160.0, 18.0, Color::from_rgba(150, 150, 180, 255));
    }

    fn render_pause_menu(&self, pause_nav: &MenuNavigator<PauseOption>, game_state: &GameState) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 30, 180));
        draw_text("PAUSED", cx - 60.0, cy - 80.0, 40.0, WHITE);
        draw_text(&format!("Score: {}", game_state.score), cx - 60.0, cy - 30.0, 22.0, Color::from_rgba(200, 200, 220, 255));

        for (i, option) in pause_nav.options.iter().enumerate() {
            let text = match option {
                PauseOption::Resume => "Resume",
                PauseOption::Restart => "Restart",
                PauseOption::Quit => "Quit",
            };
            let color = if i == pause_nav.selected { YELLOW } else { WHITE };
            draw_text(text, cx - 45.0, cy + 30.0 + (i as f32 * 38.0), 24.0, color);
        }
    }

    fn render_game_over(&self, gameover_nav: &MenuNavigator<GameOverOption>, game_state: &GameState) {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(50, 20, 20, 200));
        draw_text("GAME OVER", cx - 80.0, cy - 80.0, 40.0, RED);
        draw_text(&format!("Score: {}", game_state.score), cx - 60.0, cy - 30.0, 26.0, WHITE);
        draw_text(&format!("High Score: {}", game_state.high_score), cx - 70.0, cy + 5.0, 20.0, YELLOW);

        for (i, option) in gameover_nav.options.iter().enumerate() {
            let text = match option {
                GameOverOption::Restart => "Play Again",
                GameOverOption::Quit => "Main Menu",
            };
            let color = if i == gameover_nav.selected { YELLOW } else { WHITE };
            draw_text(text, cx - 50.0, cy + 60.0 + (i as f32 * 38.0), 24.0, color);
        }
    }

    pub fn snap_camera(&mut self, player_x: f32, player_y: f32, player_z: f32) {
        self.camera.snap(player_x, player_y, player_z);
    }
}

impl Default for GameRenderer {
    fn default() -> Self {
        Self::new()
    }
}