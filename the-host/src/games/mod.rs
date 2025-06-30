use bevy::app::Update;
use bevy::log::info;
use crate::PlayerNum;
use bevy::prelude::{App, AppExtStates, Assets, Component, NextState, Res, ResMut, State, States};
use crate::config::{Config, ConfigAccessor};
use crate::debug_input::DebugPlayerInput;

pub mod racing;
pub mod waiting;

/// Component to identify player by a player number
#[derive(Component)]
pub struct Player(PlayerNum);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum CurrentGame {
    /// Special reusable screen for waiting for anything
    Waiting,
    /// Special screen for deciding what to play next
    #[default]
    Voting,
    /// Players race each other in little cars!
    Racing,
    // this can be extended for new games!
}

/// These are all the states for the program.
/// Do not extend this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum GamePhase {
    /// Special menu for debug controls, etc.
    Menu,
    /// Deciding which game to play next
    #[default]
    Voting,
    /// Player setup, joining/leaving, etc., loading assets, etc.
    PreGame,
    /// Actual gameplay (player joining/leaving is minimal)
    PlayingGame,
    /// Game results, etc.
    PostGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
enum ConfigLoadState {
    #[default]
    Loading,
    Loaded
}

pub fn init_games(app: &mut App) {
    app.add_systems(Update, update_config_load_state);
    app.init_state::<ConfigLoadState>();
    app.init_state::<GamePhase>();
    app.init_state::<CurrentGame>();
    app.add_systems(Update, debug_go_to_racing_game_on_spacebar); // to be removed
    racing::init_app(app);
}

fn update_config_load_state(
    config_load_state: Res<State<ConfigLoadState>>,
    mut next_state: ResMut<NextState<ConfigLoadState>>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    if let ConfigLoadState::Loading = config_load_state.get() {
        if configs.get(&config_resource.handle).is_some() {
            info!("Config is loaded!!");
            next_state.set(ConfigLoadState::Loaded);
        }
    }
}

fn debug_go_to_racing_game_on_spacebar(
    mut next_game_phase: ResMut<NextState<GamePhase>>,
    mut next_game: ResMut<NextState<CurrentGame>>,
    mut debug_player_input: ResMut<DebugPlayerInput>,
) {
    if debug_player_input.button_1.just_pressed() {
        next_game_phase.set(GamePhase::PreGame);
        next_game.set(CurrentGame::Racing);
        info!("Pressed space!!");
    }
}