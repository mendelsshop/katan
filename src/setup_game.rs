//! functions to generate initial game state
//! like hex placement
use std::{iter, marker::PhantomData, mem::swap};

use crate::{
    Hexagon, Knights, Layout, Left, Number, Port, Road, Robber, Town, VictoryPoints,
    cities::City,
    colors::{CatanColor, CatanColorRef},
    development_cards::{DevelopmentCard, DevelopmentCards},
    positions::{self, BuildingPosition, FPosition, Position},
    resources,
    resources::Resources,
};
use bevy::prelude::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
fn draw_board(
    q: impl Iterator<Item = (Position, Hexagon, Number)>,
    port_q: impl Iterator<Item = (BuildingPosition, Port)>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    commands: &mut Commands<'_, '_>,
    layout: Layout,
) {
    let text_justification = JustifyText::Center;

    for q in port_q {
        let mesh = meshes.add(Circle::new(30.0));
        let (x, y) = q.0.positon_to_pixel_coordinates();
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(q.1.color())),
            Transform::from_xyz(x * 77.0, y * 77., 0.0),
        ));

        // let mesh2 = Text2d::new(q.0.0.to_string());
        // commands.spawn((
        //     mesh2,
        //     TextLayout::new_with_justify(text_justification),
        //     TextFont {
        //         font_size: 45.0,
        //         ..Default::default()
        //     },
        //     Transform::from_xyz(x * 77.0, y * 77., 0.0),
        // ));
    }
    // let mut commands = commands.entity(layout.board);
    for q in q {
        let mesh = meshes.add(RegularPolygon::new(70.0, 6));
        let mesh1 = meshes.add(Circle::new(30.0));
        let (x, y) = FPosition::hex_to_pixel(q.0.into());
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(q.1.color())),
            Transform::from_xyz(x * 77.0, y * 77., 0.0),
        ));

        if let Number::Number(n) = q.2 {
            let mesh2 = Text2d::new(n.to_string());
            commands.spawn((
                Mesh2d(mesh1),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(x * 77.0, y * 77., 0.0),
            ));
            commands.spawn((
                mesh2,
                TextLayout::new_with_justify(text_justification),
                TextFont {
                    font_size: 45.0,
                    ..Default::default()
                },
                Transform::from_xyz(x * 77.0, y * 77., 0.0),
            ));
        }
    }
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
    let (inhabited, mut desert): (Vec<_>, Vec<_>) = positions::generate_postions(3)
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
    inhabited
        .extend(positions::generate_postions_ring(3).map(|p| (p, Hexagon::Empty, Number::None)));
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
#[derive(Debug, Component, Clone, Copy)]
pub struct Ports {
    three_for_one: bool,
    two_for_one_wood: bool,
    two_for_one_brick: bool,
    two_for_one_sheep: bool,
    two_for_one_wheat: bool,
    two_for_one_ore: bool,
}
impl Ports {
    pub fn get_trade_rate(&self, resource: resources::Resource) -> u8 {
        if match resource {
            resources::Resource::Wood => self.two_for_one_wood,
            resources::Resource::Brick => self.two_for_one_brick,
            resources::Resource::Sheep => self.two_for_one_sheep,
            resources::Resource::Wheat => self.two_for_one_wheat,
            resources::Resource::Ore => self.two_for_one_ore,
        } {
            2
        } else if self.three_for_one {
            3
        } else {
            4
        }
    }
}
fn generate_port_positions(n: i8) -> impl Iterator<Item = BuildingPosition> {
    // very order dependent
    building_postions_on_ring(n)
        .enumerate()
        .filter(|(i, _)| i % 10 != 0)
        .map(|(_, pos)| pos)
        .enumerate()
        .filter(|(i, _)| i % 3 != 0)
        .map(|(_, pos)| pos)
    // when its an end of a 3 set it postion will have two from the third ring, otherwise it
}

fn building_postions_on_ring(n: i8) -> impl Iterator<Item = BuildingPosition> {
    // start at base (some variation of n, -n, 0)
    let base = Position::new(0, -n + 1, n - 1, Some(n as u8)).unwrap();
    let up = Position::new(0, -1, 1, Some(n as u8)).unwrap();
    let up_right = Position::new(1, -1, 0, Some(n as u8)).unwrap();
    let right = Position::new(1, 0, -1, Some(n as u8)).unwrap();
    let left_building =
        unsafe { BuildingPosition::new_unchecked(base, base + up, base + up_right) };
    let right_building =
        unsafe { BuildingPosition::new_unchecked(base, base + up_right, base + right) };
    let row = iter::once(right_building).chain((1..n).flat_map(move |i| {
        [
            left_building + Position::new(1 * i, 0, -1 * i, None).unwrap(),
            right_building + Position::new(1 * i, 0, -1 * i, None).unwrap(),
        ]
    }));
    (0..6).flat_map(move |i| row.clone().map(move |town| town.rotate_right_n(i)))
}
fn generate_pieces(
    commands: &mut Commands<'_, '_>,
    colors: vec::IntoIter<CatanColor>,
) -> impl Iterator<Item = CatanColorRef> {
    colors.map(|color| CatanColorRef {
        color,
        entity: commands
            .spawn((
                color,
                Left::<Town>(5, PhantomData),
                Left::<City>(4, PhantomData),
                Left::<Road>(15, PhantomData),
                Resources::new_player(),
                DevelopmentCards::new_player(),
                VictoryPoints(0),
                Knights(0),
            ))
            .id(),
    })
}
fn generate_ports(commands: &mut Commands<'_, '_>) -> Vec<(BuildingPosition, Port)> {
    // very hacky and order dependent
    let positions = generate_port_positions(3);
    let mut ports = [
        Port::ThreeForOne,
        Port::ThreeForOne,
        Port::ThreeForOne,
        Port::ThreeForOne,
        Port::TwoForOne(resources::Resource::Wood),
        Port::TwoForOne(resources::Resource::Brick),
        Port::TwoForOne(resources::Resource::Sheep),
        Port::TwoForOne(resources::Resource::Wheat),
        Port::TwoForOne(resources::Resource::Ore),
    ];

    ports.shuffle(&mut rand::rng());
    positions
        // we duplicate each port type because the postions iterator just returns each port postion
        // seperatly even though a port in the game occupies two intersections, we represent each
        // intersection seperatly but we happen to know that are in order
        .zip(ports.iter().flat_map(|c| [*c, *c]))
        .map(|(pos, port)| {
            commands.spawn((pos, port));
            (pos, port)
        })
        .collect()
}
pub fn setup(
    commands: &mut Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
    layout: Layout,
    colors: vec::IntoIter<CatanColor>,
) -> vec::IntoIter<CatanColorRef> {
    draw_board(
        generate_board(commands).into_iter(),
        generate_ports(commands).into_iter(),
        materials,
        meshes,
        commands,
        layout,
    );
    generate_development_cards(commands);
    generate_pieces(commands, colors).collect_vec().into_iter()
}
