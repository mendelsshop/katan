//! data structures for player colors
//! and logic for setting next player via their color (for setup and normal play)
use std::{
    iter::{Chain, Cycle, Rev},
    vec::IntoIter,
};

pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
use bevy::{
    color::{self, palettes::css},
    prelude::*,
};

use super::{GameState, turn_ui::PlayerBanner};

#[derive(Debug, Resource, Clone, Copy)]

// TODO: what about before turn order decided
pub struct CurrentColor(pub CatanColorRef);

impl From<CurrentColor> for CatanColor {
    fn from(value: CurrentColor) -> Self {
        value.0.color
    }
}
impl From<CurrentColor> for Entity {
    fn from(value: CurrentColor) -> Self {
        value.0.entity
    }
}
#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentSetupColor(pub CatanColorRef);
impl From<CurrentSetupColor> for Entity {
    fn from(value: CurrentSetupColor) -> Self {
        value.0.entity
    }
}
impl From<CurrentSetupColor> for CatanColor {
    fn from(value: CurrentSetupColor) -> Self {
        value.0.color
    }
}
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
pub struct SetupColorIterator(pub Chain<IntoIter<CatanColorRef>, Rev<IntoIter<CatanColorRef>>>);
pub fn set_color(
    mut color_r: ResMut<'_, CurrentColor>,
    color_rotation: ResMut<'_, ColorIterator>,

    mut player_banners: Query<'_, '_, (&mut BackgroundColor, &mut Outline, &PlayerBanner)>,
) {
    if let Some((mut background, mut border, _)) = player_banners
        .iter_mut()
        .find(|(_, _, banner)| banner.0 == color_r.0)
    {
        *background = BackgroundColor(background.0.with_alpha(0.5));
        border.color = Color::NONE;
    }
    *color_r = CurrentColor(color_rotation.into_inner().0.next().unwrap());
    if let Some((mut background, mut border, _)) = player_banners
        .iter_mut()
        .find(|(_, _, banner)| banner.0 == color_r.0)
    {
        // TODO: better shinning effect
        border.color = css::CADET_BLUE.into();
        *background = BackgroundColor(background.0.with_alpha(1.0));
    }
}

pub fn set_setup_color(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut color_r: ResMut<'_, CurrentSetupColor>,
    color_rotation: ResMut<'_, SetupColorIterator>,

    mut player_banners: Query<'_, '_, (&mut BackgroundColor, &mut Outline, &PlayerBanner)>,
) {
    if let Some(color) = color_rotation.into_inner().0.next() {
        if let Some((mut background, mut border, _)) = player_banners
            .iter_mut()
            .find(|(_, _, banner)| banner.0 == color_r.0)
        {
            border.color = Color::NONE;
            *background = BackgroundColor(background.0.with_alpha(0.5));
        }
        *color_r = CurrentSetupColor(color);
        if let Some((mut background, mut border, _)) = player_banners
            .iter_mut()
            .find(|(_, _, banner)| banner.0 == color_r.0)
        {
            border.color = css::CADET_BLUE.into();
            *background = BackgroundColor(background.0.with_alpha(1.));
            // TODO: better shinning effect
        }
    } else {
        // TODO: will this happen fast enough so that the last player wont have option to do it a
        // 3rd time
        game_state.set(GameState::Roll);
    }
}
