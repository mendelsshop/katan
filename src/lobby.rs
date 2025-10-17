use crate::{
    AppState,
    common_ui::{self, ButtonInteraction},
    utils::{
        BACKGROUND_COLOR, BORDER_COLOR_ACTIVE, BORDER_COLOR_INACTIVE, NORMAL_BUTTON, TEXT_COLOR,
    },
};
use bevy::{
    ecs::system::SystemParam,
    input_focus::{InputDispatchPlugin, InputFocus},
    prelude::*,
};
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*, ggrs};
use bevy_matchbox::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputPlugin, TextInputSystem, TextInputTextColor,
    TextInputTextFont, TextInputValue,
};

pub type GgrsSessionConfig = bevy_ggrs::GgrsConfig<u8, PeerId>;
pub struct LobbyPlugin;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::Menu)]

pub enum MenuState {
    #[default]
    Lobby,
    Room,
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Server;
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Room;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct JoinButton;

#[derive(SystemParam)]
pub struct JoinButtonState<'w, 's> {
    room_query: Single<'w, 's, &'static TextInputValue, With<Room>>,
    server_query: Single<'w, 's, &'static TextInputValue, With<Server>>,
    commands: Commands<'w, 's>,
    state: ResMut<'w, NextState<MenuState>>,
}
impl ButtonInteraction<JoinButton> for JoinButtonState<'_, '_> {
    fn interact(&mut self, _: &JoinButton) {
        self.commands
            .insert_resource(MatchboxSocket::new_unreliable(format!(
                "{}/katan?next={}",
                self.server_query.0, self.room_query.0
            )));
        self.state.set(MenuState::Room);
    }

    // TODO: url verification and room verification
}

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputDispatchPlugin)
            .add_plugins(TextInputPlugin)
            .add_plugins(GgrsPlugin::<GgrsSessionConfig>::default())
            .add_sub_state::<MenuState>()
            .add_systems(
                Update,
                focus
                    .run_if(in_state(MenuState::Lobby))
                    .before(TextInputSystem),
            )
            .add_systems(Update, wait_for_players.run_if(in_state(MenuState::Room)))
            .add_systems(
                Update,
                common_ui::button_system_with_generic::<JoinButton, JoinButtonState<'_, '_>>
                    .run_if(in_state(MenuState::Lobby)),
            )
            .add_systems(OnEnter(AppState::Menu), setup_lobby);
    }
}
pub fn setup_lobby(mut commands: Commands<'_, '_>) {
    commands.spawn((
        DespawnOnExit(AppState::Menu),
        Node {
            display: Display::Grid,
            grid_template_rows: vec![
                GridTrack::max_content(),
                GridTrack::max_content(),
                GridTrack::max_content(),
            ],
            row_gap: Val::Percent(1.),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceAround,
            align_content: AlignContent::Center,
            ..Default::default()
        },
        children![
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::max_content(),
                        GridTrack::minmax(
                            MinTrackSizingFunction::Px(200.),
                            MaxTrackSizingFunction::MaxContent
                        ),
                    ],
                    ..Default::default()
                },
                children![
                    (
                        TextFont {
                            font_size: 34.,
                            ..default()
                        },
                        Text::new("server: ")
                    ),
                    (
                        Server,
                        Node {
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        TextInputInactive(true),
                        BorderColor::all(BORDER_COLOR_ACTIVE),
                        BackgroundColor(BACKGROUND_COLOR),
                        TextInput,
                        TextInputValue("ws://127.0.0.1:3536".to_owned()),
                        TextInputTextFont(TextFont {
                            font_size: 34.,
                            ..default()
                        }),
                        bevy_ui_widgets::observe(text_input_in),
                        bevy_ui_widgets::observe(text_input_out),
                        TextInputTextColor(TextColor(TEXT_COLOR)),
                    ),
                ]
            ),
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::max_content(),
                        GridTrack::minmax(
                            MinTrackSizingFunction::Px(200.),
                            MaxTrackSizingFunction::MaxContent
                        ),
                    ],
                    ..Default::default()
                },
                children![
                    (
                        TextFont {
                            font_size: 34.,
                            ..default()
                        },
                        Text::new("players:")
                    ),
                    (
                        Room,
                        Node {
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        TextInputInactive(true),
                        BorderColor::all(BORDER_COLOR_INACTIVE),
                        BackgroundColor(BACKGROUND_COLOR),
                        TextInput,
                        TextInputValue("4".to_owned()),
                        TextInputTextFont(TextFont {
                            font_size: 34.,
                            ..default()
                        }),
                        bevy_ui_widgets::observe(text_input_in),
                        bevy_ui_widgets::observe(text_input_out),
                        TextInputTextColor(TextColor(TEXT_COLOR)),
                    ),
                ]
            ),
            (
                JoinButton,
                TextFont {
                    font_size: 34.,
                    ..default()
                },
                Text::new("join"),
                Node {
                    border: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    justify_self: JustifySelf::Center,
                    justify_content: JustifyContent::End,
                    ..Default::default()
                },
                BackgroundColor(NORMAL_BUTTON),
                BorderColor::all(BORDER_COLOR_INACTIVE),
                Button
            )
        ],
    ));
}
fn wait_for_players(
    mut commands: Commands<'_, '_>,
    mut socket: ResMut<'_, MatchboxSocket>,
    mut next_state: ResMut<'_, NextState<AppState>>,
    room_query: Single<'_, '_, &'static TextInputValue, With<Room>>,
) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = room_query
        .0
        .parse()
        .expect("player count should be a number");
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // determine the seed
    let id = socket.id().expect("no peer id assigned").0.as_u64_pair();
    let mut seed = id.0 ^ id.1;
    for peer in socket.connected_peers() {
        let peer_id = peer.0.as_u64_pair();
        seed ^= peer_id.0 ^ peer_id.1;
    }
    // commands.insert_resource(SessionSeed(seed));

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsSessionConfig>::new()
        .with_num_players(num_players)
        .with_desync_detection_mode(DesyncDetection::On { interval: 1 });

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required because GGRS takes ownership of it)
    let socket = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
    next_state.set(AppState::InGame);
}

fn focus(
    focus: Res<'_, InputFocus>,
    mut text_inputs: Query<'_, '_, (Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    if !focus.is_changed() {
        return;
    }

    for (entity, mut inactive, mut border_color) in &mut text_inputs {
        if focus.0 == Some(entity) {
            inactive.0 = false;
            *border_color = BORDER_COLOR_ACTIVE.into();
        } else {
            inactive.0 = true;
            *border_color = BORDER_COLOR_INACTIVE.into();
        }
    }
}
fn text_input_out(mut trigger: On<'_, '_, Pointer<Out>>, mut focus: ResMut<'_, InputFocus>) {
    focus.0 = None;
    trigger.propagate(false);
}

fn text_input_in(mut trigger: On<'_, '_, Pointer<Over>>, mut focus: ResMut<'_, InputFocus>) {
    focus.0 = Some(trigger.event_target());
    trigger.propagate(false);
}
