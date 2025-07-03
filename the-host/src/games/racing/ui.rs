use std::collections::{HashMap, HashSet};
use crate::{PlayerInputs, PlayerMapping, PlayerNum};
use crate::games::{GamePhase, Player};
use crate::games::racing::style::CarStyle;
use bevy::color::{Alpha, Color};
use bevy::prelude::{AlignSelf, Commands, Component, Entity, JustifyContent, JustifyText, Over, Query, Single, Text, With, default, info, Without, AssetServer, Res, TextFont, Saturation, RepeatedGridTrack, GridTrack, State};
use bevy::text::{TextColor, TextLayout};
use bevy::ui::{AlignContent, AlignItems, BackgroundColor, Display, GridPlacement, JustifyItems, Node, UiRect, Val};
use game_42_net::controls::{ButtonType, PlayerInput};
use crate::games::racing::track::{Lap, LapCounter};
/*
Plan:

Pregame:
 - shows a table of who has joined and their names (blurb)
 - each row also has an indicator of if they're ready (indicator)

During Game:
 - Shows a smaller table in the corner with players (blurbs) and their laps (indicator)


*/

// marks a racing UI entity (for teardown)
#[derive(Component)]
pub struct UiMarker;

#[derive(Component)]
pub struct PlayerBlurb {
    number: PlayerNum,
}

#[derive(Component)]
pub struct PlayerIndicator {
    ready: bool,
    lap: Lap,
}

#[derive(Component)]
pub struct PlayerRosterRow {
    row: i16,
}

#[derive(Component)]
pub struct PlayerRef(Player);

#[derive(Component)]
pub struct OverlayContainerMarker;

pub fn start_pregame_ui(mut commands: Commands) {
    info!("Spawning racing pre-game UI");
    let ui_grid = Node {
        display: Display::Grid,
        margin: UiRect::all(Val::Auto),
        grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
        min_width: Val::VMin(70.0),
        min_height: Val::VMin(70.0),
        justify_content: JustifyContent::Center,
        justify_items: JustifyItems::Center, // horizontal alignments
        align_items: AlignItems::Center,
        ..default()
    };
    commands.spawn((
        BackgroundColor(Color::WHITE.with_alpha(0.5)),
        OverlayContainerMarker,
        UiMarker,
        ui_grid,
    ));
}

/// Update player indicators in the table (ready, lapcount, etc.)
pub fn update_indicators(
    players: Query<(&Player, &LapCounter)>,
    player_input: Res<PlayerInputs>,
    player_mapping: Res<PlayerMapping>,
    indicators: Query<(&PlayerRef, &mut PlayerIndicator)>,
) {
    let players_laps: HashMap<&Player, &LapCounter> = players.iter()
        .collect();
    for (player, mut indicator) in indicators {
        let player = &player.0;
        if let Some(player_input) = player_mapping.0.get(&player.0).and_then(|user| player_input.0.get(user)) {
            indicator.ready = player_input.is_pressed(ButtonType::A);
        }
        if let Some(lap) = players_laps.get(player) {
            indicator.lap = lap.lap();
        }
    }
}

pub fn update_table_ui(
    phase: Res<State<GamePhase>>,
    blurbs: Query<(&PlayerRef, &PlayerBlurb, &mut Text), Without<PlayerIndicator>>,
    indicators: Query<(&PlayerIndicator, &mut Text)>,
) {
    for (_player, blurb, mut text) in blurbs {
        text.0 = format!("Player {}", blurb.number);
    }
    for (indicator, mut text) in indicators {
        match phase.get() {
            GamePhase::PreGame => {
                let ind = if indicator.ready {
                    "READY"
                } else {
                    "-"
                };
                text.0 = format!("{}", ind);
            }
            GamePhase::PlayingGame => {
                text.0 = format!("{}", indicator.lap);
            }
            _ => {}
        }
    }
}

/// Update the roster when new players join/leave
pub fn roster_join_leave(
    mut commands: Commands,
    players: Query<(&Player, &CarStyle)>,
    roster_rows: Query<(Entity, &PlayerRosterRow, &PlayerRef)>,
    overlay: Single<Entity, With<OverlayContainerMarker>>,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // let font = asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf");
    let tf = TextFont {
        font: font.clone(),
        ..default()
    };
    let mut new_child_ui = vec![];
    let current_roster_rows: HashSet<_> = roster_rows.iter().map(|(_e, _prr, pr)| pr.0.0).collect();
    let mut last_row = roster_rows.iter().fold(0, |acc, (_e, prr, _p)| prr.row.max(acc));
    let current_players: HashSet<_> = players.iter().map(|(player, style)| player.0).collect();
    for (player, style) in players {
        if !current_roster_rows.contains(&player.0) {
            // new player spawned!
            let rr = last_row + 1;
            last_row += 1;
            let row_height = Val::VMin(10.0);
            let blurb = commands
                .spawn((
                    UiMarker,
                    PlayerRef(player.clone()),
                    PlayerBlurb { number: player.0 },
                    Node {
                        grid_row: GridPlacement::start_span(rr, 1),
                        grid_column: GridPlacement::start_span(1, 1),
                        padding: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    PlayerRosterRow { row: rr },
                    BackgroundColor(style.color.with_saturation(0.7)),
                    Text::new("Joining..."),
                    TextColor(Color::WHITE),
                    TextLayout::new_with_justify(JustifyText::Center),
                    tf.clone(),
                ))
                .id();
            let indicator = commands
                .spawn((
                    UiMarker,
                    PlayerRef(player.clone()),
                    PlayerIndicator { ready: false, lap: 0 },
                    Node {
                        grid_row: GridPlacement::start_span(rr, 1),
                        grid_column: GridPlacement::start_span(2, 1),
                        ..default()
                    },
                    PlayerRosterRow { row: rr },
                    BackgroundColor(Color::BLACK),
                    Text::new(""),
                    TextColor(Color::WHITE),
                    TextLayout::new_with_justify(JustifyText::Center),
                    tf.clone(),
                ))
                .id();
            new_child_ui.push(blurb);
            new_child_ui.push(indicator);
        }
    }

    for (e, _prr, p) in roster_rows {
        if !current_players.contains(&p.0.0) {
            commands.entity(e).despawn();
        }
    }
    commands.entity(*overlay).add_children(&new_child_ui);
}

/// Move UI for race mode
pub fn ui_to_playing_transition(
    overlay: Single<(Entity, &mut Node), With<OverlayContainerMarker>>,
) {
    let (ent, mut node) = overlay.into_inner();
    node.margin.left = Val::Px(5.0);
    node.margin.bottom = Val::Px(5.0);
    node.min_width = Val::VMin(0.15);
    node.min_height = Val::VMin(0.15);
}

pub fn teardown_ui(mut commands: Commands, ents: Query<Entity, With<UiMarker>>) {
    info!("Tearing down UI");
    for ent in ents {
        commands.entity(ent).despawn();
    }
}
