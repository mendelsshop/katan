#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use std::{mem::swap, ops::Add};

use bevy::{prelude::*, window::PrimaryWindow};

use itertools::Itertools;
use rand::seq::SliceRandom;
fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.insert_resource(BoardSize(3));
    app.insert_resource(CurrentColor(CatanColor::White));
    app.add_systems(Startup, setup);
    app.add_systems(Update, get_cursor_world_pos);
    app.add_systems(FixedUpdate, update_board_piece);
    app.run();
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

// maybe do size const generics?
impl Position {
    pub fn new(q: i8, r: i8, s: i8, size: Option<u8>) -> Option<Self> {
        const fn in_between(x: u8, y: i8) -> bool {
            let x = (x - 1) as i8;
            -x >= y && y <= x
        }
        (q + r + s == 0
            && size.is_none_or(|x| in_between(x, q) && in_between(x, r) && in_between(x, s)))
        .then_some(Self { q, r, s })
    }
    // returns the two neighboring hexes for the two hexes passed in
    fn neighboring_two(&self, other: &Self, size: Option<u8>) -> (Option<Self>, Option<Self>) {
        if self.q == other.q {
            (
                Self::new(self.q + 1, self.r.min(other.r), self.s.min(other.s), size),
                Self::new(self.q - 1, self.r.max(other.r), self.s.max(other.s), size),
            )
        } else if self.s == other.s {
            (
                Self::new(self.s + 1, self.r.min(other.r), self.q.min(other.q), size),
                Self::new(self.s - 1, self.r.max(other.r), self.q.max(other.q), size),
            )
        } else if self.r == other.r {
            (
                Self::new(self.r + 1, self.q.min(other.q), self.s.min(other.s), size),
                Self::new(self.r - 1, self.q.max(other.q), self.s.max(other.s), size),
            )
        } else {
            panic!()
        }
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

#[derive(Debug, Component, Clone, Copy)]
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
#[derive(Debug, Component, Clone, Copy)]
enum Resource {
    Wood = 0,
    Brick,
    Sheep,
    Wheat,
    Ore,
}
#[derive(Debug, Component, Clone, Copy)]
struct Town;
#[derive(Debug, Component, Clone, Copy)]
struct City;
#[derive(Debug, Component, Clone, Copy)]
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
    inhabited.append(&mut desert);
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
#[derive(Resource)]
struct BoardSize(u8);
const fn update_board_piece(q: Query<'_, '_, (&mut Hexagon, &Position)>) {
    // q.iter_mut()
    //     .for_each(|mut foo| *foo.0 = rand::random::<u8>().into());
}

#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);
fn get_cursor_world_pos(
    mut cursor_world_pos: ResMut<'_, CursorWorldPos>,
    primary_window: Single<'_, &Window, With<PrimaryWindow>>,
    q_camera: Single<'_, (&Camera, &GlobalTransform)>,
) {
    let (main_camera, main_camera_transform) = *q_camera;
    // Get the cursor position in the world
    cursor_world_pos.0 = primary_window.cursor_position().and_then(|cursor_pos| {
        main_camera
            .viewport_to_world_2d(main_camera_transform, cursor_pos)
            .ok()
    });
}
// not for initial game setup where the are no roads yet
// TODO: maybe we should impose an order on postions for stuff like roads so that comparing them is
// easeier (i.e. first postion is smallest ....)
fn place_normal_road(
    commands: &'_ mut Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, (&Road, &CatanColor, &PiecePostion, &PiecePostion)>,
    town_q: Query<
        '_,
        '_,
        (
            &'_ Town,
            &'_ CatanColor,
            &'_ PiecePostion,
            &'_ PiecePostion,
            &'_ PiecePostion,
        ),
    >,
    city_q: Query<
        '_,
        '_,
        (
            &'_ City,
            &'_ CatanColor,
            &'_ PiecePostion,
            &'_ PiecePostion,
            &'_ PiecePostion,
        ),
    >,
) {
    let (unplaced_road, placed_road): (Vec<_>, Vec<_>) =
        road_q
            .into_iter()
            .partition(|(_, _, piece_postion, piece_postion1)| {
                **piece_postion == PiecePostion::None && **piece_postion1 == PiecePostion::None
            });
    let (unplaced_current_color_roads, _): (Vec<_>, Vec<_>) =
        unplaced_road.into_iter().partition(|r| *r.1 == color_r.0);
    // nor roads to place
    if unplaced_current_color_roads.is_empty() {
        return;
    }
    let (current_color_roads, other_color_roads): (Vec<_>, Vec<_>) =
        placed_road.into_iter().partition(|r| *r.1 == color_r.0);
    // we don't check current color roads is empty b/c by iterating over them we are essentially
    // doing that already
    // roads are between two hexes (if one coordiante is the same
    // if q same then its flat (assuming hex is flat)
    // if r is same then its diagonol like '\'
    // if s is same then its diagonol like '/'
    // if there is a new place to put road down
    // 1) the new hex has to share one coordianate with one hex and another differenet one with the
    //    other hex (more constraint (i.e cannot be 50 square of in another direction)
    let possibles_roads = current_color_roads.into_iter().flat_map(|(_, _, p1, p2)| {
        // TODO: this currently does not include roads that go from edge inwards
        // also includes "unplaces roads (roads that all postions are none)
        let (p3, p4) = p1
            .and_then_option(|p1| p2.map_option(|p2| p1.neighboring_two(&p2, Some(size_r.0))))
            .map_or((PiecePostion::None, PiecePostion::None), |(p1, p2)| {
                (p1.into(), p2.into())
            });
        [
            (
                // the other point (used to check for towns/cities)
                *p1,
                // the postion of the road
                (
                    *p2, p3,
                    // TODO: roads on edge of board
                ),
            ),
            (*p1, (*p2, p4)),
            (*p2, (*p1, p3)),
            (*p2, (*p1, p4)),
        ]
    });
    // .filter_map(|(p1, (p2, p3))| p3.map(|p3| (p1, (p2, PiecePostion::Position(p3)))));
    // 2) make sure that there is no road already there (whether that color or not)
    let possible_roads = possibles_roads.filter(|(_, (p_n1, p_n2))| {
        other_color_roads
            .iter()
            .any(|(_, _, p1, p2)| (p_n1 == *p1 && p_n2 == *p2) || (p_n1 == *p2 && p_n2 == *p1))
    });
    // 3) make sure there is no differeent color house at the three itersection
    // partition into other color used houses with single partiton
    fn filter_by_building<B: Component>(
        (road1, (road2, road3)): &(PiecePostion, (PiecePostion, PiecePostion)),
        building_q: Query<'_, '_, (&B, &CatanColor, &PiecePostion, &PiecePostion, &PiecePostion)>,
    ) -> bool {
        !building_q
            .iter()
            .any(|(_, _, building1, building2, building3)| {
                road1 == building1 && road2 == building2 && road3 == building3
                    || road1 == building1 && road2 == building3 && road3 == building2
                    || road1 == building2 && road2 == building1 && road3 == building3
                    || road1 == building2 && road2 == building3 && road3 == building1
                    || road1 == building3 && road2 == building1 && road3 == building2
                    || road1 == building3 && road2 == building2 && road3 == building1
                // TODO: houses on edge of board
            })
    }
    let possible_roads =
        possible_roads.filter(|r| filter_by_building(r, town_q) && filter_by_building(r, city_q));
    // TODO: show options
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
fn generate_pieces(commands: &mut Commands<'_, '_>) {
    for color in [
        CatanColor::Red,
        CatanColor::Blue,
        CatanColor::Green,
        CatanColor::White,
    ] {
        fn add_building<T: Bundle>(
            color: &CatanColor,
            commands: &mut Commands<'_, '_>,
        ) -> impl FnMut(T) {
            |thing| {
                commands.spawn((
                    thing,
                    *color,
                    PiecePostion::None,
                    PiecePostion::None,
                    PiecePostion::None,
                ));
            }
        }
        fn add_road<T: Bundle>(
            color: &CatanColor,
            commands: &mut Commands<'_, '_>,
        ) -> impl FnMut(T) {
            |thing| {
                commands.spawn((thing, *color, PiecePostion::None, PiecePostion::None));
            }
        }
        [Town; 5]
            .into_iter()
            .for_each(add_building(&color, commands));
        [City; 4]
            .into_iter()
            .for_each(add_building(&color, commands));

        [Road; 15].into_iter().for_each(add_road(&color, commands));
    }
}
fn setup(
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
}
