//! Game State Management for Toon Dash

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
    QuitConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShutdownStage {
    #[default]
    None,
    Requested,
    Finalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShutdownFlow {
    pub stage: ShutdownStage,
}

impl ShutdownFlow {
    pub fn request_close(&mut self) {
        if self.stage == ShutdownStage::None {
            self.stage = ShutdownStage::Requested;
        }
    }

    pub fn mark_finalizing(&mut self) {
        if self.stage == ShutdownStage::Requested {
            self.stage = ShutdownStage::Finalizing;
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != ShutdownStage::None
    }
}

// ── Game Settings (volume & speed controllers) ─────────────────────────
#[derive(Debug, Clone)]
pub struct GameSettings {
    pub master_volume: u8,
    pub music_volume: u8,
    pub effects_volume: u8,
    pub game_speed: u8,
    /// Which row is focused in the options panel (0 = master, 1 = music, 2 = effects, 3 = speed)
    pub focused_row: usize,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 5, // user initially set default master to 50%
            music_volume: 10,
            effects_volume: 10,
            game_speed: 5,  // 5 / 5.0 = 1.0x (Baseline)
            focused_row: 0,
        }
    }
}

impl GameSettings {
    pub fn master_f32(&self) -> f32 { self.master_volume as f32 / 10.0 }
    pub fn music_f32(&self)  -> f32 { self.music_volume  as f32 / 10.0 }
    pub fn effects_f32(&self) -> f32 { self.effects_volume as f32 / 10.0 }
    pub fn speed_f32(&self) -> f32 { self.game_speed as f32 / 5.0 }
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
    pub score: f32,
    pub high_score: f32,
    pub distance: f32,
    pub coins: u32,
    pub combo: u32,
    pub combo_anim_timer: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            screen: GameScreen::MainMenu,
            score: 0.0,
            high_score: 0.0,
            distance: 0.0,
            coins: 0,
            combo: 1,
            combo_anim_timer: 0.0,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.high_score = super::persistence::load_highscore() as f32;
        state
    }

    pub fn start_game(&mut self) {
        self.screen = GameScreen::Playing;
        self.score = 0.0;
        self.distance = 0.0;
        self.coins = 0;
        self.combo = 1;
        self.combo_anim_timer = 0.0;
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
            super::persistence::save_highscore(self.high_score as u32);
        }
    }

    pub fn return_to_menu(&mut self) {
        self.screen = GameScreen::MainMenu;
    }

    pub fn update_score(&mut self, dt: f32, distance: f32, coins: u32) {
        let prev_dist = self.distance;
        self.distance = distance;
        self.coins = coins;

        // Points from distance traveled, multiplied by combo
        let dist_diff = self.distance - prev_dist;
        if dist_diff > 0.0 {
            self.score += dist_diff * self.combo as f32;
        }

        // Update animation timer
        if self.combo_anim_timer > 0.0 {
            self.combo_anim_timer -= dt;
        }
    }

    pub fn add_collectible_points(&mut self, is_jewel: bool) {
        if is_jewel {
            self.score += 200.0;
            self.combo += 1;
            self.combo_anim_timer = 0.8; // Trigger animation
        } else {
            self.score += 100.0; // Coin is always 100 flat
        }
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
    Options,
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
        Self::new(vec![
            PauseOption::Resume,
            PauseOption::Restart,
            PauseOption::Options,
            PauseOption::Quit,
        ])
    }
}

impl MenuNavigator<GameOverOption> {
    pub fn game_over_menu() -> Self {
        Self::new(vec![GameOverOption::Restart, GameOverOption::Quit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_flow_transitions_in_order() {
        let mut flow = ShutdownFlow::default();
        assert_eq!(flow.stage, ShutdownStage::None);

        flow.request_close();
        assert_eq!(flow.stage, ShutdownStage::Requested);

        flow.mark_finalizing();
        assert_eq!(flow.stage, ShutdownStage::Finalizing);
    }

    #[test]
    fn request_close_is_idempotent() {
        let mut flow = ShutdownFlow::default();
        flow.request_close();
        flow.request_close();
        assert_eq!(flow.stage, ShutdownStage::Requested);
    }
}