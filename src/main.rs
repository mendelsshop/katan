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

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use bevy_ui_anchor::AnchorUiPlugin;

use crate::{game::GamePlugin, lobby::LobbyPlugin};
#[derive(Debug, Default, Component)]
pub struct MainCamera;

pub static WINDOW_HEIGHT: f32 = 1080.;
pub static WINDOW_WIDTH: f32 = 1920.;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_plugins(AnchorUiPlugin::<MainCamera>::new())
        .add_plugins((LobbyPlugin, GamePlugin))
        .add_systems(Update, resize)
        .run();
}

fn setup(
    mut commands: Commands<'_, '_>,
    primary_window: Single<'_, '_, &Window, With<PrimaryWindow>>,
) {
    commands.spawn((
        Projection::Orthographic(OrthographicProjection {
            scale: (WINDOW_HEIGHT / primary_window.height())
                .max(WINDOW_WIDTH / primary_window.width()),
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
        Camera2d,
        IsDefaultUiCamera,
    ));
}
fn resize(
    mut events: MessageReader<'_, '_, WindowResized>,
    primary_window: Single<'_, '_, Entity, With<PrimaryWindow>>,
    mut projection: Single<'_, '_, &mut Projection, With<MainCamera>>,
) {
    for WindowResized {
        window,
        width,
        height,
    } in events.read()
    {
        if *window == *primary_window
            && let Projection::Orthographic(projection) = (&mut **projection)
        {
            projection.scale = (WINDOW_HEIGHT / height).max(WINDOW_WIDTH / width);
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    GameOver,
}
