use crate::AppState;
use bevy::prelude::*;

pub struct LobbyPlugin;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::Menu)]

pub enum MenuState {
    #[default]
    Lobby,
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Server;
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Room;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_lobby);
    }
}
pub fn setup_lobby(mut commands: Commands<'_, '_>) {
    commands.spawn((
        StateScoped(MenuState::Lobby),
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(50.), GridTrack::percent(50.)],
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        children![
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                    ],
                    ..Default::default()
                },
                children![
                    Text::new("server:"),
                    (Server, Text::new("localhost:7341")),
                    Text::new("room:"),
                    (Room, Text::new("44")),
                ]
            ),
            (Text::new("join"), Button)
        ],
    ));
}
