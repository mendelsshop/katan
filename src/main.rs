#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use bevy::prelude::*;

use itertools::Itertools;
use rand::seq::SliceRandom;
fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.insert_resource(BoardSize(3));
    app.add_systems(Startup, setup);
    app.add_systems(FixedUpdate, update_board);
    app.add_systems(FixedUpdate, update_board_piece);
    app.run();
}

#[derive(Component)]
struct Number(u8);
#[derive(Component, Debug)]
struct Position {
    q: i8,
    r: i8,
    s: i8,
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

fn generate_bord(commands: &mut Commands<'_, '_>) {
    // 1 for first layer 6 for second layer 12 for third layer
    let mut postions = [Hexagon::Wheat; 4]
        .into_iter()
        .chain([Hexagon::Sheep; 4])
        .chain([Hexagon::Wood; 4])
        .chain([Hexagon::Desert; 1])
        .chain([Hexagon::Brick; 3])
        .chain([Hexagon::Ore; 3])
        .collect_vec();
    postions.shuffle(&mut rand::rng());
    generate_postions(3).zip(postions).for_each(|i| {
        commands.spawn(i);
    });
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
fn update_board(
    size: Res<'_, BoardSize>,
    q: Query<'_, '_, (&Hexagon, &Position)>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut commands: Commands<'_, '_>,
) {
    for q in q {
        let mesh = meshes.add(RegularPolygon::new(25.0, 6));
        let x = 3f32
            .sqrt()
            .mul_add(f32::from(q.1.q), 3f32.sqrt() / 2. * f32::from(q.1.r));
        let y = 3. / 2. * f32::from(q.1.r);

        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(q.0.color())),
            Transform::from_xyz(
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                x * 28.0,
                // -X_EXTENT / 2. + (q.1.0 + size.0 as i8) as f32 / (19 - 1) as f32 * X_EXTENT,
                y * 28.,
                // -X_EXTENT / 2. + (q.1.1 + size.0 as i8) as f32 / (19 - 1) as f32 * X_EXTENT,
                // -X_EXTENT / 2. + ( q.1.2.abs()) as f32 / (3 - 1) as f32 * X_EXTENT,
                0.0,
            ),
        ));
    }
}
fn setup(
    mut commands: Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    generate_bord(&mut commands);
}
