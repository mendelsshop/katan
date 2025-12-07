use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    common_ui::ButtonInteraction,
    game::{PlaceButton, UI},
    utils::NORMAL_BUTTON,
};

use super::{
    Building, GameState, Input, KatanComponent, Left,
    colors::{CatanColor, CurrentColor},
    positions::BuildingPosition,
    resources::{CITY_RESOURCES, Resources},
    towns::Town,
};

#[derive(Component, Clone, Copy, Debug)]
#[require(KatanComponent, PlaceButton)]
pub struct CityPlaceButton(Resources, BuildingPosition);
#[derive(Debug, Component, Clone, Copy)]
#[require(KatanComponent)]
#[require(Building)]
pub struct City;
pub struct CityUI;
impl UI for CityUI {
    type Pos = BuildingPosition;

    fn bundle(
        city_position: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
        scale: f32,
    ) -> impl Bundle {
        let (x, y) = city_position.positon_to_pixel_coordinates();

        let mesh1 = meshes.add(Rectangle::new(scale * 4.3, scale * 4.3));
        (
            KatanComponent,
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(color.to_bevy_color())),
            Transform::from_xyz(x * scale * 25.6, y * scale * 25.6, 0.0),
        )
    }

    fn resources() -> Resources {
        Resources {
            wood: 0,
            brick: 0,
            sheep: 0,
            wheat: 2,
            ore: 2,
        }
    }
}
#[derive(SystemParam)]
pub struct PlaceCityButtonState<'w> {
    game_state_mut: ResMut<'w, NextState<GameState>>,
    input: ResMut<'w, Input>,
}
impl ButtonInteraction<CityPlaceButton> for PlaceCityButtonState<'_> {
    fn interact(&mut self, CityPlaceButton(cost, position): &CityPlaceButton) {
        let PlaceCityButtonState {
            game_state_mut,
            input,
        } = self;

        **input = Input::AddCity(*position, *cost);
        game_state_mut.set(GameState::Turn);
    }
}

pub fn place_normal_city(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    city_free_q: Query<'_, '_, &Left<City>, With<CatanColor>>,
    town_q: Query<'_, '_, (&'_ Town, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_city_correct_color = city_free_q.get(color_r.0.entity).ok();

    // no cites to place
    let Some(_) = unplaced_city_correct_color.filter(|r| r.0 > 0) else {
        return;
    };

    let current_color_towns = town_q.into_iter().filter(|r| *r.1 == color_r.0.color);

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
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    CityPlaceButton(CITY_RESOURCES, p),
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
