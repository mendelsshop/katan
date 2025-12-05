#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

mod common_ui;
mod game;
mod lobby;
mod utils;

use bevy::prelude::*;

use crate::{game::GamePlugin, lobby::LobbyPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        // .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup)
        // .add_systems(Update, fit_canvas)
        .add_plugins((LobbyPlugin, GamePlugin))
        // .add_plugins(ScalePlugin)
        .run();
}

fn setup(mut commands: Commands<'_, '_>) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    GameOver,
}
