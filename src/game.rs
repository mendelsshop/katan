use std::{
    marker::PhantomData,
    mem,
    ops::{AddAssign, SubAssign},
};
mod cities;
mod colors;
mod development_card_actions;
mod development_cards;
mod dice;
mod larget_army;
mod longest_road;
mod positions;
mod resources;
mod resources_management;
mod roads;
mod robber;
mod setup_game;
mod towns;
mod turn_ui;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ggrs::{
    GgrsSchedule, LocalInputs, LocalPlayers, PlayerInputs, ReadInputs, RollbackApp,
    RollbackFrameRate, Session,
    ggrs::{GgrsEvent, InputStatus},
};
use bevy_matchbox::prelude::PeerId;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use self::{
    cities::BuildingRef,
    cities::City,
    colors::{
        CatanColor, CatanColorRef, ColorIterator, CurrentColor, CurrentSetupColor,
        SetupColorIterator,
    },
    colors::{set_color, set_setup_color},
    development_card_actions::{
        MonopolyButton, RoadBuildingState, YearOfPlentyButton, YearOfPlentyState,
    },
    development_cards::DevelopmentCard,
    development_cards::{DevelopmentCards, DevelopmentCardsPile},
    larget_army::LargestArmyPlugin,
    longest_road::LongestRoadPlugin,
    longest_road::PlayerLongestRoad,
    positions::{BuildingPosition, Position, RoadPosition},
    resources::DEVELOPMENT_CARD_RESOURCES,
    resources::Resources,
    resources_management::ResourceManagmentPlugin,
    resources_management::TradingResources,
    roads::{PlaceRoadButtonState, RoadPlaceButton},
    roads::{Road, RoadUI},
    robber::{Robber, RobberButton, RobberChooseColorButton, RobberDiscard, RobberResourceSpinner},
    setup_game::Ports,
    towns::{PlaceTownButtonState, TownPlaceButton},
    towns::{Town, TownUI},
    turn_ui::{DieButton, PlayerBanner},
};
use crate::{
    AppState, common_ui,
    game::resources_management::{AcceptTrade, RejectTrade},
};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Resource)]
pub struct LocalPlayer(pub CatanColorRef);
#[derive(PartialEq, Eq, Clone, Copy, Default, Deserialize, Serialize, Debug, Resource)]
pub enum Input {
    #[default]
    None,
    NextColor,
    // player, place, cost multiplier
    // also used for road building
    AddRoad(RoadPosition, Resources),
    AddCity(Entity, BuildingPosition, Resources),
    AddTown(BuildingPosition, Resources, bool),
    TakeDevelopmentCard,
    Roll(u8, u8, u8),
    // for each year of plenty done twice
    // maybe just send one with two resources
    YearOfPlenty(resources::Resource),
    Monopoly(resources::Resource),
    // person picked from, and card picked, if there is a discard(cards discarded if needed)
    Knight(Entity, resources::Resource),
    // discard
    RobberDiscard(Resources),
    RobberDiscardInit,
    // need way to cancel trade
    Trade(TradingResources),         // interactive(TradeResponce)
    TradeResponce(TradingResources), // interactive(TradeAccept)
    TradeAccept(TradingResources, Entity),
    BankTrade(TradingResources),
}
pub type GgrsSessionConfig = bevy_ggrs::GgrsConfig<Input, PeerId>;
pub struct GamePlugin;

fn read_local_inputs(
    mut commands: Commands<'_, '_>,
    local_players: Res<'_, LocalPlayers>,
    mut current_inputs: ResMut<'_, Input>,
) {
    commands.insert_resource(LocalInputs::<GgrsSessionConfig>(
        // updating of the input should happen on the fly
        local_players
            .0
            .iter()
            .map(|h| (*h, *current_inputs))
            .collect(),
    ));
    *current_inputs = Input::None;
}

#[derive(Resource, Default, Clone, Copy, Debug, Deref, DerefMut)]
pub struct SessionSeed(pub u64);

#[derive(SystemParam)]
pub struct UpdateState<'w, 's> {
    inputs: Res<'w, PlayerInputs<GgrsSessionConfig>>,
    players: Query<
        'w,
        's,
        (
            Entity,
            &'static PlayerHandle,
            &'static mut Resources,
            &'static CatanColor,
            &'static mut VictoryPoints,
            &'static mut Ports,
            &'static mut Left<Road>,
            &'static mut Left<Town>,
            &'static mut Left<City>,
            &'static mut DevelopmentCards,
        ),
    >,

    ports: Query<'w, 's, (&'static BuildingPosition, &'static Port)>,
    player_banners: Query<
        'w,
        's,
        (
            &'static mut BackgroundColor,
            &'static mut Outline,
            &'static PlayerBanner,
        ),
    >,
    bank: ResMut<'w, Resources>,
    layout: Res<'w, Layout>,
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    mut_game_state: ResMut<'w, NextState<GameState>>,
    game_state: ResMut<'w, State<GameState>>,
    setup_color_r: ResMut<'w, CurrentSetupColor>,
    color_r: ResMut<'w, CurrentColor>,
    setup_color_rotation: ResMut<'w, SetupColorIterator>,
    color_rotation: ResMut<'w, ColorIterator>,

    free_dev_cards: ResMut<'w, DevelopmentCardsPile>,
    local_player: Res<'w, LocalPlayer>,
}
fn update_from_inputs(
    UpdateState {
        inputs,
        players,
        ports,
        mut player_banners,
        mut bank,
        layout,
        mut commands,
        mut meshes,
        mut materials,
        mut mut_game_state,
        game_state,
        mut setup_color_r,
        mut color_r,
        mut setup_color_rotation,
        mut color_rotation,
        mut free_dev_cards,
        local_player,
    }: UpdateState<'_, '_>,
) {
    let count = inputs.iter().filter(|(i, _)| *i != Input::None).count();
    if count != 0 {
        println!(
            "new {:?} {:?}",
            inputs.iter().collect_vec(),
            game_state.get()
        );
    }
    for (
        entity,
        player_handle,
        mut player_resources,
        color,
        mut vps,
        mut player_ports,
        mut roads_left,
        mut towns_left,
        mut cities_left,
        mut player_dev_cards,
    ) in players
    {
        let (input, _state) = inputs[player_handle.0];
        if _state == InputStatus::Predicted {
            continue;
        }
        match input {
            Input::None => {}
            Input::NextColor => {
                if matches!(
                    *game_state.get(),
                    GameState::NotActiveSetup | GameState::SetupTown
                ) {
                    set_setup_color(
                        &mut mut_game_state,
                        &mut setup_color_r,
                        &mut setup_color_rotation,
                        &local_player,
                        &mut player_banners,
                        &mut color_r,
                        &mut color_rotation,
                    );
                } else {
                    set_color(
                        &mut color_r,
                        &mut color_rotation,
                        &local_player,
                        &mut mut_game_state,
                        &mut player_banners,
                    );
                    commands
                        .entity(layout.trades)
                        .clear_children()
                        .with_child(Text("trades".to_string()));
                }
            }
            Input::AddRoad(road_position, cost) => {
                println!("new road");
                bank.add_assign(cost);
                player_resources.sub_assign(cost);
                commands
                    .entity(entity)
                    .with_child((Road, road_position, *color));
                commands.spawn(RoadUI::bundle(
                    road_position,
                    &mut meshes,
                    &mut materials,
                    *color,
                ));

                roads_left.0 -= 1;
            }
            // make sure entity(of child town) is synced between client
            Input::AddCity(entity, city_position, cost) => {
                commands.entity(entity).remove::<Town>().insert(City);
                towns_left.0 += 1;
                cities_left.0 -= 1;
                *player_resources -= cost;
                vps.actual += 1;

                bank.add_assign(cost);
                let (x, y) = city_position.positon_to_pixel_coordinates();

                let mesh1 = meshes.add(Rectangle::new(13.0, 13.));
                commands.spawn((
                    Mesh2d(mesh1),
                    MeshMaterial2d(materials.add(color_r.0.to_bevy_color())),
                    Transform::from_xyz(x * 77.0, y * 77., 0.0),
                ));
            }
            Input::AddTown(town_position, cost, next) => {
                bank.add_assign(cost);
                player_resources.sub_assign(cost);
                commands
                    .entity(entity)
                    .with_child((Town, town_position, *color));
                commands.spawn(TownUI::bundle(
                    town_position,
                    &mut meshes,
                    &mut materials,
                    *color,
                ));
                if let Some((_, port)) = ports
                    .iter()
                    .find(|(port_position, _)| **port_position == town_position)
                {
                    *player_ports += *port;
                }

                vps.actual += 1;
                towns_left.0 -= 1;
                if next {
                    set_setup_color(
                        &mut mut_game_state,
                        &mut setup_color_r,
                        &mut setup_color_rotation,
                        &local_player,
                        &mut player_banners,
                        &mut color_r,
                        &mut color_rotation,
                    );
                }
            }
            Input::TakeDevelopmentCard => {
                if let Some(card) = free_dev_cards.0.pop() {
                    let required_resources = DEVELOPMENT_CARD_RESOURCES;
                    *player_resources -= required_resources;
                    bank.add_assign(required_resources);
                    if card == DevelopmentCard::VictoryPoint {
                        vps.from_development_cards += 1;
                    }
                    *player_dev_cards.get_mut(card) += 1;
                }
            }
            // handeld by update_from_input_roll
            Input::Roll(_number, _d1, _d2) => (),
            Input::YearOfPlenty(resource) => {
                *bank.get_mut(resource) -= 1;
                *player_resources.get_mut(resource) += 1;
            }
            // handeld by update_from_monopoly
            Input::Monopoly(_resource) => (),
            // handeld by update_from_knight
            Input::Knight(_player, _resource) => (),

            Input::RobberDiscardInit => {
                if local_player.0.entity == entity {
                    mut_game_state.set(GameState::RobberDiscardResources);
                } else {
                    mut_game_state.set(GameState::RobberDiscardResourcesInActive);
                }
            }
            Input::RobberDiscard(resources) => {
                bank.add_assign(resources);
                player_resources.sub_assign(resources);
            }

            Input::Trade(trade) => {
                // we show even if its not possible as after another trade it could be
                if entity != local_player.0.entity {
                    commands.entity(layout.trades).with_child((
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![
                                GridTrack::auto(),
                                GridTrack::auto(),
                                GridTrack::auto(),
                            ],
                            ..Default::default()
                        },
                        children![
                            Text::new(trade.to_string()),
                            (Button, Text::new("x"), RejectTrade { trade }),
                            (Button, Text::new("Ok"), AcceptTrade { trade })
                        ],
                    ));
                }
            }
            Input::TradeResponce(_) => if *game_state.get() == GameState::Turn {},
            // handeld by update_from_trade_accept
            Input::TradeAccept(_r, _e) => (),
            Input::BankTrade(trading_resources) => {
                bank.sub_assign(trading_resources);
                player_resources.add_assign(trading_resources);
            }
        }
    }
}
fn update_from_trade_accept(
    inputs: Res<'_, PlayerInputs<GgrsSessionConfig>>,
    players: Query<'_, '_, (Entity, &PlayerHandle)>,
    mut player_resources_q: Query<'_, '_, &mut Resources, With<CatanColor>>,
) {
    for player in players {
        if let (Input::TradeAccept(r, trader), InputStatus::Confirmed) = inputs[player.1.0] {
            if let Ok(mut robbed_resources) = player_resources_q.get_mut(trader) {
                robbed_resources.sub_assign(r);
            }
            if let Ok(mut resources) = player_resources_q.get_mut(player.0) {
                resources.add_assign(r);
            }
        }
    }
}
fn update_from_knight(
    inputs: Res<'_, PlayerInputs<GgrsSessionConfig>>,
    players: Query<'_, '_, (Entity, &PlayerHandle)>,
    mut player_resources_q: Query<'_, '_, &mut Resources, With<CatanColor>>,
) {
    for player in players {
        if let (Input::Knight(robbed_player, resource), InputStatus::Confirmed) = inputs[player.1.0]
        {
            if let Ok(mut robbed_resources) = player_resources_q.get_mut(robbed_player) {
                *robbed_resources.get_mut(resource) -= 1;
            }
            if let Ok(mut resources) = player_resources_q.get_mut(player.0) {
                *resources.get_mut(resource) += 1;
            }
        }
    }
}
fn update_from_monopoly(
    inputs: Res<'_, PlayerInputs<GgrsSessionConfig>>,
    players: Query<'_, '_, (Entity, &PlayerHandle)>,
    mut player_resources_q: Query<'_, '_, &mut Resources, With<CatanColor>>,
) {
    for player in players {
        if let (Input::Monopoly(resource), InputStatus::Confirmed) = inputs[player.1.0] {
            let taken = player_resources_q
                .iter_mut()
                .map(|mut r| {
                    let mut taken = 0;
                    let original = r.get_mut(resource);
                    mem::swap(&mut taken, original);
                    taken
                })
                .sum::<u8>();

            if let Ok(mut resources) = player_resources_q.get_mut(player.0) {
                // we reassign because when we go through the resources we also go through
                // current color's resources
                *resources.get_mut(resource) = taken;
            }
            break;
        }
    }
}
fn update_from_inputs_roll(
    inputs: Res<'_, PlayerInputs<GgrsSessionConfig>>,
    players: Query<'_, '_, &PlayerHandle>,

    player_resources_q: Query<'_, '_, &mut Resources, With<CatanColor>>,

    mut die_q: Query<'_, '_, (&mut Text, &mut Transform), With<DieButton>>,

    board: Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: Query<'_, '_, (&ChildOf, &Town, &BuildingPosition), With<CatanColor>>,
    cities: Query<'_, '_, (&ChildOf, &City, &BuildingPosition), With<CatanColor>>,

    resources: ResMut<'_, Resources>,
    robber: Res<'_, Robber>,
) {
    for player in players {
        if let (Input::Roll(roll, d1, d2), InputStatus::Confirmed) = inputs[player.0] {
            dice::update_dice(&mut die_q, d1, d2);
            dice::distribute_resources(
                roll,
                board,
                towns,
                cities,
                player_resources_q,
                resources,
                robber,
            );
            break;
        }
    }
}
const FPS: usize = 60;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<GameState>()
            .add_sub_state::<YearOfPlentyState>()
            .add_sub_state::<RoadBuildingState>()
            .add_systems(ReadInputs, read_local_inputs)
            .add_plugins((
                ResourceManagmentPlugin,
                LargestArmyPlugin,
                LongestRoadPlugin,
            ))
            .insert_resource(Input::None)
            .insert_resource(RollbackFrameRate(FPS))
            .rollback_component_with_copy::<towns::Town>()
            .rollback_component_with_copy::<Left<towns::Town>>()
            .rollback_component_with_copy::<Left<cities::City>>()
            .rollback_component_with_copy::<Resources>()
            .rollback_component_with_copy::<Ports>()
            .rollback_component_with_copy::<VictoryPoints>()
            .rollback_component_with_clone::<PlayerLongestRoad>()
            .rollback_component_with_copy::<Knights>()
            .rollback_component_with_copy::<Left<roads::Road>>()
            .rollback_component_with_clone::<Mesh2d>()
            .rollback_component_with_copy::<CatanColorRef>()
            .rollback_resource_with_copy::<Robber>()
            .rollback_component_with_clone::<MeshMaterial2d<ColorMaterial>>()
            .rollback_component_with_clone::<Node>()
            .rollback_component_with_copy::<Transform>()
            .rollback_component_with_copy::<BuildingPosition>()
            .rollback_component_with_copy::<CatanColor>()
            .rollback_component_with_copy::<RoadPosition>()
            .rollback_component_with_copy::<cities::City>()
            .rollback_component_with_copy::<roads::Road>()
            .add_systems(
                Update,
                handle_ggrs_events.run_if(in_state(AppState::InGame)),
            )
            .insert_resource(BoardSize(3))
            .init_resource::<Robber>()
            .init_resource::<RobberDiscard>()
            .insert_resource(Resources::new_game())
            // TODO: is there way to init resource
            // without giving a value
            .insert_resource(CurrentColor(CatanColorRef {
                color: CatanColor::White,
                entity: Entity::PLACEHOLDER,
                handle: PlayerHandle(0),
            }))
            .insert_resource(CurrentSetupColor(CatanColorRef {
                color: CatanColor::White,
                entity: Entity::PLACEHOLDER,
                handle: PlayerHandle(0),
            }))
            .add_systems(OnEnter(AppState::InGame), game_setup)
            .add_systems(
                OnEnter(GameState::Roll),
                (
                    cleanup_ui::<DevelopmentCard>,
                    development_cards::setup_show_dev_cards,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    development_cards::show_dev_cards,
                    development_card_actions::development_card_action_interaction,
                )
                    .run_if(in_state(GameState::Turn)),
            )
            .add_systems(OnEnter(GameState::SetupRoad), roads::place_setup_road)
            .add_systems(OnEnter(GameState::SetupTown), towns::place_setup_town)
            .add_systems(
                Update,
                turn_ui::top_interaction.run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(GameState::PlaceRobber), robber::place_robber)
            .add_systems(
                OnEnter(GameState::RobberDiscardResourcesInActive),
                robber::take_extra_resources,
            )
            .add_systems(
                OnEnter(GameState::RobberDiscardResources),
                robber::take_extra_resources,
            )
            .add_systems(
                OnExit(GameState::SetupRoad),
                cleanup_button::<RoadPlaceButton>,
            )
            .add_systems(
                OnExit(GameState::SetupTown),
                cleanup_button::<TownPlaceButton>,
            )
            .add_systems(
                OnExit(GameState::PlaceRoad),
                cleanup_button::<RoadPlaceButton>,
            )
            .add_systems(
                OnExit(RoadBuildingState::Road1),
                cleanup_button::<RoadPlaceButton>,
            )
            .add_systems(
                OnExit(RoadBuildingState::Road2),
                cleanup_button::<RoadPlaceButton>,
            )
            .add_systems(
                OnExit(GameState::PlaceTown),
                cleanup_button::<TownPlaceButton>,
            )
            .add_systems(OnExit(GameState::PlaceCity), cleanup_button::<BuildingRef>)
            .add_systems(
                OnExit(GameState::PlaceRobber),
                cleanup_button::<RobberButton>,
            )
            .add_systems(
                OnExit(GameState::RobberPickColor),
                cleanup_button::<RobberChooseColorButton>,
            )
            .add_systems(
                OnExit(GameState::Monopoly),
                cleanup_button::<MonopolyButton>,
            )
            .add_systems(
                OnEnter(GameState::Monopoly),
                development_card_actions::monopoly_setup,
            )
            .add_systems(
                Update,
                development_card_actions::monopoly_interaction
                    .run_if(in_state(GameState::Monopoly)),
            )
            .add_systems(
                OnExit(GameState::YearOfPlenty),
                cleanup_button::<YearOfPlentyButton>,
            )
            .add_systems(
                OnEnter(GameState::YearOfPlenty),
                development_card_actions::setup_year_of_plenty,
            )
            .add_systems(
                Update,
                development_card_actions::year_of_plenty_interaction
                    .run_if(in_state(GameState::YearOfPlenty)),
            )
            .add_systems(OnEnter(GameState::PlaceRoad), roads::place_normal_road::<1>)
            .add_systems(
                Update,
                development_cards::show_dev_cards.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnEnter(RoadBuildingState::Road1),
                roads::place_normal_road::<0>,
            )
            .add_systems(
                OnEnter(RoadBuildingState::Road2),
                roads::place_normal_road::<0>,
            )
            .add_systems(OnEnter(GameState::PlaceTown), towns::place_normal_town)
            .add_systems(OnEnter(GameState::PlaceCity), cities::place_normal_city)
            .add_systems(
                Update,
                turn_ui::turn_ui_road_interaction.run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                turn_ui::turn_ui_town_interaction.run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                turn_ui::turn_ui_city_interaction.run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                turn_ui::turn_ui_roll_interaction.run_if(in_state(GameState::Roll)),
            )
            .add_systems(
                Update,
                robber::choose_player_to_take_from_interaction
                    .run_if(in_state(GameState::RobberPickColor)),
            )
            .add_systems(
                Update,
                robber::place_robber_interaction.run_if(in_state(GameState::PlaceRobber)),
            )
            .add_systems(
                OnEnter(GameState::Start),
                (
                    turn_ui::setup_top,
                    (|mut game_state: ResMut<'_, NextState<GameState>>,
                      mut color_r: ResMut<'_, CurrentColor>,
                      mut color_rotation: ResMut<'_, ColorIterator>,

                      local_players: Res<'_, LocalPlayer>,

                      mut setup_color_r: ResMut<'_, CurrentSetupColor>,
                      mut setup_color_rotation: ResMut<'_, SetupColorIterator>,
                      mut player_banners: Query<
                        '_,
                        '_,
                        (&mut BackgroundColor, &mut Outline, &PlayerBanner),
                    >| {
                        set_setup_color(
                            &mut game_state,
                            &mut setup_color_r,
                            &mut setup_color_rotation,
                            &local_players,
                            &mut player_banners,
                            &mut color_r,
                            &mut color_rotation,
                        );
                    }),
                    turn_ui::show_turn_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                // TODO: if in turn or place state
                turn_ui::turn_ui_next_interaction.run_if({
                    move |current_state: Option<Res<'_, State<GameState>>>| match current_state {
                        Some(current_state) => ![
                            GameState::PlaceRobber,
                            GameState::Monopoly,
                            GameState::YearOfPlenty,
                            GameState::RobberPickColor,
                            GameState::Roll,
                            GameState::RobberDiscardResources,
                            GameState::RoadBuilding,
                            GameState::NotActive,
                            GameState::NotActiveSetup,
                            GameState::Start,
                            GameState::Nothing,
                        ]
                        .contains(&current_state),
                        None => true,
                    }
                }),
            )
            .add_systems(
                Update,
                development_cards::buy_development_card_interaction
                    .run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                common_ui::button_system_with_generic::<TownPlaceButton, PlaceTownButtonState<'_>>
                    .run_if(in_state(GameState::SetupTown).or(in_state(GameState::PlaceTown))),
            )
            .add_systems(
                GgrsSchedule,
                (
                    update_from_inputs,
                    update_from_inputs_roll,
                    update_from_monopoly,
                    update_from_knight,
                    update_from_trade_accept,
                )
                    .ambiguous_with_all(),
            )
            .add_systems(
                Update,
                common_ui::button_system_with_generic::<RoadPlaceButton, PlaceRoadButtonState<'_>>
                    .run_if(
                        in_state(GameState::SetupRoad)
                            .or(in_state(GameState::PlaceRoad)
                                .or(in_state(GameState::RoadBuilding))),
                    ),
            )
            .add_systems(
                Update,
                |road: Query<'_, '_, (&RoadPosition, &CatanColor), Added<Road>>,
                 mut meshes: ResMut<'_, Assets<Mesh>>,

                 mut materials: ResMut<'_, Assets<ColorMaterial>>,
                 mut commands: Commands<'_, '_>| {
                    for (road_position, catan_color) in road {
                        commands.spawn(RoadUI::bundle(
                            *road_position,
                            &mut meshes,
                            &mut materials,
                            *catan_color,
                        ));
                    }
                },
            )
            .add_systems(
                Update,
                (
                    common_ui::spinner_buttons_interactions::<
                        RobberResourceSpinner,
                        ResMut<'_, RobberDiscard>,
                    >(),
                    robber::counter_sumbit_interaction,
                    robber::counter_text_update,
                )
                    .run_if(
                        in_state(GameState::RobberDiscardResources)
                            .or(in_state(GameState::RobberDiscardResourcesInActive)),
                    ),
            )
            .add_systems(
                Update,
                (robber::done_discarding).run_if(in_state(GameState::RobberDiscardResources)),
            )
            .add_systems(
                Update,
                cities::place_normal_city_interaction.run_if(in_state(GameState::PlaceCity)),
            );
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum GameState {
    NotActive,
    NotActiveSetup,
    RobberDiscardResources,
    RobberDiscardResourcesInActive,
    #[default]
    Nothing,
    Start,
    PlaceRoad,
    Roll,
    Turn,
    PlaceTown,
    PlaceCity,
    SetupRoad,
    SetupTown,
    RoadBuilding, // (dev card)
    YearOfPlenty, // (dev card)
    Monopoly,
    // picking which color to pick from
    RobberPickColor,
    // picking which place to put robber on
    PlaceRobber,
}

// for players input with ggrs
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct PlayerHandle(pub usize);
#[derive(Component, PartialEq, Debug, Clone, Copy)]
enum Number {
    Number(u8),
    None,
}

#[derive(Debug, Component, Clone, Copy)]
// our hexagons are pointy
enum Hexagon {
    Wood = 0,
    Brick,
    Sheep,
    Wheat,
    Ore,
    Desert,
    Water,
    Port,
    Empty,
}
impl From<u8> for Hexagon {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Wood,
            1 => Self::Brick,
            2 => Self::Sheep,
            3 => Self::Wheat,
            4 => Self::Ore,
            5 => Self::Desert,
            6 => Self::Water,
            7 => Self::Port,
            8 => Self::Empty,
            _ => Self::Empty,
        }
    }
}
impl Hexagon {
    fn color(&self) -> Color {
        match self {
            Self::Wood => Color::srgb_u8(161, 102, 47),
            Self::Brick => Color::srgb_u8(198, 74, 60),
            Self::Sheep => Color::srgb_u8(0, 255, 0),
            Self::Wheat => Color::srgb_u8(255, 191, 0),
            Self::Ore => Color::srgb_u8(67, 67, 65),
            Self::Desert => Color::srgb_u8(194, 178, 128),
            Self::Water => Color::srgb_u8(0, 0, 255),
            Self::Port => Color::srgb_u8(0, 0, 255),
            Self::Empty => Color::BLACK.with_alpha(-1.),
        }
    }
}
#[derive(Debug, Component, Clone, Copy)]
enum Port {
    TwoForOne(resources::Resource),
    ThreeForOne,
}
impl Port {
    pub fn color(&self) -> Color {
        match self {
            Self::TwoForOne(resource) => resource.color(),
            Self::ThreeForOne => Color::srgb_u8(194, 178, 128),
        }
        .darker(0.02)
    }
}
#[derive(Resource, Clone, Copy)]
struct BoardSize(u8);

fn cleanup_button<T: Component>(
    mut commands: Commands<'_, '_>,
    mut interaction_query: Query<'_, '_, Entity, (With<T>, With<Button>)>,
) {
    for entity in &mut interaction_query {
        commands.entity(entity).despawn();
    }
}
fn cleanup_ui<T: Component>(
    mut commands: Commands<'_, '_>,
    mut interaction_query: Query<'_, '_, Entity, (With<T>, With<Node>)>,
) {
    for entity in &mut interaction_query {
        commands.entity(entity).despawn();
    }
}
pub trait UI {
    type Pos;
    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
    ) -> impl Bundle;
    fn resources() -> Resources;
}

// not for initial game setup where the are no roads yet
// TODO: maybe we should impose an order on postions for stuff like roads so that comparing them is
// easeier (i.e. first postion is smallest ....)

#[derive(Component, PartialEq, Eq, Debug, Copy, Clone)]
struct Left<T>(pub u8, PhantomData<T>);

// town city "enherit" from building make some quries easier
// i think right way to do it with is with `[require(..)]`
#[derive(Component, PartialEq, Default, Clone, Copy)]
struct Building;

#[derive(Component, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub struct VictoryPoints {
    pub actual: u8,
    pub from_development_cards: u8,
}
#[derive(Component, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub struct Knights(pub u8);
#[derive(Resource, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub struct PlayerCount(pub u8);
fn game_setup(
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut commands: Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
    player_count: Res<'_, PlayerCount>,
    seed: Res<'_, SessionSeed>,

    local_players: Res<'_, LocalPlayers>,
) {
    let layout = layout(&mut commands);
    commands.insert_resource(layout);
    let catan_colors = setup_game::setup(
        &mut commands,
        meshes,
        materials,
        layout,
        player_count,
        seed.0,
        local_players,
    );

    commands.insert_resource(ColorIterator(catan_colors.clone().cycle()));
    commands.insert_resource(SetupColorIterator(
        catan_colors.clone().chain(catan_colors.rev()),
    ));

    next_state.set(GameState::Start);
}

pub fn next_player(
    next_state: &mut ResMut<'_, NextState<GameState>>,
    local_players: &Res<'_, LocalPlayer>,
    new: CatanColorRef,
    active: GameState,
    inactive: GameState,
) {
    if local_players.0.handle == new.handle {
        println!("active {active:?}");
        next_state.set(active);
    } else {
        next_state.set(inactive);
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct Layout {
    pub player_banner: Entity,
    pub development_cards: Entity,
    pub setting_pull_out: Entity,
    pub resources: Entity,
    pub board: Entity,
    pub ui: Entity,
    pub trades: Entity,
}
fn layout(commands: &mut Commands<'_, '_>) -> Layout {
    let player_banner_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
            // children![Text("banner".to_string()),],
        ))
        .id();
    let settings_pull_out_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),

                ..default()
            },
            BorderColor::all(Color::BLACK),
            children![Text("settings".to_string()),],
        ))
        .id();
    let development_cards_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::percent(20.),
                    GridTrack::percent(20.),
                    GridTrack::percent(20.),
                    GridTrack::percent(20.),
                    GridTrack::percent(20.),
                ],
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
        ))
        .id();
    let resources_layout = commands
        .spawn((
            Node {
                display: Display::Grid,

                grid_template_rows: vec![GridTrack::percent(10.), GridTrack::percent(90.)],
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
        ))
        .id();
    let mut card_layout = commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_rows: vec![
                GridTrack::percent(60.),
                GridTrack::percent(20.),
                GridTrack::percent(20.),
            ],
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BorderColor::all(Color::BLACK),
    ));
    card_layout.add_children(&[
        settings_pull_out_layout,
        development_cards_layout,
        resources_layout,
    ]);
    let card_layout = card_layout.id();
    let board_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
        ))
        .id();
    let ui_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
        ))
        .id();
    let mut main_ui_layout = commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(85.), GridTrack::percent(15.)],
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BorderColor::all(Color::BLACK),
    ));
    main_ui_layout.add_children(&[board_layout, ui_layout]);
    let main_ui_layout = main_ui_layout.id();
    let trades_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                grid_auto_flow: GridAutoFlow::Row,
                align_content: AlignContent::Start,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            children![Text("trades".to_string()),],
            BorderColor::all(Color::BLACK),
        ))
        .id();
    let mut main_layout = commands.spawn((Node {
        display: Display::Grid,
        grid_template_columns: vec![
            GridTrack::percent(25.),
            GridTrack::percent(50.),
            GridTrack::percent(50.),
        ],
        ..default()
    },));

    main_layout.add_children(&[card_layout, main_ui_layout, trades_layout]);
    let main_layout = main_layout.id();
    let mut layout = commands.spawn((Node {
        display: Display::Grid,

        width: Val::Percent(100.),
        height: Val::Percent(100.),
        grid_template_rows: vec![GridTrack::percent(10.), GridTrack::percent(90.)],
        ..default()
    },));
    layout.add_children(&[player_banner_layout, main_layout]);
    Layout {
        player_banner: player_banner_layout,
        development_cards: development_cards_layout,
        resources: resources_layout,
        board: board_layout,
        ui: ui_layout,
        trades: trades_layout,
        setting_pull_out: settings_pull_out_layout,
    }
}
fn handle_ggrs_events(mut session: ResMut<'_, Session<GgrsSessionConfig>>) {
    if let Session::P2P(s) = session.as_mut() {
        for event in s.events() {
            match event {
                GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                    // warn!("GGRS event: {event:?}")
                }
                GgrsEvent::DesyncDetected {
                    local_checksum,
                    remote_checksum,
                    frame,
                    ..
                } => {
                    error!(
                        "Desync on frame {frame}. Local checksum: {local_checksum:X}, remote checksum: {remote_checksum:X}"
                    );
                }
                _ => info!("GGRS event: {event:?}"),
            }
        }
    }
}
