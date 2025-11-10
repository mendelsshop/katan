//! functions to generate initial game state
//! like hex placement
use crate::game::{
    LocalPlayer, PlayerCount, PlayerHandle, development_cards::DevelopmentCardsPile,
};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::{
    iter,
    marker::PhantomData,
    mem::swap,
    ops::{Add, AddAssign},
};

use super::{
    Hexagon, Knights, Layout, Left, Number, Port, Robber, VictoryPoints,
    cities::City,
    colors::{CatanColor, CatanColorRef},
    development_cards::{DevelopmentCard, DevelopmentCards},
    longest_road::PlayerLongestRoad,
    positions::{self, BuildingPosition, FPosition, Position},
    resources::{self, Resources},
    roads::Road,
    towns::Town,
};
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_ggrs::{AddRollbackCommandExtension, LocalPlayers};
use itertools::Itertools;
use rand::{Rng, SeedableRng, seq::SliceRandom};
fn draw_board(
    q: impl Iterator<Item = (Position, Hexagon, Number)>,
    port_q: impl Iterator<Item = (BuildingPosition, Port)>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    commands: &mut Commands<'_, '_>,
    layout: Layout,
) {
    let text_justification = Justify::Center;

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
fn generate_development_cards(commands: &mut Commands<'_, '_>, rng: &mut Xoshiro256PlusPlus) {
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
    development_cards.shuffle(rng);

    commands.insert_resource(DevelopmentCardsPile(development_cards.to_vec()));
}
fn generate_board(
    commands: &mut Commands<'_, '_>,
    rng: &mut Xoshiro256PlusPlus,
) -> Vec<(Position, Hexagon, Number)> {
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
    numbers.shuffle(rng);
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

    inhabited.shuffle(rng);
    let (inhabited, mut desert): (Vec<_>, Vec<_>) = positions::generate_postions(3)
        .zip(inhabited)
        .map(|(position, (hex, number))| (position, hex, number))
        .partition(|p| p.2 != Number::None);
    let (reds, normal_number): (Vec<_>, Vec<_>) = inhabited
        .into_iter()
        .partition(|(_, _, n)| Number::Number(8) == *n || Number::Number(6) == *n);
    let mut inhabited = fix_numbers(reds, normal_number, rng);
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
    rng: &mut Xoshiro256PlusPlus,
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
                normal.swap_remove((rng.random::<u8>() % normal.len() as u8) as usize);

            swap(&mut red.1, &mut new_hexagon.1);
            swap(&mut red.0, &mut new_hexagon.0);
            used.push(new_hexagon);
        });
    }

    used.append(&mut normal);
    used
}
#[derive(Debug, Component, Clone, Copy, Default)]
pub struct Ports {
    three_for_one: bool,
    two_for_one_wood: bool,
    two_for_one_brick: bool,
    two_for_one_sheep: bool,
    two_for_one_wheat: bool,
    two_for_one_ore: bool,
}
impl Ports {
    pub const fn new_player() -> Self {
        Self {
            three_for_one: false,
            two_for_one_wood: false,
            two_for_one_brick: false,
            two_for_one_sheep: false,
            two_for_one_wheat: false,
            two_for_one_ore: false,
        }
    }
    pub const fn get_trade_rate(&self, resource: resources::Resource) -> u8 {
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
impl Add<Port> for Ports {
    type Output = Self;

    fn add(self, rhs: Port) -> Self::Output {
        match rhs {
            Port::TwoForOne(resources::Resource::Wood) => Self {
                two_for_one_wood: true,
                ..self
            },
            Port::TwoForOne(resources::Resource::Brick) => Self {
                two_for_one_brick: true,
                ..self
            },
            Port::TwoForOne(resources::Resource::Sheep) => Self {
                two_for_one_sheep: true,
                ..self
            },
            Port::TwoForOne(resources::Resource::Wheat) => Self {
                two_for_one_wheat: true,
                ..self
            },
            Port::TwoForOne(resources::Resource::Ore) => Self {
                two_for_one_ore: true,
                ..self
            },
            Port::ThreeForOne => Self {
                three_for_one: true,
                ..self
            },
        }
    }
}

impl AddAssign<Port> for Ports {
    fn add_assign(&mut self, rhs: Port) {
        match rhs {
            Port::TwoForOne(resources::Resource::Wood) => self.two_for_one_wood = true,
            Port::TwoForOne(resources::Resource::Brick) => self.two_for_one_brick = true,
            Port::TwoForOne(resources::Resource::Sheep) => self.two_for_one_sheep = true,
            Port::TwoForOne(resources::Resource::Wheat) => self.two_for_one_wheat = true,
            Port::TwoForOne(resources::Resource::Ore) => self.two_for_one_ore = true,
            Port::ThreeForOne => self.three_for_one = true,
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
            left_building + Position::new(i, 0, -i, None).unwrap(),
            right_building + Position::new(i, 0, -i, None).unwrap(),
        ]
    }));
    (0..6).flat_map(move |i| row.clone().map(move |town| town.rotate_right_n(i)))
}
fn generate_pieces(
    commands: &mut Commands<'_, '_>,
    player_count: u8,
    rng: &mut Xoshiro256PlusPlus,
    local_players: Res<'_, LocalPlayers>,
) -> Vec<CatanColorRef> {
    let mut catan_colors = vec![
        CatanColor::White,
        CatanColor::Green,
        CatanColor::Red,
        CatanColor::Blue,
    ];
    catan_colors.shuffle(rng);

    let colors = catan_colors.into_iter();

    colors
        .enumerate()
        .take(player_count as usize)
        .map(|(handle, color)| {
            let catan_color_ref = CatanColorRef {
                color,
                handle: PlayerHandle(handle),
                entity: commands
                    .spawn((
                        color,
                        Left::<Town>(5, PhantomData),
                        Left::<City>(4, PhantomData),
                        Left::<Road>(15, PhantomData),
                        PlayerHandle(handle),
                        Resources::new_player(),
                        // maybe initialize this as part of longest road plugin just have system that run
                        // on startup(or eventually game start state) that adds this comonent to each
                        // "player"
                        PlayerLongestRoad(HashSet::new()),
                        DevelopmentCards::new_player(),
                        Ports::new_player(),
                        VictoryPoints {
                            actual: 0,
                            from_development_cards: 0,
                        },
                        Knights(0),
                    ))
                    .add_rollback()
                    .id(),
            };
            if local_players.0.contains(&handle) {
                commands.insert_resource(LocalPlayer(catan_color_ref));
            }
            catan_color_ref
        })
        .collect_vec()
}
fn generate_ports(
    commands: &mut Commands<'_, '_>,
    rng: &mut Xoshiro256PlusPlus,
) -> Vec<(BuildingPosition, Port)> {
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

    ports.shuffle(rng);
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
    player_count: Res<'_, PlayerCount>,
    seed: u64,
    local_players: Res<'_, LocalPlayers>,
) -> vec::IntoIter<CatanColorRef> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    draw_board(
        generate_board(commands, &mut rng).into_iter(),
        generate_ports(commands, &mut rng).into_iter(),
        materials,
        meshes,
        commands,
        layout,
    );
    generate_development_cards(commands, &mut rng);
    generate_pieces(commands, player_count.0, &mut rng, local_players).into_iter()
}
