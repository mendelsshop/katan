use bevy::{ecs::query::QueryData, prelude::*};
use itertools::Itertools;

use crate::{
    BoardSize, Building, GameState, Left, UI,
    colors::{CatanColor, CurrentColor, NORMAL_BUTTON},
    positions::{self, BuildingPosition, Coordinate, Position, RoadPosition},
    resources::{ROAD_RESOURCES, Resources},
    towns::{buildings_on_road, check_no_touching_buildings},
};

#[derive(QueryData, Debug, Clone, Copy)]
pub struct RoadQuery(
    pub &'static Road,
    pub &'static CatanColor,
    pub &'static RoadPosition,
);

#[derive(Debug, Component, Clone, Copy, Default)]
pub struct Road;
pub fn place_normal_road(
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
fn get_setup_road_placements(
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,

    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
) -> impl Iterator<Item = RoadPosition> {
    let size = BoardSize(size_r.0);
    // generate all road possobilties
    // generate the ring around it for edge roads
    positions::generate_postions(4)
        .array_combinations::<2>()
        .filter_map(move |[p1, p2]| RoadPosition::new(p1, p2, Some(size_r.0)))
        // filter out ones that are already placed
        .filter(move |road| !road_q.iter().map(|r| r.2).contains(road))
        // only show road if town can placed near it
        .filter(move |r| {
            buildings_on_road(size, *r).any(|b| check_no_touching_buildings(&b, building_q, size.0))
        })
}
pub fn place_setup_road(
    mut commands: Commands<'_, '_>,
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let count = get_setup_road_placements(size_r, road_q, building_q)
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
pub struct RoadUI;
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
