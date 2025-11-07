use bevy::prelude::*;
use itertools::Itertools;

use crate::game::{Input, PlayerHandle};

use super::{
    CatanColor, CurrentColor, GameState, Hexagon, Knights, Layout, Left, Number, Resources, Robber,
    VictoryPoints,
    cities::City,
    colors::CatanColorRef,
    development_cards::DevelopmentCards,
    dice,
    larget_army::LargetArmyRef,
    longest_road::{LongestRoadRef, PlayerLongestRoad},
    positions::{BuildingPosition, Position},
    resources::{CITY_RESOURCES, ROAD_RESOURCES, TOWN_RESOURCES},
    roads::Road,
    towns::Town,
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
    game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (&DieButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    board: Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: Query<'_, '_, (&ChildOf, &Town, &BuildingPosition), With<CatanColor>>,
    cities: Query<'_, '_, (&ChildOf, &City, &BuildingPosition), With<CatanColor>>,
    player_resources: Query<'_, '_, &mut Resources, With<CatanColor>>,
    resources: ResMut<'_, Resources>,
    robber: Res<'_, Robber>,
    die_q: Query<'_, '_, (&mut Text, &mut Transform), With<DieButton>>,
) {
    for (_, interaction, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();

                dice::full_roll_dice(
                    board,
                    towns,
                    cities,
                    player_resources,
                    resources,
                    robber,
                    die_q,
                    game_state,
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
    mut input: ResMut<'_, Input>,
    interaction_query: Single<
        '_,
        '_,
        (&NextButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
) {
    let (_, interaction, mut button) = interaction_query.into_inner();
    // for (entity, interaction, mut button) in &mut interaction_query {
    match *interaction {
        Interaction::Pressed => {
            *input = Input::NextColor;

            // game_state.set(GameState::Roll);
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
    interaction_query: Single<
        '_,
        '_,
        (&CityButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity);
    if let Ok(resources) = player_resources {
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
    interaction_query: Single<
        '_,
        '_,
        (&TownButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity);
    if let Ok(resources) = player_resources {
        let (_, interaction, mut button) = interaction_query.into_inner();
        if resources.contains(TOWN_RESOURCES) {
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
    interaction_query: Single<
        '_,
        '_,
        (&RoadButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, &Resources, With<CatanColor>>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.get(color_r.0.entity).ok();
    if let Some(resources) = player_resources {
        let (_, interaction, mut button) = interaction_query.into_inner();
        if resources.contains(ROAD_RESOURCES) {
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
                BorderColor::all(Color::BLACK),
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
                BorderColor::all(Color::BLACK),
                TextColor(Color::BLACK),
                TextLayout::new_with_justify(Justify::Center),
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
                BorderColor::all(Color::BLACK),
                BackgroundColor(Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
                TextColor(Color::BLACK),
                Button,
                Text::new("0"),
                DieButton,
            ),
        ],
    ));
}
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct BannerRef(Entity);

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PlayerBanner(pub CatanColorRef);
pub fn setup_top(
    mut commands: Commands<'_, '_>,
    // the With<...> is a hack to just filter to the players just in case other entities have color
    players: Query<'_, '_, (Entity, &CatanColor, &PlayerHandle)>,
    layout: Res<'_, Layout>,
) {
    let player_count = players.iter().count();
    let banners = players
        .iter()
        .map(|player| {
            let id = commands
                .spawn((
                    Node {
                        display: Display::Grid,
                        ..default()
                    },
                    Outline {
                        width: Val::Px(4.),
                        offset: Val::Px(0.),
                        color: Color::NONE,
                    },
                    BackgroundColor(player.1.to_bevy_color()),
                    // TODO: compartmentalize banner
                    Text::new(""),
                    TextColor(Color::BLACK),
                    PlayerBanner(CatanColorRef {
                        color: *player.1,
                        entity: player.0,
                        handle: *player.2,
                    }),
                ))
                .id();
            commands.entity(player.0).insert(BannerRef(id));
            id
        })
        .collect_vec();

    let mut top = commands.spawn(Node {
        display: Display::Grid,
        grid_template_columns: vec![GridTrack::percent(100. / player_count as f32); player_count],
        column_gap: Val::Px(5.),
        ..default()
    });
    top.add_children(&banners);
    let top = top.id();
    commands.entity(layout.player_banner).add_child(top);
    println!("done setup");
}

pub fn top_interaction(
    mut banners: Query<'_, '_, (&PlayerBanner, &mut Text)>,
    players: Query<
        '_,
        '_,
        (
            &CatanColor,
            &Left<Town>,
            &Left<City>,
            &Left<Road>,
            &DevelopmentCards,
            &Resources,
            &VictoryPoints,
            &BannerRef,
            &Knights,
            Option<&LargetArmyRef>,
            Option<&LongestRoadRef>,
            &PlayerLongestRoad,
        ),
        (
            Or<(
                Changed<Left<Town>>,
                Changed<Left<City>>,
                Changed<Left<Road>>,
                Changed<DevelopmentCards>,
                Changed<Resources>,
                Changed<VictoryPoints>,
                Changed<Knights>,
                Changed<LargetArmyRef>,
                Changed<LongestRoadRef>,
                Changed<PlayerLongestRoad>,
            )>,
        ),
    >,
) {
    for (
        catan_color,
        towns,
        cities,
        roads,
        development_cards,
        resources,
        victory_points,
        banner_ref,
        knights,
        larget_army,
        longest_road,
        longest_road_count,
    ) in players
    {
        if let Ok((player_banner, mut text)) = banners.get_mut(banner_ref.0) {
            *text = Text::new(format!(
                "vps: {}, resources: {}, dev cards: {}, knights: {}{}, roads: {}{}",
                victory_points.actual,
                resources.count(),
                development_cards.count(),
                knights.0,
                if larget_army.is_some() {
                    "(largest army)"
                } else {
                    ""
                },
                longest_road_count.0.len(),
                if longest_road.is_some() {
                    "(longest road)"
                } else {
                    ""
                }
            ));
        }
    }
}
