use bevy::prelude::*;

use itertools::Itertools;
fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.insert_resource(BoardSize(3));
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_board);
    app.run();
}
const X_EXTENT: f32 = 900.;

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
    Wood,
    Brick,
    Sheep,
    Wheat,
    Ore,
    Desert,
    Water,
    Port(Port),
    Empty,
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

fn update_board(
    size: Res<BoardSize>,
    q: Query<(&Hexagon, &Position)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for q in q {
        let mesh = meshes.add(RegularPolygon::new(25.0, 6));
        let color = Color::hsl(360. * 3 as f32 / 1 as f32, 0.95, 0.7);
        let x = 3f32
            .sqrt()
            .mul_add(q.1.q as f32, (3f32.sqrt() / 2. * (q.1.r as f32)));
        let y = 3. / 2. * (q.1.r as f32);
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(color)),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    generate_bord(&mut commands, 3);
    // for (i, shape) in shapes.into_iter().enumerate() {
    //     // Distribute colors evenly across the rainbow.
    //     let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);
    //
    //     commands.spawn((
    //         Mesh2d(shape),
    //         MeshMaterial2d(materials.add(color)),
    //         Transform::from_xyz(
    //             // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
    //             -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
    //             0.0,
    //             0.0,
    //         ),
    //     ));
    // }
}
