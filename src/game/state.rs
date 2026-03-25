//! Game State Management for Toon Dash

use super::types::GameConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameScreen {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

impl Default for GameScreen {
    fn default() -> Self { Self::MainMenu }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub screen: GameScreen,
    pub score: u32,
    pub high_score: u32,
    pub distance: f32,
    pub coins: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            screen: GameScreen::MainMenu,
            score: 0,
            high_score: 0,
            distance: 0.0,
            coins: 0,
        }
    }
}

impl GameState {
    pub fn new() -> Self { Self::default() }

    pub fn start_game(&mut self) {
        self.screen = GameScreen::Playing;
        self.score = 0;
        self.distance = 0.0;
        self.coins = 0;
    }

    pub fn pause(&mut self) {
        if self.screen == GameScreen::Playing {
            self.screen = GameScreen::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.screen == GameScreen::Paused {
            self.screen = GameScreen::Playing;
        }
    }

    pub fn game_over(&mut self) {
        self.screen = GameScreen::GameOver;
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }

    pub fn return_to_menu(&mut self) {
        self.screen = GameScreen::MainMenu;
    }

    pub fn update_score(&mut self, distance: f32, coins: u32, config: &GameConfig) {
        self.distance = distance;
        self.coins = coins;
        self.score = (distance as u32) + (coins * config.coin_value);
    }

    pub fn is_playing(&self) -> bool { self.screen == GameScreen::Playing }
    pub fn is_paused(&self) -> bool { self.screen == GameScreen::Paused }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuOption {
    #[default]
    Play,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PauseOption {
    #[default]
    Resume,
    Restart,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameOverOption {
    #[default]
    Restart,
    Quit,
}

pub struct MenuNavigator<T> {
    pub options: Vec<T>,
    pub selected: usize,
}

impl<T> MenuNavigator<T> {
    pub fn new(options: Vec<T>) -> Self {
        Self { options, selected: 0 }
    }

    pub fn up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }

    pub fn down(&mut self) {
        if self.selected < self.options.len() - 1 { self.selected += 1; }
    }

    pub fn current(&self) -> &T { &self.options[self.selected] }
}

impl MenuNavigator<MenuOption> {
    pub fn main_menu() -> Self {
        Self::new(vec![MenuOption::Play, MenuOption::Quit])
    }
}

impl MenuNavigator<PauseOption> {
    pub fn pause_menu() -> Self {
        Self::new(vec![PauseOption::Resume, PauseOption::Restart, PauseOption::Quit])
    }
}

impl MenuNavigator<GameOverOption> {
    pub fn game_over_menu() -> Self {
        Self::new(vec![GameOverOption::Restart, GameOverOption::Quit])
    }
}