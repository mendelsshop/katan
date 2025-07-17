#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

mod colors;
mod development_cards;
mod dice;
mod positions;
mod resources;
mod turn_ui;
use std::mem::swap;

use bevy::{ecs::query::QueryData, prelude::*};
use itertools::Itertools;
use rand::seq::SliceRandom;

use crate::{
    colors::{CatanColor, ColorIterator, CurrentColor, CurrentSetupColor, SetupColorIterator},
    positions::{BuildingPosition, Coordinate, FPosition, Position, RoadPosition},
    resources::{
        CITY_RESOURCES, DEVELOPMENT_CARD_RESOURCES, ROAD_RESOURCES, Resources, TOWN_RESOURCES,
    },
    turn_ui::DevelopmentCardButton,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.init_state::<GameState>();
    app.insert_resource(BoardSize(3));
    app.init_resource::<Robber>();
    app.insert_resource(Resources::new_game());
    // TODO: is there way to init resource
    // without giving a value
    app.insert_resource(CurrentColor(CatanColor::White));
    app.insert_resource(CurrentSetupColor(CatanColor::White));
    app.add_systems(Startup, setup);

    app.add_systems(OnEnter(GameState::SetupRoad), place_setup_road);
    app.add_systems(OnEnter(GameState::SetupTown), place_setup_town);

    app.add_systems(OnExit(GameState::SetupRoad), cleanup::<RoadPosition>);
    app.add_systems(OnExit(GameState::SetupTown), cleanup::<BuildingPosition>);
    app.add_systems(OnExit(GameState::PlaceRoad), cleanup::<RoadPosition>);
    app.add_systems(OnExit(GameState::PlaceTown), cleanup::<BuildingPosition>);
    app.add_systems(OnExit(GameState::PlaceCity), cleanup::<BuildingPosition>);

    app.add_systems(OnEnter(GameState::PlaceRoad), place_normal_road);
    app.add_systems(OnEnter(GameState::PlaceTown), place_normal_town);
    app.add_systems(OnEnter(GameState::PlaceCity), place_normal_city);
    app.add_systems(
        OnTransition {
            // you might think, that we would do this after the last town (with SetupTown), but due
            // to how the color/player changing logic for setup its not acutally so
            exited: GameState::SetupRoad,
            entered: GameState::Roll,
        },
        turn_ui::show_turn_ui,
    );

    app.add_systems(
        Update,
        turn_ui::turn_ui_road_interaction.run_if(in_state(GameState::Turn)),
    );

    app.add_systems(
        Update,
        turn_ui::turn_ui_town_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui::turn_ui_city_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui::turn_ui_roll_interaction.run_if(in_state(GameState::Roll)),
    );

    app.add_systems(
        Update,
        // TODO: if in turn or place state
        turn_ui::turn_ui_next_interaction,
    );
    app.add_systems(OnEnter(GameState::SetupRoad), colors::set_setup_color);
    app.add_systems(OnEnter(GameState::Roll), colors::set_color);
    app.add_systems(
        Update,
        place_normal_interaction::<Road, RoadPosition, RoadUI, CurrentSetupColor>
            .run_if(in_state(GameState::SetupRoad)),
    );
    app.add_systems(
        Update,
        buy_development_card_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Town, BuildingPosition, TownUI, CurrentSetupColor>
            .run_if(in_state(GameState::SetupTown)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Road, RoadPosition, RoadUI, CurrentColor>
            .run_if(in_state(GameState::PlaceRoad)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Town, BuildingPosition, TownUI, CurrentColor>
            .run_if(in_state(GameState::PlaceTown)),
    );

    app.add_systems(
        Update,
        place_normal_city_interaction.run_if(in_state(GameState::PlaceCity)),
    );
    app.run();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
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
}
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

const fn place_robber() {
    // show ui to place robber
    // on every hex besides for current robber hex
    // the make interaction function, that when clicked:
    // 1) moves the robber there, set the robber postion
    // 2) tries to take a resource from other player, or show ui to choose which player to pick
    //    from
}
const fn place_robber_interaction() {}
const fn choose_player_to_take_from() {}
const fn choose_player_to_take_from_interaction() {}

#[derive(Debug, Component, Clone, Copy, Default)]
#[require(Building)]
struct Town;
#[derive(Debug, Component, Clone, Copy)]
#[require(Building)]
struct City;
#[derive(Debug, Component, Clone, Copy, Default)]
struct Road;
#[derive(Debug, Component, Clone, Copy)]
enum DevelopmentCard {
    Knight,
    Monopoly,
    YearOfPlenty,
    RoadBuilding,
    VictoryPoint,
}
fn generate_development_cards(commands: &mut Commands<'_, '_>) {
    let mut development_cards = [
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::Knight,
        DevelopmentCard::VictoryPoint,
        DevelopmentCard::VictoryPoint,
        DevelopmentCard::VictoryPoint,
        DevelopmentCard::VictoryPoint,
        DevelopmentCard::VictoryPoint,
        DevelopmentCard::RoadBuilding,
        DevelopmentCard::RoadBuilding,
        DevelopmentCard::Monopoly,
        DevelopmentCard::Monopoly,
        DevelopmentCard::YearOfPlenty,
        DevelopmentCard::YearOfPlenty,
    ];
    development_cards.shuffle(&mut rand::rng());

    for card in development_cards {
        commands.spawn(card);
    }
}
fn generate_board(commands: &mut Commands<'_, '_>) -> Vec<(Position, Hexagon, Number)> {
    let mut numbers = [
        (Number::Number(2)),
        (Number::Number(3)),
        (Number::Number(3)),
        (Number::Number(4)),
        (Number::Number(4)),
        (Number::Number(5)),
        (Number::Number(5)),
        (Number::Number(6)),
        (Number::Number(6)),
        (Number::Number(8)),
        (Number::Number(8)),
        (Number::Number(9)),
        (Number::Number(9)),
        (Number::Number(10)),
        (Number::Number(10)),
        (Number::Number(11)),
        (Number::Number(11)),
        (Number::Number(12)),
    ];
    numbers.shuffle(&mut rand::rng());
    let inhabited_hexagons = [
        Hexagon::Wheat,
        Hexagon::Wheat,
        Hexagon::Wheat,
        Hexagon::Wheat,
        Hexagon::Wood,
        Hexagon::Wood,
        Hexagon::Wood,
        Hexagon::Wood,
        Hexagon::Sheep,
        Hexagon::Sheep,
        Hexagon::Sheep,
        Hexagon::Sheep,
        Hexagon::Ore,
        Hexagon::Ore,
        Hexagon::Ore,
        Hexagon::Brick,
        Hexagon::Brick,
        Hexagon::Brick,
    ];
    let mut inhabited = inhabited_hexagons
        .into_iter()
        .zip(numbers)
        .chain([(Hexagon::Desert, Number::None); 1])
        .collect_vec();

    // 1 for first layer 6 for second layer 12 for third layer

    inhabited.shuffle(&mut rand::rng());
    let (inhabited, mut desert): (Vec<_>, Vec<_>) = generate_postions(3)
        .zip(inhabited)
        .map(|(position, (hex, number))| (position, hex, number))
        .partition(|p| p.2 != Number::None);
    let (reds, normal_number): (Vec<_>, Vec<_>) = inhabited
        .into_iter()
        .partition(|(_, _, n)| Number::Number(8) == *n || Number::Number(6) == *n);
    let mut inhabited = fix_numbers(reds, normal_number);
    if let Some(desert) = desert.first() {
        commands.insert_resource(Robber(desert.0));
    }
    inhabited.append(&mut desert);
    inhabited.extend(generate_postions_ring(3).map(|p| (p, Hexagon::Empty, Number::None)));
    for hex in &inhabited {
        commands.spawn((hex.0, hex.1, hex.2));
    }
    inhabited
}

fn fix_numbers(
    mut reds: Vec<(Position, Hexagon, Number)>,
    mut normal: Vec<(Position, Hexagon, Number)>,
) -> Vec<(Position, Hexagon, Number)> {
    let cube_direction_vectors = [
        Position { q: 1, r: 0, s: -1 },
        Position { q: 1, r: -1, s: 0 },
        Position { q: 0, r: -1, s: 1 },
        Position { q: -1, r: 0, s: 1 },
        Position { q: -1, r: 1, s: 0 },
        Position { q: 0, r: 1, s: -1 },
    ];
    let mut used = vec![];

    while let Some(red @ (p, _, _)) = reds.pop() {
        let touches = |p1| cube_direction_vectors.map(|p1| p + p1).contains(&p1);
        used.push(red);
        let mut new_used;
        (new_used, normal) = normal.into_iter().partition(|p| touches(p.0));
        used.append(&mut new_used);
        reds.iter_mut().filter(|p| touches(p.0)).for_each(|red| {
            let mut new_hexagon =
                normal.swap_remove((rand::random::<u8>() % normal.len() as u8) as usize);

            swap(&mut red.1, &mut new_hexagon.1);
            swap(&mut red.0, &mut new_hexagon.0);
            used.push(new_hexagon);
        });
    }

    used.append(&mut normal);
    used
}

fn generate_postions_ring(n: i8) -> impl Iterator<Item = Position> {
    let has_big_coordinate: _ = move |i: i8| i == -n || i == n;
    generate_postions(n + 1).filter(move |q| {
        has_big_coordinate(q.q) || has_big_coordinate(q.r) || has_big_coordinate(q.s)
    })
}
fn generate_postions(n: i8) -> impl Iterator<Item = Position> {
    (0..3)
        .map(|_| -n + 1..n)
        .multi_cartesian_product()
        .filter(|q| q[0] + q[1] + q[2] == 0)
        .map(|i| Position {
            q: i[0],
            r: i[1],
            s: i[2],
        })
}
#[derive(Debug)]
enum Port {}
#[derive(Resource, Clone, Copy)]
struct BoardSize(u8);

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn cleanup<T: Component>(
    mut commands: Commands<'_, '_>,
    mut interaction_query: Query<'_, '_, Entity, (With<T>, With<Button>)>,
) {
    for entity in &mut interaction_query {
        commands.entity(entity).despawn();
    }
}

fn place_normal_city_interaction(
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
fn buy_development_card_interaction(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    mut free_dev_cards: Query<'_, '_, (Entity, &DevelopmentCard), Without<CatanColor>>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    mut resources: ResMut<'_, Resources>,
    interaction_query: Query<
        '_,
        '_,
        (
            &DevelopmentCardButton,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
        ),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, mut color, mut button) in interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();
                if let Some(card) = free_dev_cards.iter_mut().next() {
                    commands.entity(card.0).insert(color_r.0);
                }

                let required_resources = DEVELOPMENT_CARD_RESOURCES;
                let player_resources = player_resources.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= required_resources;
                }
                *resources += required_resources;

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
struct RoadUI;
impl UI for RoadUI {
    type Pos = RoadPosition;

    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
    ) -> impl Bundle {
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(Rectangle::new(7.0, 20.));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(color.to_bevy_color())),
            Transform::from_xyz(x * 28.0, y * 28., 0.0).with_rotation(Quat::from_rotation_z(
                match pos.shared_coordinate() {
                    Coordinate::R => 0f32,
                    Coordinate::Q => -60f32,
                    Coordinate::S => 60f32,
                }
                .to_radians(),
            )),
        )
    }

    fn resources() -> Resources {
        ROAD_RESOURCES
    }
}
struct TownUI;
impl UI for TownUI {
    type Pos = BuildingPosition;

    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
    ) -> impl Bundle {
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(RegularPolygon::new(7.0, 3));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(color.to_bevy_color())),
            Transform::from_xyz(x * 28.0, y * 28., 0.0),
        )
    }

    fn resources() -> Resources {
        TOWN_RESOURCES
    }
}
// should interaction be doing the ui update for showing the roads/towns
fn place_normal_interaction<
    Kind: Component + Default + std::fmt::Debug,
    Pos: Component + Copy,
    U: UI<Pos = Pos>,
    // TODO: unify the different types of color for setup and during the game
    // one way would be to make a color enum that has variant for setup and one for the rest of the game
    // another way would be make the type of color be a marker struct that `#[requires(CatanColor)]`
    // and then we could just look for CatanColor, and when we need the more specific one we specify
    // via the marker struct
    C: Into<CatanColor> + Resource + Copy,
>(
    mut resources: ResMut<'_, Resources>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    game_state: Res<'_, State<GameState>>,
    mut game_state_mut: ResMut<'_, NextState<GameState>>,
    color_r: Res<'_, C>,
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut kind_free_q: Query<'_, '_, (&Kind, &CatanColor, &mut Left)>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Pos,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
            &Resources,
        ),
        (Changed<Interaction>, Without<CatanColor>),
    >,
) {
    let current_color: CatanColor = (*color_r.into_inner()).into();
    for (entity, interaction, mut color, mut button, required_resources) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();

                commands.spawn((Kind::default(), current_color, *entity));
                let kind_left = kind_free_q.iter_mut().find(|x| x.1 == &current_color);
                if let Some((_, _, mut left)) = kind_left {
                    *left = Left(left.0 - 1);
                }
                let player_resources = player_resources.iter_mut().find(|x| x.1 == &current_color);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= *required_resources;
                }
                *resources += *required_resources;
                match *game_state.get() {
                    GameState::Nothing | GameState::Start | GameState::Roll | GameState::Turn => {}
                    GameState::PlaceRoad | GameState::PlaceTown | GameState::PlaceCity => {
                        game_state_mut.set(GameState::Turn);
                    }
                    GameState::SetupRoad => game_state_mut.set(GameState::SetupTown),
                    GameState::SetupTown => game_state_mut.set(GameState::SetupRoad),
                }
                commands.spawn(U::bundle(
                    *entity,
                    &mut meshes,
                    &mut materials,
                    current_color,
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

fn get_setup_road_placements(
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,
) -> impl Iterator<Item = RoadPosition> {
    // generate all road possobilties

    // generate the ring around it for edge roads
    generate_postions(4)
        .array_combinations::<2>()
        .filter_map(move |[p1, p2]| RoadPosition::new(p1, p2, Some(size_r.0)))
        // filter out ones that are already placed
        .filter(move |road| !road_q.iter().map(|r| r.2).contains(road))
}
fn place_setup_road(
    mut commands: Commands<'_, '_>,
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    // TODO: only show road if town can placed near it
    let count = get_setup_road_placements(size_r, road_q)
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
                    Resources::default(),
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
// not for initial game setup where the are no roads yet
// TODO: maybe we should impose an order on postions for stuff like roads so that comparing them is
// easeier (i.e. first postion is smallest ....)
fn place_normal_road(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    road_free_q: Query<'_, '_, (&Road, &CatanColor, &Left)>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_roads_correct_color = road_free_q.iter().find(|r| r.1 == &color_r.0);

    // no roads to place
    let Some(_) = unplaced_roads_correct_color.filter(|r| r.2.0 > 0) else {
        return;
    };

    let (current_color_roads, _): (Vec<_>, Vec<_>) =
        road_q.into_iter().partition(|r| *r.1 == color_r.0);

    // we don't check current color roads is empty b/c by iterating over them we are essentially
    // doing that already
    // roads are between two hexes (if one coordiante is the same
    // if q same then its flat (assuming hex is flat)
    // if r is same then its diagonol like '\'
    // if s is same then its diagonol like '/'
    // if there is a new place to put road down
    // 1) the new hex has to share one coordianate with one hex and another differenet one with the
    //    other hex (more constraint (i.e cannot be 50 square of in another direction)
    let possibles_roads = current_color_roads
        .into_iter()
        .flat_map(|RoadQueryItem(_, _, road)| {
            match road {
                RoadPosition::Both(p1, p2, _) => {
                    let (p3, p4) = road.neighboring_two(Some(size_r.0));
                    let make_road_pos = |p, option_p1: Option<_>, p2: &Position| {
                        option_p1.and_then(|p1| {
                            RoadPosition::new(p, p1, Some(size_r.0)).map(|r| (*p2, r))
                        })
                    };
                    [
                        (
                            // the other point (used to check for towns/cities)
                            // the postion of the road
                            make_road_pos(*p2, p3, p1)
                        ),
                        (make_road_pos(*p2, p4, p1)),
                        (make_road_pos(*p1, p3, p2)),
                        (make_road_pos(*p1, p4, p2)),
                    ]
                    .into_iter()
                    .flatten()
                }
            }
        });

    // 2) make sure that there is no road already there (whether that color or not)
    let possible_roads =
        possibles_roads.filter(|(_, r)| !road_q.iter().any(|RoadQueryItem(_, _, r1)| r == r1));

    // 3) make sure there is no differeent color town at the three itersection
    // partition into other color used towns with single partiton
    fn filter_by_building<'a>(
        (road1, road2): &(Position, RoadPosition),
        mut building_q: impl Iterator<Item = &'a BuildingPosition>,
    ) -> bool {
        let road_intersection = match road2 {
            RoadPosition::Both(position, position1, _) => {
                BuildingPosition::All(*road1, *position, *position1)
            }
        };
        !building_q.any(|bp| &road_intersection == bp)
    }
    let possible_roads = possible_roads.filter(|r| {
        filter_by_building(
            r,
            building_q
                .iter()
                .filter_map(|(_, color, pos)| Some(pos).filter(|_| *color != color_r.0)),
        )
    });

    let count = possible_roads
        .filter_map(|p| {
            let (x, y) = p.1.positon_to_pixel_coordinates();
            (x != 0. || y != 0.).then_some((x, y, p.1))
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
                    RoadUI::resources(),
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

fn place_normal_city(
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
fn place_setup_town(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentSetupColor>,
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &'_ CatanColor, &'_ BuildingPosition)>,
) {
    let possible_towns =
        get_possible_town_placements(color_r.0, BoardSize(size_r.0), road_q, building_q);
    possible_towns
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
                    Resources::default(),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )],
            )
        })
        .for_each(|b| {
            commands.spawn(b);
        });
}
fn place_normal_town(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    town_free_q: Query<'_, '_, (&Town, &CatanColor, &Left)>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_towns_correct_color = town_free_q.iter().find(|r| r.1 == &color_r.0);

    // no towns to place
    let Some(_) = unplaced_towns_correct_color.filter(|r| r.2.0 > 0) else {
        return;
    };

    let possible_towns =
        get_possible_town_placements(color_r.0, BoardSize(size_r.0), road_q, building_q);
    let count = possible_towns
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
                    TownUI::resources(),
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

fn get_possible_town_placements(
    color_r: CatanColor,
    size_r: BoardSize,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&Building, &CatanColor, &BuildingPosition)>,
) -> impl Iterator<Item = BuildingPosition> {
    let (current_color_roads, _): (Vec<_>, Vec<_>) =
        road_q.into_iter().partition(|r| *r.1 == color_r);

    let possibles_towns = buildings_on_roads(
        current_color_roads
            .into_iter()
            .map(|RoadQueryItem(_, _, road)| *road),
        BoardSize(size_r.0),
    );

    let filter_by_building =
        move |position: &BuildingPosition,
              building_q: Query<'_, '_, (&_, &CatanColor, &BuildingPosition)>| {
            match position {
                BuildingPosition::All(position, position1, position2) => !buildings_on_roads(
                    [
                        RoadPosition::new(*position, *position1, Some(size_r.0)),
                        RoadPosition::new(*position, *position2, Some(size_r.0)),
                        RoadPosition::new(*position1, *position2, Some(size_r.0)),
                    ]
                    .into_iter()
                    .flatten(),
                    BoardSize(size_r.0),
                )
                .any(|p| building_q.iter().any(|(_, _, place_b)| &p == place_b)),
            }
        };

    possibles_towns.filter(move |r| filter_by_building(r, building_q))
}

fn buildings_on_roads(
    current_color_roads: impl Iterator<Item = RoadPosition>,
    size_r: BoardSize,
) -> impl Iterator<Item = BuildingPosition> {
    current_color_roads.flat_map(move |road| match road {
        RoadPosition::Both(p1, p2, _) => {
            let (p3, p4) = road.neighboring_two(Some(size_r.0));
            let make_town_pos = |p, option_p1: Option<_>, p2| {
                option_p1.and_then(|p1| BuildingPosition::new(p, p1, p2, Some(size_r.0)))
            };
            [(make_town_pos(p1, p3, p2)), (make_town_pos(p1, p4, p2))]
                .into_iter()
                .flatten()
        }
    })
}
fn draw_board(
    q: impl Iterator<Item = (Position, Hexagon, Number)>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    commands: &mut Commands<'_, '_>,
) {
    let text_justification = JustifyText::Center;
    for q in q {
        let mesh = meshes.add(RegularPolygon::new(25.0, 6));
        let mesh1 = meshes.add(Circle::new(13.0));
        let (x, y) = FPosition::hex_to_pixel(q.0.into());
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(q.1.color())),
            Transform::from_xyz(x * 28.0, y * 28., 0.0),
        ));

        if let Number::Number(n) = q.2 {
            let mesh2 = Text2d::new(n.to_string());
            commands.spawn((
                Mesh2d(mesh1),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(x * 28.0, y * 28., 0.0),
            ));
            commands.spawn((
                mesh2,
                TextLayout::new_with_justify(text_justification),
                Transform::from_xyz(x * 28.0, y * 28., 0.0),
            ));
        }
    }
}
#[derive(Component, PartialEq, Eq, Debug)]
struct Left(pub u8);

// town city "enherit" from building make some quries easier
// i think right way to do it with is with `[require(..)]`
#[derive(Component, PartialEq, Default, Clone, Copy)]
struct Building;

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Robber(Position);
impl Default for Robber {
    fn default() -> Self {
        Self(Position { q: 0, r: 0, s: 0 })
    }
}
fn generate_pieces(commands: &mut Commands<'_, '_>) {
    for color in [
        CatanColor::Red,
        CatanColor::Blue,
        CatanColor::Green,
        CatanColor::White,
    ] {
        commands.spawn((Town, color, Left(5)));
        commands.spawn((City, color, Left(4)));
        commands.spawn((Road, color, Left(15)));
        commands.spawn((Resources::new_player(), color));
    }
}

#[derive(QueryData, Debug, Clone, Copy)]

pub struct RoadQuery(&'static Road, &'static CatanColor, &'static RoadPosition);
fn setup(
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut commands: Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
    asset_server: Res<'_, AssetServer>,
) {
    commands.spawn(Camera2d);
    draw_board(
        generate_board(&mut commands).into_iter(),
        materials,
        meshes,
        &mut commands,
    );
    generate_development_cards(&mut commands);
    generate_pieces(&mut commands);
    next_state.set(GameState::SetupRoad);

    // this has to be set dynamically
    commands.insert_resource(ColorIterator(
        vec![
            CatanColor::White,
            CatanColor::Red,
            CatanColor::Blue,
            CatanColor::Green,
        ]
        .into_iter()
        .cycle(),
    ));
    commands.insert_resource(SetupColorIterator(
        vec![
            CatanColor::White,
            CatanColor::Red,
            CatanColor::Blue,
            CatanColor::Green,
        ]
        .into_iter()
        .chain(
            vec![
                CatanColor::White,
                CatanColor::Red,
                CatanColor::Blue,
                CatanColor::Green,
            ]
            .into_iter()
            .rev(),
        ),
    ));
}
