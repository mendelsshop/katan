use bevy::prelude::*;

use itertools::Itertools;
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

#[derive(Debug, Component)]
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
            Hexagon::Wood => Color::srgb_u8(161, 102, 47),
            Hexagon::Brick => Color::srgb_u8(198, 74, 60),
            Hexagon::Sheep => Color::srgb_u8(0, 255, 0),
            Hexagon::Wheat => Color::srgb_u8(255, 191, 0),
            Hexagon::Ore => Color::srgb_u8(67, 67, 65),
            Hexagon::Desert => Color::srgba_u8(210, 180, 140, 1),
            Hexagon::Water => Color::srgb_u8(0, 0, 255),
            Hexagon::Port => Color::srgb_u8(0, 0, 255),
            Hexagon::Empty => Color::BLACK.with_alpha(-1.),
        }
    }
}

fn generate_bord(commands: &mut Commands, n: i8) {
    // 1 for first layer 6 for second layer 12 for third layer
    (0..3)
        .map(|_| -n + 1..n)
        .multi_cartesian_product()
        .filter(|q| q[0] + q[1] + q[2] == 0)
        .for_each(|i| {
            commands.spawn((
                Hexagon::Desert,
                Position {
                    q: i[0],
                    r: i[1],
                    s: i[2],
                },
            ));
        });
}
#[derive(Debug)]
enum Port {}
#[derive(Resource)]
struct BoardSize(u8);
fn update_board_piece(mut q: Query<(&mut Hexagon, &Position)>) {
    q.iter_mut()
        .for_each(|mut foo| *foo.0 = rand::random::<u8>().into());
}
fn update_board(
    size: Res<BoardSize>,
    q: Query<(&Hexagon, &Position)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for q in q {
        let mesh = meshes.add(RegularPolygon::new(25.0, 6));
        let x = 3f32
            .sqrt()
            .mul_add(q.1.q as f32, 3f32.sqrt() / 2. * (q.1.r as f32));
        let y = 3. / 2. * (q.1.r as f32);

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
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    generate_bord(&mut commands, 3);
}
