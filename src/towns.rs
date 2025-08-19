use bevy::prelude::*;

use crate::{
    BoardSize, Building, GameState, Left, UI,
    colors::{CatanColor, CurrentColor, CurrentSetupColor, NORMAL_BUTTON},
    positions::{BuildingPosition, RoadPosition},
    resources::{Resources, TOWN_RESOURCES},
    roads::{RoadQuery, RoadQueryItem},
};

#[derive(Debug, Component, Clone, Copy, Default)]
#[require(Building)]
pub struct Town;
pub fn place_normal_town(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    town_free_q: Query<'_, '_, (&Left<Town>), With<CatanColor>>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_towns_correct_color = town_free_q.get(color_r.0.entity);

    // no towns to place
    let Some(_) = unplaced_towns_correct_color.ok().filter(|r| r.0 > 0) else {
        return;
    };

    let possible_towns =
        get_possible_town_placements(color_r.0.color, BoardSize(size_r.0), road_q, building_q);
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
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    p,
                    TownUI::resources(),
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
pub fn place_setup_town(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentSetupColor>,
    size_r: Res<'_, BoardSize>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &'_ CatanColor, &'_ BuildingPosition)>,
) {
    let possible_towns =
        get_possible_town_placements(color_r.0, BoardSize(size_r.0), road_q, building_q);
    possible_towns
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
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    p,
                    Resources::default(),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )],
            )
        })
        .for_each(|b| {
            commands.spawn(b);
        });
}
fn get_possible_town_placements(
    color_r: CatanColor,
    size_r: BoardSize,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&Building, &CatanColor, &BuildingPosition)>,
) -> impl Iterator<Item = BuildingPosition> {
    let (current_color_roads, _): (Vec<_>, Vec<_>) =
        road_q.into_iter().partition(|r| *r.1 == color_r);

    let possibles_towns = buildings_on_roads(
        current_color_roads
            .into_iter()
            .map(|RoadQueryItem(_, _, road)| *road),
        BoardSize(size_r.0),
    );

    possibles_towns.filter(move |r| check_no_touching_buildings(r, building_q, size_r.0))
}
/// verifies that there is no buildings with in one road of this building
pub fn check_no_touching_buildings(
    position: &BuildingPosition,
    building_q: Query<'_, '_, (&Building, &CatanColor, &BuildingPosition)>,
    size_r_0: u8,
) -> bool {
    match position {
        BuildingPosition::All(position, position1, position2) => !buildings_on_roads(
            [
                RoadPosition::new(*position, *position1, Some(size_r_0)),
                RoadPosition::new(*position, *position2, Some(size_r_0)),
                RoadPosition::new(*position1, *position2, Some(size_r_0)),
            ]
            .into_iter()
            .flatten(),
            BoardSize(size_r_0),
        )
        .any(|p| building_q.iter().any(|(_, _, place_b)| &p == place_b)),
    }
}
fn buildings_on_roads(
    current_color_roads: impl Iterator<Item = RoadPosition>,
    size_r: BoardSize,
) -> impl Iterator<Item = BuildingPosition> {
    current_color_roads.flat_map(move |road| buildings_on_road(size_r, road))
}

pub fn buildings_on_road(
    size_r: BoardSize,
    road: RoadPosition,
) -> impl Iterator<Item = BuildingPosition> {
    match road {
        RoadPosition::Both(p1, p2, _) => {
            let (p3, p4) = road.neighboring_two(Some(size_r.0));
            let make_town_pos = |p, option_p1: Option<_>, p2| {
                option_p1.and_then(|p1| BuildingPosition::new(p, p1, p2, Some(size_r.0)))
            };
            [(make_town_pos(p1, p3, p2)), (make_town_pos(p1, p4, p2))]
                .into_iter()
                .flatten()
        }
    }
}
pub struct TownUI;
impl UI for TownUI {
    type Pos = BuildingPosition;

    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
    ) -> impl Bundle {
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(RegularPolygon::new(7.0, 3));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(color.to_bevy_color())),
            Transform::from_xyz(x * 77.0, y * 77., 0.0),
        )
    }

    fn resources() -> Resources {
        TOWN_RESOURCES
    }
}
