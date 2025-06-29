#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use std::{mem::swap, ops::Add};

use bevy::{ecs::query::QueryData, prelude::*, window::PrimaryWindow};

use itertools::Itertools;
use rand::seq::SliceRandom;
fn main() {
    println!("Hello, world!");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.init_state::<GameState>();
    app.insert_resource(BoardSize(3));
    app.insert_resource(CurrentColor(CatanColor::White));
    app.insert_resource(CursorWorldPos(None));
    app.add_systems(Startup, setup);
    app.add_systems(Update, get_cursor_world_pos);
    app.add_systems(FixedUpdate, update_board_piece);
    app.add_systems(OnEnter(GameState::PlaceRoad), (place_normal_road,));
    app.add_systems(OnEnter(GameState::Turn), (show_turn_ui,));

    app.add_systems(
        Update,
        turn_ui_road_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(OnExit(GameState::PlaceRoad), (cleanup_road_place,));
    app.add_systems(
        Update,
        place_normal_road_interaction.run_if(in_state(GameState::PlaceRoad)),
    );
    app.run();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Nothing,
    Start,
    PlaceRoad,
    Turn,
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
    fn hex_to_pixel(self) -> (f32, f32) {
        let x = 3f32.sqrt().mul_add(self.q, 3f32.sqrt() / 2. * self.r);
        let y = 3. / 2. * self.r;
        (x, y)
    }
}
// maybe do size const generics?
impl Position {
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
    // returns the two neighboring hexes for the two hexes passed in
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

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component, PartialEq, Debug, Clone, Copy)]
// button in game to start road placement ui
struct RoadButton;
#[derive(Component, PartialEq, Debug, Clone, Copy)]
// button in gam/ to start town placement ui
struct TownButton;
fn turn_ui_road_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &RoadButton,
            &Interaction,
            // &mut BackgroundColor,
            &mut Button,
        ),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // input_focus.set(entity);
                // **text = "Press".to_string();
                // *color = PRESSED_BUTTON.into();
                // *border_color = BorderColor::all(RED.into());

                // The accessibility system's only update the button's state when the `Button` component is marked as changed.
                button.set_changed();

                // TODO: proper state switch
                game_state.set(GameState::PlaceRoad);
                button.set_changed();
            }
            Interaction::Hovered => {
                // input_focus.set(entity);
                // **text = "Hover".to_string();
                // *color = HOVERED_BUTTON.into();
                // *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                // input_focus.clear();
                // **text = "Button".to_string();
                // *color = NORMAL_BUTTON.into();
                // *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}
fn show_turn_ui(mut commands: Commands<'_, '_>, asset_server: Res<'_, AssetServer>) {
    // TODO: have button with picture of road
    let road_icon = asset_server.load("road.png");
    let town_icon: Handle<Image> = asset_server.load("house.png");
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
                // NodeImageMode
                // BackgroundColor(NORMAL_BUTTON),
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
                // BackgroundColor(NORMAL_BUTTON),
            )
        ],
    ));
}
fn cleanup_road_place(
    mut commands: Commands<'_, '_>,
    mut interaction_query: Query<'_, '_, Entity, (With<RoadPostion>, With<Button>)>,
) {
    for entity in &mut interaction_query {
        commands.entity(entity).despawn();
    }
}
fn place_normal_road_interaction(
    mut game_state: ResMut<'_, NextState<GameState>>,
    color_r: Res<'_, CurrentColor>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
    meshes: ResMut<'_, Assets<Mesh>>,
    mut commands: Commands<'_, '_>,
    mut road_free_q: Query<'_, '_, (&Road, &CatanColor, &mut Left)>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &RoadPostion,
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
                // input_focus.set(entity);
                // **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                // *border_color = BorderColor::all(RED.into());

                // The accessibility system's only update the button's state when the `Button` component is marked as changed.
                button.set_changed();

                commands.spawn((Road, color_r.0, *entity));
                let roads_left = road_free_q.iter_mut().find(|x| x.1 == &color_r.0);
                if let Some((_, _, mut left)) = roads_left {
                    *left = Left(left.0 - 1);
                }

                // TODO: proper state switch
                game_state.set(GameState::Turn);
                button.set_changed();
            }
            Interaction::Hovered => {
                // input_focus.set(entity);
                // **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                // *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                // input_focus.clear();
                // **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                // *border_color = BorderColor::all(Color::BLACK);
            }
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
    town_q: Query<'_, '_, (&'_ Town, &'_ CatanColor, &'_ BuildingPosition)>,
    city_q: Query<'_, '_, (&'_ City, &'_ CatanColor, &'_ BuildingPosition)>,

    cursor_world_pos: ResMut<'_, CursorWorldPos>,
    q_camera: Single<'_, (&Camera, &GlobalTransform)>,
) {
    let unplaced_roads_correct_color = road_free_q.iter().find(|r| r.1 == &color_r.0);

    // nor roads to place
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
                RoadPostion::Both(p1, p2, q) => {
                    // TODO: this currently does not include roads that go from edge inwards
                    // also includes "unplaces roads (roads that all postions are none)

                    // neighboring two seems to be a bit flawed, and maybe should be road postion
                    let (p3, p4) = road.neighboring_two(Some(size_r.0));
                    let make_road_pos = |p, option_p1: Option<_>, p1| {
                        option_p1.and_then(|p1| {
                            println!("{p:?}-{p1:?} = {:?}", RoadPostion::new(p, p1));
                            RoadPostion::new(p, p1).map(|r| (p1, r))
                        })
                    };
                    [
                        (
                            // the other point (used to check for towns/cities)

                            // the postion of the road
                            make_road_pos(*p2, p3, p1)
                            // TODO: roads on edge of board
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
    fn filter_by_building<B: Component>(
        (road1, road2): &(Position, RoadPostion),
        building_q: Query<'_, '_, (&B, &CatanColor, &BuildingPosition)>,
    ) -> bool {
        let road_intersection = match road2 {
            RoadPostion::Both(position, position1, _) => {
                BuildingPosition::All(*road1, *position, *position1)
            }
        };
        !building_q.iter().any(|(_, _, bp)| &road_intersection == bp)
    }
    let possible_roads =
        possible_roads.filter(|r| filter_by_building(r, town_q) && filter_by_building(r, city_q));
    // TODO: show options
    possible_roads
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
                        // border: UiRect::all(Val::Px(2.0)),
                        // horizontally center child text
                        // justify_content: JustifyContent::Center,
                        // vertically center child text
                        // align_items: AlignItems::Center,
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
        .for_each(|b| {
            commands.spawn(b);
        });
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
// If we had whether it was an edge or not as individual struct we could interersting stuff with
// quries at the type level (maybe have base road postion type for quires that use both)
enum RoadPostion {
    // maybe we could enforce more stuff i.e. they will share one coordinate
    // and the others will be +1 or -1 respectively
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
    fn new(p1: Position, p2: Position) -> Option<Self> {
        let c = p1.get_shared_coordinate(&p2);
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
                        Position::new(p1.q.min(p2.q), p1.r.min(p2.r), p1.s + 1, size),
                        Position::new(p1.q.max(p2.q), p1.r.max(p2.r), p1.s - 1, size),
                    ),
                    Coordinate::S => (
                        Position::new(p1.q.min(p2.q), p1.r + 1, p1.s.min(p2.s), size),
                        Position::new(p1.q.max(p2.q), p1.r - 1, p1.s.max(p2.s), size),
                    ),
                }
            }
        }
    }
    fn positon_to_pixel_coordinates(&self) -> (f32, f32) {
        match self {
            Self::Both(
                Position { q, r, s },
                Position {
                    q: q1,
                    r: r1,
                    s: s1,
                },
                Coordinate::Q,
            ) => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                let midpoint = FPosition {
                    q: f32::from(*q),
                    r: f32::from(r + r1) / 2.,
                    s: f32::from(s + s1) / 2.,
                };
                midpoint.hex_to_pixel()
            }
            Self::Both(
                Position { q, r, s },
                Position {
                    q: q1,
                    r: r1,
                    s: s1,
                },
                Coordinate::R,
            ) => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                let midpoint = FPosition {
                    r: f32::from(*r),
                    q: f32::from(q + q1) / 2.,
                    s: f32::from(s + s1) / 2.,
                };
                midpoint.hex_to_pixel()
            }
            Self::Both(
                Position { q, r, s },
                Position {
                    q: q1,
                    r: r1,
                    s: s1,
                },
                Coordinate::S,
            ) => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                let midpoint = FPosition {
                    s: f32::from(*s),
                    r: f32::from(r + r1) / 2.,
                    q: f32::from(q + q1) / 2.,
                };
                midpoint.hex_to_pixel()
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
#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum TwoWayDirection {
    Left,
    Right,
}
#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum ThreeWayDirection {
    Left,
    Middle,
    Right,
}
// TODO: town city "enherit" from building make some quries easier
#[derive(Component, PartialEq)]
struct Building;
#[derive(Component)]
enum BuildingPosition {
    All(Position, Position, Position),
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
    }
    commands.spawn((
        Road,
        CatanColor::White,
        RoadPostion::new(
            Position { q: 0, r: 0, s: 0 },
            Position { q: 0, r: -1, s: 1 },
            // Position { q: 2, r: -1, s: -1 },
            // Position { q: 2, r: 0, s: -2 },
        )
        .unwrap(),
    ));
}
#[derive(QueryData, Debug)]
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
    next_state.set(GameState::Turn);
}
