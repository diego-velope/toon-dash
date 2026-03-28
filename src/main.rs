// src/main.rs
//! Main entry point for Toon Dash

use macroquad::audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams};
use macroquad::prelude::*;
use toon_dash::game::*;
use toon_dash::input::*;
use toon_dash::rendering::*;

#[cfg(target_arch = "wasm32")]
use getrandom::{register_custom_getrandom, Error};

#[cfg(target_arch = "wasm32")]
fn macroquad_getrandom(buf: &mut [u8]) -> Result<(), Error> {
    for b in buf {
        *b = macroquad::rand::gen_range(0u16, 256u16) as u8;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
register_custom_getrandom!(macroquad_getrandom);

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
    let mut collectible_manager = CollectibleManager::new();
    let mut renderer = GameRenderer::new();

    // Menu state
    let mut menu_nav = MenuNavigator::main_menu();
    let mut pause_nav = MenuNavigator::pause_menu();
    let mut gameover_nav = MenuNavigator::game_over_menu();
    let mut sub_screen = MenuSubScreen::None;
    let mut game_settings = GameSettings::default();
    let mut character_choice = CharacterChoice::default();
    let mut select_char_focused = false;

    // Load models and audio
    info!("Loading assets...");
    toon_dash::game::loading::set_progress(10.0);
    renderer.load_models().await;
    toon_dash::game::loading::set_progress(90.0);
    let bgm = load_sound("audio/chasing_rocket_turtle_instrumental.wav")
        .await
        .expect("Failed to load background music");
    let coin_sfx = load_sound("audio/coin_collect.wav")
        .await
        .expect("Failed to load coin sound");
    toon_dash::game::loading::set_progress(100.0);
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

    toon_dash::game::loading::hide_splash();
    let mut last_time = get_time();

    loop {
        let now = get_time();
        let dt = ((now - last_time) as f32).min(0.1);
        last_time = now;

        // Process input
        input.update();

        // Progressive game speed scaling: base speed + 1.0 per 5000 score
        let speed_multiplier = 1.0 + (game_state.score / 5000.0);
        let total_speed = game_settings.speed_f32() * speed_multiplier;
        let scaled_dt = dt * total_speed;

        // Update audio volume dynamically based on logic: min(master, music)
        let effective_vol =
            game_settings.master_volume.min(game_settings.music_volume) as f32 / 10.0;
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
                                        collectible_manager.reset();
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

                    // ── Options overlay (volume & speed control) ───────────────
                    MenuSubScreen::Options => {
                        if input.is_back_just_pressed() {
                            sub_screen = MenuSubScreen::None;
                        }
                        if input.is_up_just_pressed() {
                            if game_settings.focused_row > 0 {
                                game_settings.focused_row -= 1;
                            }
                        }
                        if input.is_down_just_pressed() {
                            if game_settings.focused_row < 3 {
                                game_settings.focused_row += 1;
                            }
                        }
                        if input.is_left_just_pressed() {
                            match game_settings.focused_row {
                                0 => {
                                    if game_settings.master_volume > 0 {
                                        game_settings.master_volume -= 1;
                                    }
                                }
                                1 => {
                                    if game_settings.music_volume > 0 {
                                        game_settings.music_volume -= 1;
                                    }
                                }
                                2 => {
                                    if game_settings.effects_volume > 0 {
                                        game_settings.effects_volume -= 1;
                                    }
                                }
                                3 => {
                                    if game_settings.game_speed > 1 {
                                        game_settings.game_speed -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if input.is_right_just_pressed() {
                            match game_settings.focused_row {
                                0 => {
                                    if game_settings.master_volume < 10 {
                                        game_settings.master_volume += 1;
                                    }
                                }
                                1 => {
                                    if game_settings.music_volume < 10 {
                                        game_settings.music_volume += 1;
                                    }
                                }
                                2 => {
                                    if game_settings.effects_volume < 10 {
                                        game_settings.effects_volume += 1;
                                    }
                                }
                                3 => {
                                    if game_settings.game_speed < 10 {
                                        game_settings.game_speed += 1;
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

                player.update(scaled_dt, dt, &config);

                // Update track
                track.update(player.distance_traveled, &config);

                // Spawn obstacles and coins
                let obstacle_zones =
                    track.get_obstacle_zones(player.distance_traveled, config.spawn_distance);
                obstacle_manager.spawn_from_segments(&obstacle_zones, &config);
                obstacle_manager.update(player.distance_traveled, &config);

                let coin_zones =
                    track.get_coin_zones(player.distance_traveled, config.spawn_distance);
                collectible_manager.spawn_from_segments(&coin_zones, &config);
                collectible_manager.update(player.distance_traveled, &config);

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

                // Check collectible collection
                let (coins, jewels) = collectible_manager.check_collection(
                    player.lane,
                    player.position.y,
                    player.distance_traveled,
                );

                if coins > 0 || jewels > 0 {
                    let sfx_vol = game_settings
                        .master_volume
                        .min(game_settings.effects_volume) as f32
                        / 10.0;
                    play_sound(
                        &coin_sfx,
                        PlaySoundParams {
                            looped: false,
                            volume: sfx_vol,
                        },
                    );

                    for _ in 0..coins {
                        game_state.add_collectible_points(false);
                    }
                    for _ in 0..jewels {
                        game_state.add_collectible_points(true);
                    }
                }

                game_state.update_score(
                    scaled_dt,
                    player.distance_traveled,
                    game_state.coins + coins,
                );
            }
            GameScreen::Paused => {
                // If the options menu is open, capture input exclusively for it
                if sub_screen == MenuSubScreen::Options {
                    if input.is_back_just_pressed()
                        || input.is_action_just_pressed() && game_settings.focused_row == 4
                    {
                        // We will add a "Back" button at row 4 or just use ECS/Back button
                        sub_screen = MenuSubScreen::None;
                    }
                    if input.is_up_just_pressed() {
                        if game_settings.focused_row > 0 {
                            game_settings.focused_row -= 1;
                        }
                    }
                    if input.is_down_just_pressed() {
                        if game_settings.focused_row < 3 {
                            game_settings.focused_row += 1;
                        }
                    }
                    if input.is_left_just_pressed() {
                        match game_settings.focused_row {
                            0 => {
                                if game_settings.master_volume > 0 {
                                    game_settings.master_volume -= 1;
                                }
                            }
                            1 => {
                                if game_settings.music_volume > 0 {
                                    game_settings.music_volume -= 1;
                                }
                            }
                            2 => {
                                if game_settings.effects_volume > 0 {
                                    game_settings.effects_volume -= 1;
                                }
                            }
                            3 => {
                                if game_settings.game_speed > 1 {
                                    game_settings.game_speed -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    if input.is_right_just_pressed() {
                        match game_settings.focused_row {
                            0 => {
                                if game_settings.master_volume < 10 {
                                    game_settings.master_volume += 1;
                                }
                            }
                            1 => {
                                if game_settings.music_volume < 10 {
                                    game_settings.music_volume += 1;
                                }
                            }
                            2 => {
                                if game_settings.effects_volume < 10 {
                                    game_settings.effects_volume += 1;
                                }
                            }
                            3 => {
                                if game_settings.game_speed < 10 {
                                    game_settings.game_speed += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
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
                                collectible_manager.reset();
                                pause_nav = MenuNavigator::pause_menu();
                            }
                            PauseOption::Options => {
                                sub_screen = MenuSubScreen::Options;
                            }
                            PauseOption::Quit => {
                                game_state.return_to_menu();
                                menu_nav = MenuNavigator::main_menu();
                            }
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
                            collectible_manager.reset();
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
            &collectible_manager,
            &menu_nav,
            &pause_nav,
            &gameover_nav,
            &sub_screen,
            &game_settings,
            &character_choice,
            select_char_focused,
        );

        next_frame().await;
    }
}
