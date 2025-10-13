use crate::AppState;
use bevy::prelude::*;

pub struct LobbyPlugin;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::Menu)]

pub enum MenuState {
    #[default]
    Lobby,
}

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_lobby);
    }
}
pub fn setup_lobby(mut commands: Commands<'_, '_>) {
    commands.spawn((
        StateScoped(MenuState::Lobby),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
    ));
}
