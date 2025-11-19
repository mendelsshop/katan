use bevy::prelude::*;

use crate::utils::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

use super::{
    Building, GameState, Input, KatanComponent, Left,
    colors::{CatanColor, CurrentColor},
    positions::BuildingPosition,
    resources::{CITY_RESOURCES, Resources},
    towns::Town,
};
#[derive(Debug, Component, Clone, Copy)]
#[require(KatanComponent)]
#[require(Building)]
pub struct City;
pub fn place_normal_city_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &BuildingRef,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
            &Resources,
        ),
        (Changed<Interaction>, Without<CatanColor>),
    >,
    mut input: ResMut<'_, Input>,
) {
    for (entity, interaction, mut color, mut button, required_resources) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();
                *input = Input::AddCity(entity.1, *required_resources);

                game_state.set(GameState::Turn);
                break;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                button.set_changed();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
#[derive(Debug, Component, Clone, Copy)]
#[require(KatanComponent)]
pub struct BuildingRef(Entity, BuildingPosition);
pub fn place_normal_city(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    city_free_q: Query<'_, '_, &Left<City>, With<CatanColor>>,
    town_q: Query<'_, '_, (Entity, &'_ Town, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_city_correct_color = city_free_q.get(color_r.0.entity).ok();

    // no cites to place
    let Some(_) = unplaced_city_correct_color.filter(|r| r.0 > 0) else {
        return;
    };

    let current_color_towns = town_q.into_iter().filter(|r| *r.2 == color_r.0.color);

    let possibles_cities = current_color_towns.into_iter().map(|(e, _, _, p)| (e, *p));

    let count = possibles_cities
        .filter_map(|(e, p)| {
            let (x, y) = p.positon_to_pixel_coordinates();
            (x != 0. || y != 0.).then_some((x, y, e, p))
        })
        .map(|(x, y, e, p)| {
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Button,
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    BuildingRef(e, p),
                    CITY_RESOURCES,
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )],
            )
        })
        .map(|b| {
            commands.spawn(b);
        })
        .count();
    if count == 0 {
        game_state.set(GameState::Turn);
    }
}
