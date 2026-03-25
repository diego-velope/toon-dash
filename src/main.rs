// src/main.rs
//! Main entry point for Toon Dash

use macroquad::prelude::*;
use toon_dash::game::*;
use toon_dash::input::*;
use toon_dash::rendering::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Toon Dash".to_string(),
        fullscreen: false,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("assets");

    info!("Starting Toon Dash...");

    // Initialize game systems
    let config = GameConfig::default();
    let mut input = TvInput::new();
    let mut game_state = GameState::new();
    let mut player = Player::new();
    let mut track = Track::new();
    let mut obstacle_manager = ObstacleManager::new();
    let mut coin_manager = CoinManager::new();
    let mut renderer = GameRenderer::new();

    // Menu navigators
    let mut menu_nav = MenuNavigator::main_menu();
    let mut pause_nav = MenuNavigator::pause_menu();
    let mut gameover_nav = MenuNavigator::game_over_menu();

    // Load models
    info!("Loading models...");
    renderer.load_models().await;
    info!("Models loaded!");

    let mut last_time = get_time();

    loop {
        let now = get_time();
        let dt = ((now - last_time) as f32).min(0.1);
        last_time = now;

        // Process input
        input.update();

        // Handle game logic based on current screen
        match game_state.screen {
            GameScreen::MainMenu => {
                if input.is_up_just_pressed() {
                    menu_nav.up();
                }
                if input.is_down_just_pressed() {
                    menu_nav.down();
                }
                if input.is_action_just_pressed() {
                    match menu_nav.current() {
                        MenuOption::Play => {
                            game_state.start_game();
                            player.reset();
                            track.reset();
                            obstacle_manager.reset();
                            coin_manager.reset();
                            menu_nav = MenuNavigator::main_menu();
                         
                            // ── CRITICAL: snap camera to player before first frame ──────────
                            // Without this, the 0.15 lerp takes ~15 seconds to move the camera
                            // from its default position (0,5,-8) to behind the player.
                            renderer.snap_camera(
                                player.position.x,
                                player.position.y,
                                player.position.z,
                            );
                        }
                         
                        MenuOption::Quit => {
                            std::process::exit(0);
                        }
                    }
                }
            }
            GameScreen::Playing => {
                // Update player
                if input.is_left_just_pressed() {
                    player.change_lane(-1);
                }
                if input.is_right_just_pressed() {
                    player.change_lane(1);
                }
                if input.is_up_just_pressed() {
                    player.jump(&config);
                }
                if input.is_down_just_pressed() {
                    player.slide(&config);
                }
                if input.is_back_just_pressed() {
                    game_state.pause();
                }

                player.update(dt, &config);

                // Update track
                track.update(player.distance_traveled, &config);

                // Spawn obstacles and coins
                let obstacle_zones = track.get_obstacle_zones(player.distance_traveled, config.spawn_distance);
                obstacle_manager.spawn_from_segments(&obstacle_zones, &config);
                obstacle_manager.update(player.distance_traveled, &config);

                let coin_zones = track.get_coin_zones(player.distance_traveled, config.spawn_distance);
                coin_manager.spawn_from_segments(&coin_zones, &config);
                coin_manager.update(player.distance_traveled, &config);

                // Check collisions
                let player_bbox = player.get_bounding_box();
                
                if let Some(_obstacle) = obstacle_manager.check_collision(
                    &player_bbox,
                    player.lane,
                    player.is_airborne(),
                    player.is_sliding(),
                ) {
                    player.die();
                    game_state.game_over();
                    gameover_nav = MenuNavigator::game_over_menu();
                }

                // Check coin collection
                let coins = coin_manager.check_collection(player.lane, player.position.y, player.distance_traveled);
                
                // Update score
                game_state.update_score(player.distance_traveled, game_state.coins + coins, &config);
            }
            GameScreen::Paused => {
                if input.is_up_just_pressed() {
                    pause_nav.up();
                }
                if input.is_down_just_pressed() {
                    pause_nav.down();
                }
                if input.is_action_just_pressed() {
                    match pause_nav.current() {
                        PauseOption::Resume => {
                            game_state.resume();
                        }
                        PauseOption::Restart => {
                            game_state.start_game();
                            player.reset();
                            track.reset();
                            obstacle_manager.reset();
                            coin_manager.reset();
                            pause_nav = MenuNavigator::pause_menu();
                        }
                        PauseOption::Quit => {
                            game_state.return_to_menu();
                            menu_nav = MenuNavigator::main_menu();
                        }
                    }
                }
            }
            GameScreen::GameOver => {
                if input.is_up_just_pressed() {
                    gameover_nav.up();
                }
                if input.is_down_just_pressed() {
                    gameover_nav.down();
                }
                if input.is_action_just_pressed() {
                    match gameover_nav.current() {
                        GameOverOption::Restart => {
                            game_state.start_game();
                            player.reset();
                            track.reset();
                            obstacle_manager.reset();
                            coin_manager.reset();
                            gameover_nav = MenuNavigator::game_over_menu();
                        }
                        GameOverOption::Quit => {
                            game_state.return_to_menu();
                            menu_nav = MenuNavigator::main_menu();
                        }
                    }
                }
            }
        }

        // Update camera
        renderer.update(&player, dt);

        // Render
        renderer.render(
            &game_state,
            &track,
            &player,
            &obstacle_manager,
            &coin_manager,
            &menu_nav,
            &pause_nav,
            &gameover_nav,
        );

        next_frame().await;
    }
}