#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use std::{
    mem::swap,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use bevy::{ecs::query::QueryData, prelude::*};

use itertools::Itertools;
use rand::seq::SliceRandom;

fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.init_state::<GameState>();
    app.insert_resource(BoardSize(3));
    app.init_resource::<Robber>();
    app.insert_resource(CurrentColor(CatanColor::White));
    app.insert_resource(Resources::new_game());
    app.add_systems(Startup, setup);
    app.add_systems(OnEnter(GameState::PlaceRoad), (place_normal_road,));
    app.add_systems(OnEnter(GameState::PlaceTown), (place_normal_town,));
    app.add_systems(OnEnter(GameState::PlaceCity), (place_normal_city,));
    app.add_systems(OnEnter(GameState::Turn), (show_turn_ui,));

    app.add_systems(
        Update,
        turn_ui_road_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui_town_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui_city_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui_roll_interaction.run_if(in_state(GameState::Roll)),
    );

    app.add_systems(OnExit(GameState::PlaceRoad), (cleanup::<RoadPostion>,));
    app.add_systems(OnExit(GameState::PlaceTown), (cleanup::<BuildingPosition>,));
    app.add_systems(OnExit(GameState::PlaceCity), (cleanup::<BuildingPosition>,));
    app.add_systems(
        Update,
        place_normal_interaction::<Road, RoadPostion, RoadUI>
            .run_if(in_state(GameState::PlaceRoad)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Town, BuildingPosition, TownUI>
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
}
#[derive(Component, PartialEq, Debug, Clone, Copy)]
enum Number {
    Number(u8),
    None,
}
#[derive(Component, Debug, PartialEq, Clone, Copy)]
struct Position {
    q: i8,
    r: i8,
    s: i8,
}
#[derive(Debug, PartialEq, Clone, Copy)]
struct FPosition {
    q: f32,
    r: f32,
    s: f32,
}
impl From<Position> for FPosition {
    fn from(Position { q, r, s }: Position) -> Self {
        Self {
            q: f32::from(q),
            r: f32::from(r),
            s: f32::from(s),
        }
    }
}
impl FPosition {
    const fn filter_coordinate(mut self, coordinate: Coordinate) -> Self {
        match coordinate {
            Coordinate::Q => self.q = 0.,
            Coordinate::R => self.r = 0.,
            Coordinate::S => self.s = 0.,
        }
        self
    }
    const fn get_shared_coordinate(&self, other: &Self) -> Option<Coordinate> {
        if self.q == other.q {
            Some(Coordinate::Q)
        } else if self.r == other.r {
            Some(Coordinate::R)
        } else if self.s == other.s {
            Some(Coordinate::S)
        } else {
            None
        }
    }
    pub fn intersect(self, other: Self) -> Option<Self> {
        self.get_shared_coordinate(&other)
            .map(|shared_coordinate| self.interesect_with_coordinate(other, shared_coordinate))
    }

    const fn interesect_with_coordinate(
        self,
        Self {
            q: q1,
            r: r1,
            s: s1,
        }: Self,
        shared_coordinate: Coordinate,
    ) -> Self {
        let Self { q, r, s } = self;
        match shared_coordinate {
            Coordinate::Q => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    q,
                    r: f32::midpoint(r, r1),
                    s: f32::midpoint(s, s1),
                }
            }
            Coordinate::R => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    r,
                    q: f32::midpoint(q, q1),
                    s: f32::midpoint(s, s1),
                }
            }
            Coordinate::S => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    s,
                    r: f32::midpoint(r, r1),
                    q: f32::midpoint(q, q1),
                }
            }
        }
    }
    fn hex_to_pixel(self) -> (f32, f32) {
        let x = 3f32.sqrt().mul_add(self.q, 3f32.sqrt() / 2. * self.r);
        let y = 3. / 2. * self.r;
        (x, y)
    }
}
// maybe do size const generics?
impl Position {
    const DIRECTION_VECTORS: [Self; 6] = [
        Self { q: 1, r: 0, s: -1 },
        Self { q: 1, r: -1, s: 0 },
        Self { q: 0, r: -1, s: 1 },
        Self { q: -1, r: 0, s: 1 },
        Self { q: -1, r: 1, s: 0 },
        Self { q: 0, r: 1, s: -1 },
    ];
    fn rotate_right(&self) -> Self {
        let Self { q, r, s } = self;
        Self {
            q: -r,
            r: -s,
            s: -q,
        }
    }
    fn building_positions_around(&self) -> [BuildingPosition; 6] {
        Self::DIRECTION_VECTORS.map(|p| {
            let p1 = p.rotate_right();
            BuildingPosition::All(*self, p + *self, p1 + *self)
        })
    }
    fn all_points_are(&self, mut f: impl FnMut(i8) -> bool) -> bool {
        f(self.q) && f(self.r) && f(self.s)
    }
    fn any_points_is(&self, mut f: impl FnMut(i8) -> bool) -> bool {
        f(self.q) || f(self.r) || f(self.s)
    }
    const fn get_shared_coordinate(&self, other: &Self) -> Option<Coordinate> {
        if self.q == other.q {
            Some(Coordinate::Q)
        } else if self.r == other.r {
            Some(Coordinate::R)
        } else if self.s == other.s {
            Some(Coordinate::S)
        } else {
            None
        }
    }

    // TODO: maybe this should be a result as their are two possiblities for failure
    // 1) it doesn't add uo to 0
    // 2) its out of the board
    pub fn new(q: i8, r: i8, s: i8, size: Option<u8>) -> Option<Self> {
        const fn in_between(bound: u8, point: i8) -> bool {
            let bound = (bound) as i8;
            -bound <= point && point <= bound
        }
        (q + r + s == 0
            && size.is_none_or(|size| {
                in_between(size, q) && in_between(size, r) && in_between(size, s)
            }))
        .then_some(Self { q, r, s })
    }
}
impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}
impl Div<f32> for FPosition {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            q: self.q / rhs,
            r: self.r / rhs,
            s: self.s / rhs,
        }
    }
}
impl Add for FPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
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
#[derive(Debug, Resource, Clone, Copy)]
// TODO: what about before turn order decided
struct CurrentColor(CatanColor);
#[derive(Debug, Component, Clone, Copy, PartialEq)]
enum CatanColor {
    Red,
    Green,
    Blue,
    White,
}

#[derive(Debug, Component, Resource, Clone, Copy)]
pub struct Resources {
    wood: u8,
    brick: u8,
    sheep: u8,
    wheat: u8,
    ore: u8,
}
impl Sub for Resources {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            wood: self.wood - rhs.wood,
            brick: self.brick - rhs.brick,
            sheep: self.sheep - rhs.sheep,
            wheat: self.wheat - rhs.wheat,
            ore: self.ore - rhs.ore,
        }
    }
}
impl Mul<u8> for Resources {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        Self {
            wood: self.wood * rhs,
            brick: self.brick * rhs,
            sheep: self.sheep * rhs,
            wheat: self.wheat * rhs,
            ore: self.ore * rhs,
        }
    }
}
impl SubAssign for Resources {
    fn sub_assign(&mut self, rhs: Self) {
        self.wood -= rhs.wood;
        self.brick -= rhs.brick;
        self.sheep -= rhs.sheep;
        self.wheat -= rhs.wheat;
        self.ore -= rhs.ore;
    }
}
impl AddAssign for Resources {
    fn add_assign(&mut self, rhs: Self) {
        self.wood += rhs.wood;
        self.brick += rhs.brick;
        self.sheep += rhs.sheep;
        self.wheat += rhs.wheat;
        self.ore += rhs.ore;
    }
}
impl Add for Resources {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            wood: self.wood + rhs.wood,
            brick: self.brick + rhs.brick,
            sheep: self.sheep + rhs.sheep,
            wheat: self.wheat + rhs.wheat,
            ore: self.ore + rhs.ore,
        }
    }
}

impl Resources {
    #[must_use]
    pub const fn contains(self, rhs: Self) -> bool {
        self.wood >= rhs.wood
            && self.brick >= rhs.brick
            && self.sheep >= rhs.sheep
            && self.wheat >= rhs.wheat
            && self.ore >= rhs.ore
    }
    #[must_use]
    pub const fn new_player() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }
    #[must_use]
    pub const fn new_game() -> Self {
        Self::new(19, 19, 19, 19, 19)
    }
    #[must_use]
    pub const fn new(wood: u8, brick: u8, sheep: u8, wheat: u8, ore: u8) -> Self {
        Self {
            wood,
            brick,
            sheep,
            wheat,
            ore,
        }
    }
}
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
fn generate_bord(commands: &mut Commands<'_, '_>) -> Vec<(Position, Hexagon, Number)> {
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

#[derive(Component, PartialEq, Debug, Clone, Copy)]
// button in game to start road placement ui
struct RoadButton;
#[derive(Component, PartialEq, Debug, Clone, Copy)]
// button in game to start town placement ui
struct TownButton;
#[derive(Component, PartialEq, Debug, Clone, Copy)]
// button in game to start city placement ui
struct CityButton;
// TODO: combine with turn_ui_road_interaction
fn turn_ui_city_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (&CityButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, (&Resources, &CatanColor)>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.iter().find(|x| x.1 == &color_r.0);
    if let Some((resources, _)) = player_resources {
        for (entity, interaction, mut button) in &mut interaction_query {
            if resources.contains(TownUI::resources()) {
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
}
fn turn_ui_town_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (&TownButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, (&Resources, &CatanColor)>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.iter().find(|x| x.1 == &color_r.0);
    if let Some((resources, _)) = player_resources {
        for (entity, interaction, mut button) in &mut interaction_query {
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
}
fn turn_ui_road_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (&RoadButton, &Interaction, &mut Button),
        Changed<Interaction>,
    >,
    player_resources: Query<'_, '_, (&Resources, &CatanColor)>,
    color_r: Res<'_, CurrentColor>,
) {
    let player_resources = player_resources.iter().find(|x| x.1 == &color_r.0);
    if let Some((resources, _)) = player_resources {
        for (entity, interaction, mut button) in &mut interaction_query {
            if resources.contains(TownUI::resources()) {
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
}
fn show_turn_ui(mut commands: Commands<'_, '_>, asset_server: Res<'_, AssetServer>) {
    let road_icon = asset_server.load("road.png");
    let town_icon: Handle<Image> = asset_server.load("house.png");
    let city_icon = asset_server.load("city.png");
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
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
                    left: Val::Px(15.),
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
                    left: Val::Px(25.),
                    width: Val::Px(37.306),
                    height: Val::Px(25.0),
                    ..default()
                },
                Button,
                ImageNode::new(city_icon),
                CityButton,
            )
        ],
    ));
}
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
        ),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, mut color, mut button) in &mut interaction_query {
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
                let city_resources = Resources {
                    wood: 0,
                    brick: 0,
                    sheep: 0,
                    wheat: 2,
                    ore: 3,
                };
                let player_resources = player_resources.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= city_resources;
                }
                *resources += city_resources;

                game_state.set(GameState::Turn);
                let (x, y) = entity.positon_to_pixel_coordinates();

                let mesh1 = meshes.add(Rectangle::new(13.0, 13.));
                commands.spawn((
                    Mesh2d(mesh1),
                    MeshMaterial2d(materials.add(Color::BLACK)),
                    Transform::from_xyz(x * 28.0, -y * 28., 0.0),
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
pub trait UI {
    type Pos;
    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle;
    fn resources() -> Resources;
}
struct RoadUI;
impl UI for RoadUI {
    type Pos = RoadPostion;

    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(Rectangle::new(7.0, 20.));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform::from_xyz(x * 28.0, -y * 28., 0.0).with_rotation(Quat::from_rotation_z(
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
        Resources {
            wood: 1,
            brick: 1,
            sheep: 0,
            wheat: 0,
            ore: 0,
        }
    }
}
struct TownUI;
impl UI for TownUI {
    type Pos = BuildingPosition;

    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(RegularPolygon::new(7.0, 3));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(Color::BLACK)),
            Transform::from_xyz(x * 28.0, -y * 28., 0.0),
        )
    }

    fn resources() -> Resources {
        Resources {
            wood: 1,
            brick: 1,
            sheep: 1,
            wheat: 1,
            ore: 0,
        }
    }
}
// should interaction be doing the ui update for showing the roads/towns
fn place_normal_interaction<Kind: Component + Default, Pos: Component + Copy, U: UI<Pos = Pos>>(
    mut resources: ResMut<'_, Resources>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
    color_r: Res<'_, CurrentColor>,
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut kind_free_q: Query<'_, '_, (&Kind, &CatanColor, &mut Left)>,
    mut interaction_query: Query<
        '_,
        '_,
        (&Pos, &Interaction, &mut BackgroundColor, &mut Button),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, mut color, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();

                commands.spawn((Kind::default(), color_r.0, *entity));
                let kind_left = kind_free_q.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((_, _, mut left)) = kind_left {
                    *left = Left(left.0 - 1);
                }
                let player_resources = player_resources.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= U::resources();
                }
                *resources += U::resources();
                game_state.set(GameState::Turn);
                commands.spawn(U::bundle(*entity, &mut meshes, &mut materials));
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
fn turn_ui_roll_interaction(
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
    mut die_q: Query<'_, '_, &mut Text, With<DieButton>>,
) {
    for (entity, interaction, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();

                game_state.set(GameState::Turn);
                full_roll_dice(
                    &board,
                    &towns,
                    &cities,
                    &mut player_resources,
                    &mut resources,
                    &robber,
                    &mut die_q,
                );
                button.set_changed();
            }
            Interaction::Hovered => {
                button.set_changed();
            }
            Interaction::None => {}
        }
    }
}
#[derive(Component, PartialEq, Default, Clone, Copy)]
struct DieButton;
fn roll_dice() -> (u8, u8, u8) {
    let dice1 = rand::random_range(1..=6);
    let dice2 = rand::random_range(1..=6);
    (dice1 + dice2, dice1, dice2)
}
fn full_roll_dice(
    board: &Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: &Query<'_, '_, (&Town, &CatanColor, &BuildingPosition)>,
    cities: &Query<'_, '_, (&City, &CatanColor, &BuildingPosition)>,
    player_resources: &mut Query<'_, '_, (&CatanColor, &mut Resources)>,
    resources: &mut ResMut<'_, Resources>,
    robber: &Res<'_, Robber>,
    die_q: &mut Query<'_, '_, &mut Text, With<DieButton>>,
) {
    let (roll, d1, d2) = roll_dice();
    println!("rolled {roll} {:?}", die_q.iter().clone().collect_vec());
    // assumes two dice
    die_q
        .iter_mut()
        .zip([d1, d2])
        .for_each(|(mut die_ui, new_roll)| {
            println!("d {new_roll}");
            **die_ui = new_roll.to_string();
        });

    // TODO: what happens when 7 rolled
    if roll != 7 {
        distribute_resources(
            roll,
            board.iter().map(|(h, n, p)| (*h, *n, *p)),
            towns.iter().map(|(b, c, p)| (*b, *c, *p)),
            cities.iter().map(|(b, c, p)| (*b, *c, *p)),
            player_resources
                .iter_mut()
                .map(|(c, r)| (*c, r.into_inner())),
            resources,
            robber,
        );
    }
}

fn distribute_resources<'a>(
    roll: u8,
    board: impl Iterator<Item = (Hexagon, Number, Position)> + Clone,
    towns: impl Iterator<Item = (Town, CatanColor, BuildingPosition)>,
    cities: impl Iterator<Item = (City, CatanColor, BuildingPosition)>,
    player_resources: impl Iterator<Item = (CatanColor, &'a mut Resources)>,
    resources: &mut Resources,
    robber: &Robber,
) {
    let mut player_resources = player_resources.collect_vec();
    let board = board.filter(|(_, number, p)| {
        p != &robber.0 && matches!(number, Number::Number(n) if *n == roll)
    });
    fn on_board_with_hex<Building>(
        mut board: impl Iterator<Item = (Hexagon, Number, Position)>,
        buildings: impl Iterator<Item = (Building, CatanColor, BuildingPosition)>,
    ) -> impl Iterator<Item = (Building, CatanColor, Hexagon)> {
        buildings.filter_map(move |(b, catan_color, BuildingPosition::All(p1, p2, p3))| {
            // does this need to be cloned
            board
                .find(|(_, _, pos)| pos == &p1 || pos == &p2 || pos == &p3)
                .map(|(hex, _, _)| (b, catan_color, hex))
        })
    }
    fn get_by_color<'a, T: 'a>(
        color: &CatanColor,
        mut things: impl Iterator<Item = &'a mut (CatanColor, T)>,
    ) -> Option<&'a mut T> {
        things.find(|(c, _)| c == color).map(|(_, t)| t)
    }
    fn hexagon_to_resources(hex: Hexagon) -> Resources {
        match hex {
            Hexagon::Wood => Resources {
                wood: 1,
                brick: 0,
                sheep: 0,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Brick => Resources {
                wood: 0,
                brick: 1,
                sheep: 0,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Sheep => Resources {
                wood: 0,
                brick: 0,
                sheep: 1,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Wheat => Resources {
                wood: 0,
                brick: 0,
                sheep: 0,
                wheat: 1,
                ore: 0,
            },
            Hexagon::Ore => Resources {
                wood: 0,
                brick: 0,
                sheep: 0,
                wheat: 0,
                ore: 1,
            },
            Hexagon::Desert => todo!(),
            Hexagon::Water => todo!(),
            Hexagon::Port => todo!(),
            Hexagon::Empty => todo!(),
        }
    }
    for (b, color, hex) in on_board_with_hex(board.clone(), towns) {
        let player_resources = get_by_color(&color, player_resources.iter_mut());
        if let Some(player_resources) = player_resources {
            let gained = hexagon_to_resources(hex);
            **player_resources += gained;
            *resources -= gained;
        }
    }
    for (b, color, hex) in on_board_with_hex(board, cities) {
        let player_resources = get_by_color(&color, player_resources.iter_mut());
        if let Some(player_resources) = player_resources {
            let gained = hexagon_to_resources(hex) * 2;
            **player_resources += gained;
            *resources -= gained;
        }
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

    println!("current new roads");
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
            println!("original road {road:?}");
            match road {
                RoadPostion::Both(p1, p2, _) => {
                    let (p3, p4) = road.neighboring_two(Some(size_r.0));
                    println!("p3 {p3:?} p4 {p4:?}");
                    let make_road_pos = |p, option_p1: Option<_>, p2: &Position| {
                        option_p1.and_then(|p1| {
                            println!(
                                "non filtered roads {p:?}-{p1:?} = {:?}",
                                RoadPostion::new(p, p1, Some(size_r.0))
                            );
                            RoadPostion::new(p, p1, Some(size_r.0)).map(|r| (*p2, r))
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

    println!("filtering out neighboring roads");
    // 2) make sure that there is no road already there (whether that color or not)
    let possible_roads = possibles_roads
        .filter(|(_, r)| !road_q.iter().any(|RoadQueryItem(_, _, r1)| r == r1))
        .inspect(|r| println!("{:?}", r.1));

    // 3) make sure there is no differeent color town at the three itersection
    // partition into other color used towns with single partiton
    fn filter_by_building<'a>(
        (road1, road2): &(Position, RoadPostion),
        mut building_q: impl Iterator<Item = &'a BuildingPosition>,
    ) -> bool {
        let road_intersection = match road2 {
            RoadPostion::Both(position, position1, _) => {
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
                        top: Val::Px(y * 28.),
                        ..default()
                    },
                    p,
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
                        top: Val::Px(y * 28.),
                        ..default()
                    },
                    p,
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

    let (current_color_roads, _): (Vec<_>, Vec<_>) =
        road_q.into_iter().partition(|r| *r.1 == color_r.0);

    let possibles_towns = buildings_on_roads(
        current_color_roads
            .into_iter()
            .map(|RoadQueryItem(_, _, road)| *road),
        BoardSize(size_r.0),
    );

    let filter_by_building =
        |position: &BuildingPosition,
         building_q: Query<'_, '_, (&_, &CatanColor, &BuildingPosition)>| {
            match position {
                BuildingPosition::All(position, position1, position2) => !buildings_on_roads(
                    [
                        RoadPostion::new(*position, *position1, Some(size_r.0)),
                        RoadPostion::new(*position, *position2, Some(size_r.0)),
                        RoadPostion::new(*position1, *position2, Some(size_r.0)),
                    ]
                    .into_iter()
                    .flatten(),
                    BoardSize(size_r.0),
                )
                .any(|p| building_q.iter().any(|(_, _, place_b)| &p == place_b)),
            }
        };
    let possible_towns = possibles_towns
        .inspect(|p| println!("possible town {p:?}"))
        .filter(|r| filter_by_building(r, building_q));
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
                        top: Val::Px(y * 28.),
                        ..default()
                    },
                    p,
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

fn buildings_on_roads(
    current_color_roads: impl Iterator<Item = RoadPostion>,
    size_r: BoardSize,
) -> impl Iterator<Item = BuildingPosition> {
    current_color_roads.flat_map(move |road| match road {
        RoadPostion::Both(p1, p2, _) => {
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
        let x = 3f32
            .sqrt()
            .mul_add(f32::from(q.0.q), 3f32.sqrt() / 2. * f32::from(q.0.r));
        let y = 3. / 2. * f32::from(q.0.r);
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
#[derive(Component, PartialEq, Debug, Clone, Copy)]
enum PiecePostion {
    None,
    Position(Position),
}
impl From<Option<Position>> for PiecePostion {
    fn from(value: Option<Position>) -> Self {
        value.map_or(Self::None, Self::Position)
    }
}
impl PiecePostion {
    fn map(self, f: impl FnOnce(Position) -> Position) -> Self {
        match self {
            Self::None => Self::None,
            Self::Position(position) => Self::Position(f(position)),
        }
    }

    fn map_option<T>(self, f: impl FnOnce(Position) -> T) -> Option<T> {
        match self {
            Self::None => None,
            Self::Position(position) => Some(f(position)),
        }
    }

    fn and_then(self, f: impl FnOnce(Position) -> Self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Position(position) => f(position),
        }
    }

    fn and_then_option<T>(self, f: impl FnOnce(Position) -> Option<T>) -> Option<T> {
        match self {
            Self::None => None,
            Self::Position(position) => f(position),
        }
    }
}
#[derive(Component, PartialEq, Eq)]
struct Left(pub u8);
#[derive(Component, Clone, Copy, Debug)]
enum RoadPostion {
    /// Dont use this constructor use `Self::new`
    Both(Position, Position, Coordinate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Coordinate {
    Q,
    R,
    S,
}
impl RoadPostion {
    // for creating none edge roads
    fn new(p1: Position, p2: Position, size: Option<u8>) -> Option<Self> {
        let not_off_board = size.is_none_or(|size| {
            p1.all_points_are(|p| -(size as i8) < p && p < size as i8)
                || p2.all_points_are(|p| -(size as i8) < p && p < size as i8)
        });
        let c = p1.get_shared_coordinate(&p2).filter(|_| not_off_board);
        c.map(|c| Self::Both(p1, p2, c))
    }
    fn neighboring_two(&self, size: Option<u8>) -> (Option<Position>, Option<Position>) {
        match self {
            Self::Both(p1, p2, coordinate) => {
                // maybe just do permutations of two other point that add up to 0
                match coordinate {
                    Coordinate::Q => (
                        Position::new(p1.q + 1, p1.r.min(p2.r), p1.s.min(p2.s), size),
                        Position::new(p1.q - 1, p1.r.max(p2.r), p1.s.max(p2.s), size),
                    ),
                    Coordinate::R => (
                        Position::new(p1.q.min(p2.q), p1.r + 1, p1.s.min(p2.s), size),
                        Position::new(p1.q.max(p2.q), p1.r - 1, p1.s.max(p2.s), size),
                    ),
                    Coordinate::S => (
                        Position::new(p1.q.min(p2.q), p1.r.min(p2.r), p1.s + 1, size),
                        Position::new(p1.q.max(p2.q), p1.r.max(p2.r), p1.s - 1, size),
                    ),
                }
            }
        }
    }
    const fn shared_coordinate(&self) -> Coordinate {
        match self {
            Self::Both(_, _, coordinate) => *coordinate,
        }
    }
    fn positon_to_pixel_coordinates(&self) -> (f32, f32) {
        match self {
            Self::Both(position, position1, coordinate) => {
                let fposition: FPosition = (*position).into();
                fposition
                    .interesect_with_coordinate((*position1).into(), *coordinate)
                    .hex_to_pixel()
            }
        }
    }
}

impl PartialEq for RoadPostion {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Both(l0, l1, l2), Self::Both(r0, r1, r2)) => {
                ((l0 == r0 && l1 == r1) || (l0 == r1 && l1 == r0)) && l2 == r2
            }
            _ => false,
        }
    }
}

// town city "enherit" from building make some quries easier
// i think right way to do it with is with `[require(..)]`
#[derive(Component, PartialEq, Default, Clone, Copy)]
struct Building;
#[derive(Component, Clone, Copy, Debug)]
enum BuildingPosition {
    All(Position, Position, Position),
}

impl BuildingPosition {
    const fn new(p1: Position, p2: Position, p3: Position, size: Option<u8>) -> Option<Self> {
        Some(Self::All(p1, p2, p3))
    }

    fn positon_to_pixel_coordinates(&self) -> (f32, f32) {
        match self {
            Self::All(position, position1, position2) => {
                let fposition: FPosition = (*position).into();
                let fposition1: FPosition = (*position1).into();
                let fposition2: FPosition = (*position2).into();
                ((fposition + fposition1 + fposition2) / 3.).hex_to_pixel()
            }
        }
    }
}

impl PartialEq for BuildingPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::All(l0, l1, l2), Self::All(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
                    || l0 == r0 && l1 == r2 && l2 == r1
                    || l0 == r1 && l1 == r0 && l2 == r2
                    || l0 == r1 && l1 == r2 && l2 == r0
                    || l0 == r2 && l1 == r0 && l2 == r1
                    || l0 == r2 && l1 == r1 && l2 == r0
            }
            _ => false,
        }
    }
}
#[derive(Resource, PartialEq, Clone, Copy)]
struct Robber(Position);
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
    commands.spawn((
        Road,
        CatanColor::White,
        RoadPostion::new(
            Position { q: 0, r: 0, s: 0 },
            Position { q: -1, r: 0, s: 1 },
            // Position { q: 2, r: -1, s: -1 },
            // Position { q: 2, r: 0, s: -2 },
            Some(3),
        )
        .unwrap(),
    ));
}
fn setup_dice(mut commands: Commands<'_, '_>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            ..default()
        },
        children![
            (
                Node {
                    left: Val::Px(-15.),
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(1.)),
                    top: Val::Px(-4.),
                    ..default()
                },
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
                    left: Val::Px(-4.),
                    top: Val::Px(-4.),

                    width: Val::Px(20.),
                    height: Val::Px(20.0),

                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
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
            )
        ],
    ));
}
#[derive(QueryData, Debug, Clone, Copy)]

pub struct RoadQuery(&'static Road, &'static CatanColor, &'static RoadPostion);
fn setup(
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut commands: Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    draw_board(
        generate_bord(&mut commands).into_iter(),
        materials,
        meshes,
        &mut commands,
    );
    generate_development_cards(&mut commands);
    generate_pieces(&mut commands);
    next_state.set(GameState::Roll);
    setup_dice(commands);
}
