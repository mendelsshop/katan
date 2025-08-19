use bevy::prelude::*;

use crate::{
    CatanColor, CurrentColor, GameState, Hexagon, Layout, Number, Resources, RoadUI, Robber, Town,
    TownUI, UI,
    cities::City,
    positions::{BuildingPosition, Position},
    resources::CITY_RESOURCES,
};
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
// button in game to start road placement ui
pub struct RoadButton;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
// button in game to obtain a developmennt card
pub struct DevelopmentCardButton;
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
// button in game to start town placement ui
pub struct TownButton;
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
// button in game to start city placement ui
pub struct CityButton;
#[derive(Component, PartialEq, Eq, Default, Clone, Copy)]
pub struct DieButton;

#[derive(Component, PartialEq, Eq, Default, Clone, Copy)]
pub struct NextButton;
// for roll there are two dice so it cannot be a single (its probably possible to have on dice
// thing which looks like two dice)
pub fn turn_ui_roll_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (&DieButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    board: Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: Query<'_, '_, (&Town, &CatanColor, &BuildingPosition)>,
    cities: Query<'_, '_, (&City, &CatanColor, &BuildingPosition)>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
    mut resources: ResMut<'_, Resources>,
    robber: Res<'_, Robber>,
    mut die_q: Query<'_, '_, (&mut Text, &mut Transform), With<DieButton>>,
) {
    for (_, interaction, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();

                crate::dice::full_roll_dice(
                    &board,
                    &towns,
                    &cities,
                    &mut player_resources,
                    &mut resources,
                    robber,
                    &mut die_q,
                    &mut game_state,
                );
                button.set_changed();
                break;
            }
            Interaction::Hovered => {
                button.set_changed();
            }
            Interaction::None => {}
        }
    }
}
pub fn turn_ui_next_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    interaction_query: Single<'_, (&NextButton, &Interaction, &mut Button), Changed<Interaction>>,
) {
    let (_, interaction, mut button) = interaction_query.into_inner();
    // for (entity, interaction, mut button) in &mut interaction_query {
    match *interaction {
        Interaction::Pressed => {
            game_state.set(GameState::Roll);
            button.set_changed();
        }
        Interaction::Hovered => {
            button.set_changed();
        }
        Interaction::None => {}
    }
    // }
}

// TODO: combine with turn_ui_road_interaction
pub fn turn_ui_city_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    interaction_query: Single<'_, (&CityButton, &Interaction, &mut Button), Changed<Interaction>>,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity);
    if let Ok((resources)) = player_resources {
        let (_, interaction, mut button) = interaction_query.into_inner();
        if resources.contains(CITY_RESOURCES) {
            match *interaction {
                Interaction::Pressed => {
                    button.set_changed();

                    game_state.set(GameState::PlaceCity);
                    button.set_changed();
                }
                Interaction::Hovered => {
                    button.set_changed();
                }
                Interaction::None => {}
            }
        } else {
            // TODO: grey out
        }
    }
}
pub fn turn_ui_town_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    interaction_query: Single<'_, (&TownButton, &Interaction, &mut Button), Changed<Interaction>>,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity);
    if let Ok(resources) = player_resources {
        let (_, interaction, mut button) = interaction_query.into_inner();
        if resources.contains(TownUI::resources()) {
            match *interaction {
                Interaction::Pressed => {
                    button.set_changed();

                    game_state.set(GameState::PlaceTown);
                    button.set_changed();
                }
                Interaction::Hovered => {
                    button.set_changed();
                }
                Interaction::None => {}
            }
        } else {
            // TODO: grey out
        }
    }
}
pub fn turn_ui_road_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    interaction_query: Single<'_, (&RoadButton, &Interaction, &mut Button), Changed<Interaction>>,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity).ok();
    if let Some(resources) = player_resources {
        let (_, interaction, mut button) = interaction_query.into_inner();
        if resources.contains(RoadUI::resources()) {
            match *interaction {
                Interaction::Pressed => {
                    button.set_changed();

                    game_state.set(GameState::PlaceRoad);
                    button.set_changed();
                }
                Interaction::Hovered => {
                    button.set_changed();
                }
                Interaction::None => {}
            }
        } else {
            // TODO: grey out
        }
    }
}
pub fn show_turn_ui(
    mut commands: Commands<'_, '_>,
    asset_server: Res<'_, AssetServer>,
    layout: Res<'_, Layout>,
) {
    let road_icon = asset_server.load("road.png");
    let town_icon = asset_server.load("house.png");
    let city_icon = asset_server.load("city.png");
    let development_card_back_icon = asset_server.load("development_card_back.png");
    let next_turn_icon = asset_server.load("x.png");
    commands.entity(layout.ui).insert((
        Node {
            grid_template_columns: vec![
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
            ],
            column_gap: Val::Px(5.),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            display: Display::Grid,
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Px(25.0),
                    height: Val::Px(10.0),
                    ..default()
                },
                Button,
                ImageNode::new(road_icon),
                RoadButton,
            ),
            (
                Node {
                    width: Val::Px(25.0),
                    height: Val::Px(25.0),
                    ..default()
                },
                Button,
                ImageNode::new(town_icon),
                TownButton,
            ),
            (
                Node {
                    width: Val::Px(37.306),
                    height: Val::Px(25.0),
                    ..default()
                },
                Button,
                ImageNode::new(city_icon),
                CityButton,
            ),
            (
                // TODO: blur out if not any dev cards left, or maybe do this in iteraction
                Node {
                    width: Val::Px(17.),
                    height: Val::Px(25.0),
                    ..default()
                },
                Button,
                ImageNode::new(development_card_back_icon),
                DevelopmentCardButton,
            ),
            (
                ImageNode::new(next_turn_icon),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                Button,
                NextButton,
                Outline {
                    width: Val::Px(4.),
                    offset: Val::Px(0.),
                    color: Color::BLACK,
                },
                BorderColor(Color::BLACK),
            )
        ],
    ));
    // TODO: better way to do ui layouting
    commands.entity(layout.board).with_child((
        Node {
            align_self: AlignSelf::End,
            justify_self: JustifySelf::End,
            right: Val::Percent(25.),
            bottom: Val::Percent(25.),

            ..default()
        },
        children![
            (
                Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::End,
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                Transform::from_rotation(Quat::from_rotation_z(6.)),
                Button,
                Text::new("0"),
                BorderColor(Color::BLACK),
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(JustifyText::Center),
                BackgroundColor(Color::WHITE),
                Outline {
                    width: Val::Px(4.),
                    offset: Val::Px(0.),
                    color: Color::BLACK,
                },
                DieButton,
            ),
            (
                Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::End,
                    width: Val::Px(20.),
                    height: Val::Px(20.0),

                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                Transform::from_rotation(Quat::from_rotation_z(-25.)),
                Outline {
                    width: Val::Px(4.),
                    offset: Val::Px(0.),
                    color: Color::BLACK,
                },
                BorderColor(Color::BLACK),
                BackgroundColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::BLACK),
                Button,
                Text::new("0"),
                DieButton,
            ),
        ],
    ));
}
