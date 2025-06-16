#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use std::{mem::swap, ops::Add};

use bevy::prelude::*;

use itertools::Itertools;
use rand::seq::{IndexedMutRandom, SliceRandom};
fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.insert_resource(BoardSize(3));
    app.add_systems(Startup, setup);
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
    let partition: (Vec<_>, Vec<_>) = generate_postions(3)
        .zip(inhabited)
        .map(|(position, (hex, number))| (position, hex, number))
        .partition(|(_, _, n)| Number::Number(8) == *n || Number::Number(6) == *n);
    let inhabited = fix_numbers(partition.0, partition.1);
    inhabited.iter().for_each(|hex| {
        commands.spawn((hex.0, hex.1, hex.2));
    });
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
        // only works if bouard < 100 pieces
        let length = 100 / used.len();
        reds.iter_mut().filter(|p| touches(p.0)).for_each(|red| {
            let new_hexagon = normal.choose_weighted_mut(&mut rand::rng(), |p| {
                // ignore desert piece (no number)
                if p.2 == Number::None { 0 } else { length }
            });
            if let Ok(new_hexagon) = new_hexagon {
                swap(&mut red.1, &mut new_hexagon.1);
                swap(&mut red.0, &mut new_hexagon.0);
            }
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
}
