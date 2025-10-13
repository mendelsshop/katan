#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

mod game;
mod lobby;

use bevy::prelude::*;

use crate::{game::GamePlugin, lobby::LobbyPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_plugins((LobbyPlugin, GamePlugin))
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn(Camera2d);
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}
