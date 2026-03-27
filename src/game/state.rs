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

// ── Sub-screens shown as overlays on the main menu ──────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuSubScreen {
    #[default]
    None,
    HowToPlay,
    Options,
    CharacterSelect,
}

// ── Audio settings (master + music volume 0-10) ─────────────────────────
#[derive(Debug, Clone)]
pub struct AudioSettings {
    pub master_volume: u8,
    pub music_volume: u8,
    /// Which row is focused in the options panel (0 = master, 1 = music)
    pub focused_row: usize,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 10,
            music_volume: 10,
            focused_row: 0,
        }
    }
}

impl AudioSettings {
    pub fn master_f32(&self) -> f32 { self.master_volume as f32 / 10.0 }
    pub fn music_f32(&self)  -> f32 { self.music_volume  as f32 / 10.0 }
}

// ── Selectable characters ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterChoice {
    Oodi,
    Ooli,
    Oozi,
    Oopi,
    Oobi,
}

impl Default for CharacterChoice {
    fn default() -> Self { Self::Oodi }
}

impl CharacterChoice {
    pub const ALL: [CharacterChoice; 5] = [
        CharacterChoice::Oodi,
        CharacterChoice::Ooli,
        CharacterChoice::Oozi,
        CharacterChoice::Oopi,
        CharacterChoice::Oobi,
    ];

    pub fn mesh_key(&self) -> &'static str {
        match self {
            CharacterChoice::Oodi => "char_oodi",
            CharacterChoice::Ooli => "char_ooli",
            CharacterChoice::Oozi => "char_oozi",
            CharacterChoice::Oopi => "char_oopi",
            CharacterChoice::Oobi => "char_oobi",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CharacterChoice::Oodi => "OODI",
            CharacterChoice::Ooli => "OOLI",
            CharacterChoice::Oozi => "OOZI",
            CharacterChoice::Oopi => "OOPI",
            CharacterChoice::Oobi => "OOBI",
        }
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|c| c == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % Self::ALL.len()]
    }

    pub fn next(&self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(&self) -> Self {
        let idx = self.index();
        if idx == 0 { Self::from_index(Self::ALL.len() - 1) } else { Self::from_index(idx - 1) }
    }
}

// ── Game State ──────────────────────────────────────────────────────────
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

// ── Menu options ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuOption {
    #[default]
    Play,
    HowToPlay,
    Options,
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

// ── Generic navigator ──────────────────────────────────────────────────
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
        Self::new(vec![
            MenuOption::Play,
            MenuOption::HowToPlay,
            MenuOption::Options,
            MenuOption::Quit,
        ])
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