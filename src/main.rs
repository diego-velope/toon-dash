// src/main.rs
//! Main entry point for Toon Dash

use macroquad::audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams};
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

    // Menu state
    let mut menu_nav = MenuNavigator::main_menu();
    let mut pause_nav = MenuNavigator::pause_menu();
    let mut gameover_nav = MenuNavigator::game_over_menu();
    let mut sub_screen = MenuSubScreen::None;
    let mut audio = AudioSettings::default();
    let mut character_choice = CharacterChoice::default();
    let mut select_char_focused = false;

    // Load models and audio
    info!("Loading assets...");
    renderer.load_models().await;
    let bgm = load_sound("audio/toon_dash_background_music.wav")
        .await
        .expect("Failed to load background music");
    info!("Assets loaded!");

    // Start background music
    play_sound(
        &bgm,
        PlaySoundParams {
            looped: true,
            // Volume will be updated immediately in the match loop
            volume: 1.0,
        },
    );

    let mut last_time = get_time();

    loop {
        let now = get_time();
        let dt = ((now - last_time) as f32).min(0.1);
        last_time = now;

        // Process input
        input.update();

        // Update audio volume dynamically based on logic: min(master, music)
        let effective_vol = audio.master_volume.min(audio.music_volume) as f32 / 10.0;
        set_sound_volume(&bgm, effective_vol);

        // Handle game logic based on current screen
        match game_state.screen {
            GameScreen::MainMenu => {
                match sub_screen {
                    // ── No overlay: navigate main menu buttons ───────────
                    MenuSubScreen::None => {
                        if select_char_focused {
                            // SELECT CHARACTER button is focused
                            if input.is_action_just_pressed() {
                                sub_screen = MenuSubScreen::CharacterSelect;
                                select_char_focused = false;
                            }
                            if input.is_left_just_pressed() || input.is_back_just_pressed() {
                                select_char_focused = false;
                            }
                        } else {
                            // Normal menu navigation
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
                                        renderer.set_active_character(&character_choice);
                                        renderer.snap_camera(
                                            player.position.x,
                                            player.position.y,
                                            player.position.z,
                                        );
                                    }
                                    MenuOption::HowToPlay => {
                                        sub_screen = MenuSubScreen::HowToPlay;
                                    }
                                    MenuOption::Options => {
                                        sub_screen = MenuSubScreen::Options;
                                    }
                                    MenuOption::Quit => {
                                        std::process::exit(0);
                                    }
                                }
                            }
                            // Right arrow focuses SELECT CHARACTER button
                            if input.is_right_just_pressed() {
                                select_char_focused = true;
                            }
                        }
                    }

                    // ── How to Play overlay ──────────────────────────────
                    MenuSubScreen::HowToPlay => {
                        if input.is_action_just_pressed() || input.is_back_just_pressed() {
                            sub_screen = MenuSubScreen::None;
                        }
                    }

                    // ── Options overlay (volume control) ─────────────────
                    MenuSubScreen::Options => {
                        if input.is_back_just_pressed() {
                            sub_screen = MenuSubScreen::None;
                        }
                        if input.is_up_just_pressed() {
                            if audio.focused_row > 0 {
                                audio.focused_row -= 1;
                            }
                        }
                        if input.is_down_just_pressed() {
                            if audio.focused_row < 1 {
                                audio.focused_row += 1;
                            }
                        }
                        if input.is_left_just_pressed() {
                            match audio.focused_row {
                                0 => {
                                    if audio.master_volume > 0 {
                                        audio.master_volume -= 1;
                                    }
                                }
                                1 => {
                                    if audio.music_volume > 0 {
                                        audio.music_volume -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if input.is_right_just_pressed() {
                            match audio.focused_row {
                                0 => {
                                    if audio.master_volume < 10 {
                                        audio.master_volume += 1;
                                    }
                                }
                                1 => {
                                    if audio.music_volume < 10 {
                                        audio.music_volume += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // ── Character Select overlay ─────────────────────────
                    MenuSubScreen::CharacterSelect => {
                        if input.is_back_just_pressed() || input.is_action_just_pressed() {
                            sub_screen = MenuSubScreen::None;
                        }
                        if input.is_left_just_pressed() {
                            character_choice = character_choice.prev();
                        }
                        if input.is_right_just_pressed() {
                            character_choice = character_choice.next();
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
                let obstacle_zones =
                    track.get_obstacle_zones(player.distance_traveled, config.spawn_distance);
                obstacle_manager.spawn_from_segments(&obstacle_zones, &config);
                obstacle_manager.update(player.distance_traveled, &config);

                let coin_zones =
                    track.get_coin_zones(player.distance_traveled, config.spawn_distance);
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
                let coins = coin_manager.check_collection(
                    player.lane,
                    player.position.y,
                    player.distance_traveled,
                );
                game_state.update_score(
                    player.distance_traveled,
                    game_state.coins + coins,
                    &config,
                );
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
            &sub_screen,
            &audio,
            &character_choice,
            select_char_focused,
        );

        next_frame().await;
    }
}
