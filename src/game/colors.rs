//! data structures for player colors
//! and logic for setting next player via their color (for setup and normal play)
use std::{
    iter::{Chain, Cycle, Rev},
    vec::IntoIter,
};

use bevy::{
    color::{self, palettes::css},
    prelude::*,
};
use bevy_ggrs::LocalPlayers;

use crate::game::PlayerHandle;

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
    pub handle: PlayerHandle,
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
    color_r: &mut ResMut<'_, CurrentColor>,
    color_rotation: &mut ResMut<'_, ColorIterator>,

    local_players: &Res<'_, LocalPlayers>,

    game_state: &mut ResMut<'_, NextState<GameState>>,
    player_banners: &mut Query<'_, '_, (&mut BackgroundColor, &mut Outline, &PlayerBanner)>,
) {
    if let Some((mut background, mut border, _)) = player_banners
        .iter_mut()
        .find(|(_, _, banner)| banner.0 == color_r.0)
    {
        *background = BackgroundColor(background.0.with_alpha(0.5));
        border.color = Color::NONE;
    }
    **color_r = CurrentColor(color_rotation.0.next().unwrap());
    if let Some((mut background, mut border, _)) = player_banners
        .iter_mut()
        .find(|(_, _, banner)| banner.0 == color_r.0)
    {
        // TODO: better shinning effect
        border.color = css::CADET_BLUE.into();
        *background = BackgroundColor(background.0.with_alpha(1.0));
    }
    super::next_player(
        game_state,
        local_players,
        color_r.0,
        GameState::Roll,
        GameState::NotActive,
    );
}

pub fn set_setup_color(
    game_state: &mut ResMut<'_, NextState<GameState>>,
    setup_color_r: &mut ResMut<'_, CurrentSetupColor>,
    setup_color_rotation: &mut ResMut<'_, SetupColorIterator>,

    local_players: &Res<'_, LocalPlayers>,
    player_banners: &mut Query<'_, '_, (&mut BackgroundColor, &mut Outline, &PlayerBanner)>,

    color_r: &mut ResMut<'_, CurrentColor>,
    color_rotation: &mut ResMut<'_, ColorIterator>,
) {
    if let Some(color) = setup_color_rotation.0.next() {
        println!("next color {color:?}");
        if let Some((mut background, mut border, _)) = player_banners
            .iter_mut()
            .find(|(_, _, banner)| banner.0 == setup_color_r.0)
        {
            border.color = Color::NONE;
            *background = BackgroundColor(background.0.with_alpha(0.5));
        }
        **setup_color_r = CurrentSetupColor(color);
        if let Some((mut background, mut border, _)) = player_banners
            .iter_mut()
            .find(|(_, _, banner)| banner.0 == setup_color_r.0)
        {
            border.color = css::CADET_BLUE.into();
            *background = BackgroundColor(background.0.with_alpha(1.));
            // TODO: better shinning effect
        }
        super::next_player(
            game_state,
            local_players,
            setup_color_r.0,
            GameState::SetupRoad,
            GameState::NotActiveSetup,
        );
    } else {
        println!("done");
        // TODO: will this happen fast enough so that the last player wont have option to do it a
        // 3rd time
        game_state.set(GameState::NotActive);
        set_color(
            color_r,
            color_rotation,
            local_players,
            game_state,
            player_banners,
        );
    }
}
