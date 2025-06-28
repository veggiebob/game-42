use bevy::app::Update;
use crate::PlayerNum;
use bevy::prelude::{App, AppExtStates, Assets, Component, NextState, Res, ResMut, State, States};
use crate::config::{Config, ConfigAccessor};

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
    racing::init_app(app);
}

fn config_is_loaded(config_loaded: Res<State<ConfigLoadState>>) -> bool {
    matches!(config_loaded.get(), ConfigLoadState::Loaded)
}

fn update_config_load_state(
    config_load_state: Res<State<ConfigLoadState>>,
    mut next_state: ResMut<NextState<ConfigLoadState>>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    if let ConfigLoadState::Loading = config_load_state.get() {
        if configs.get(&config_resource.handle).is_some() {
            next_state.set(ConfigLoadState::Loaded);
        }
    }
}