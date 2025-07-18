use bevy::prelude::*;

use crate::{
    Building, GameState, Left,
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    positions::BuildingPosition,
    resources::{CITY_RESOURCES, Resources},
    towns::Town,
};
#[derive(Debug, Component, Clone, Copy)]
#[require(Building)]
pub struct City;
pub fn place_normal_city_interaction(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut game_state: ResMut<'_, NextState<GameState>>,
    color_r: Res<'_, CurrentColor>,
    mut town_free_q: Query<'_, '_, (&Town, &CatanColor, &mut Left), Without<City>>,
    town_q: Query<'_, '_, (Entity, &Town, &CatanColor, &BuildingPosition)>,
    mut city_free_q: Query<'_, '_, (&City, &CatanColor, &mut Left), Without<Town>>,
    mut resources: ResMut<'_, Resources>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &BuildingPosition,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
            &Resources,
        ),
        (Changed<Interaction>, Without<CatanColor>),
    >,
) {
    for (entity, interaction, mut color, mut button, required_resources) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();

                let town_to_be_replaced =
                    town_q
                        .iter()
                        .find(|(_, _, catan_color, building_position)| {
                            **catan_color == color_r.0 && *building_position == entity
                        });
                if let Some((entity1, _, _, _)) = town_to_be_replaced {
                    commands.entity(entity1).remove::<Town>().insert(City);
                }
                let towns_left = town_free_q.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((_, _, mut left)) = towns_left {
                    *left = Left(left.0 + 1);
                }
                let city_left = city_free_q.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((_, _, mut left)) = city_left {
                    *left = Left(left.0 - 1);
                }

                let player_resources = player_resources.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= *required_resources;
                }
                *resources += *required_resources;

                game_state.set(GameState::Turn);
                let (x, y) = entity.positon_to_pixel_coordinates();

                let mesh1 = meshes.add(Rectangle::new(13.0, 13.));
                commands.spawn((
                    Mesh2d(mesh1),
                    MeshMaterial2d(materials.add(color_r.0.to_bevy_color())),
                    Transform::from_xyz(x * 28.0, y * 28., 0.0),
                ));

                button.set_changed();
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
pub fn place_normal_city(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    city_free_q: Query<'_, '_, (&City, &CatanColor, &Left)>,
    town_q: Query<'_, '_, (&'_ Town, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_city_correct_color = city_free_q.iter().find(|r| r.1 == &color_r.0);

    // no cites to place
    let Some(_) = unplaced_city_correct_color.filter(|r| r.2.0 > 0) else {
        return;
    };

    let current_color_towns = town_q.into_iter().filter(|r| *r.1 == color_r.0);

    let possibles_cities = current_color_towns.into_iter().map(|(_, _, p)| *p);

    let count = possibles_cities
        .filter_map(|p| {
            let (x, y) = p.positon_to_pixel_coordinates();
            (x != 0. || y != 0.).then_some((x, y, p))
        })
        .map(|(x, y, p)| {
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
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        left: Val::Px(x * 28.),
                        bottom: Val::Px(y * 28.),
                        ..default()
                    },
                    p,
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
