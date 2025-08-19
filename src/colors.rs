//! data structures for player colors
//! and logic for setting next player via their color (for setup and normal play)
use std::{
    iter::{Chain, Cycle, Rev},
    vec::IntoIter,
};

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
use bevy::{color, prelude::*};

use crate::GameState;

#[derive(Debug, Resource, Clone, Copy)]

// TODO: what about before turn order decided
pub struct CurrentColor(pub CatanColorRef);

impl From<CurrentColor> for CatanColor {
    fn from(value: CurrentColor) -> Self {
        value.0.color
    }
}
#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentSetupColor(pub CatanColor);
impl From<CurrentSetupColor> for CatanColor {
    fn from(value: CurrentSetupColor) -> Self {
        value.0
    }
}
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatanColor {
    Red,
    Green,
    Blue,
    White,
}
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CatanColorRef {
    pub color: CatanColor,
    pub entity: Entity,
}

impl CatanColorRef {
    pub fn to_bevy_color(self) -> Color {
        self.color.to_bevy_color()
    }
}
impl CatanColor {
    pub fn to_bevy_color(self) -> Color {
        match self {
            Self::Red => color::palettes::basic::RED.into(),
            Self::Green => color::palettes::basic::GREEN.into(),
            Self::Blue => color::palettes::basic::BLUE.into(),
            Self::White => color::palettes::basic::WHITE.into(),
        }
    }
}
#[derive(Resource, Debug)]
pub struct ColorIterator(pub Cycle<IntoIter<CatanColorRef>>);

#[derive(Resource, Debug)]
// TODO: use color ref
pub struct SetupColorIterator(pub Chain<IntoIter<CatanColor>, Rev<IntoIter<CatanColor>>>);
pub fn set_color(mut color_r: ResMut<'_, CurrentColor>, color_rotation: ResMut<'_, ColorIterator>) {
    *color_r = CurrentColor(color_rotation.into_inner().0.next().unwrap());
}

pub fn set_setup_color(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut color_r: ResMut<'_, CurrentSetupColor>,
    color_rotation: ResMut<'_, SetupColorIterator>,
) {
    if let Some(color) = color_rotation.into_inner().0.next() {
        *color_r = CurrentSetupColor(color);
    } else {
        // TODO: will this happen fast enough so that the last player wont have option to do it a
        // 3rd time
        game_state.set(GameState::Roll);
    }
}
