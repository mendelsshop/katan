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
    town_free_q: Query<'_, '_, (&Town, &CatanColor, &Left)>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &'_ CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_towns_correct_color = town_free_q.iter().find(|r| r.1 == &color_r.0);

    // no towns to place
    let Some(_) = unplaced_towns_correct_color.filter(|r| r.2.0 > 0) else {
        return;
    };

    let possible_towns =
        get_possible_town_placements(color_r.0, BoardSize(size_r.0), road_q, building_q);
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
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        left: Val::Px(x * 28.),
                        bottom: Val::Px(y * 28.),
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

    let filter_by_building =
        move |position: &BuildingPosition,
              building_q: Query<'_, '_, (&_, &CatanColor, &BuildingPosition)>| {
            match position {
                BuildingPosition::All(position, position1, position2) => !buildings_on_roads(
                    [
                        RoadPosition::new(*position, *position1, Some(size_r.0)),
                        RoadPosition::new(*position, *position2, Some(size_r.0)),
                        RoadPosition::new(*position1, *position2, Some(size_r.0)),
                    ]
                    .into_iter()
                    .flatten(),
                    BoardSize(size_r.0),
                )
                .any(|p| building_q.iter().any(|(_, _, place_b)| &p == place_b)),
            }
        };

    possibles_towns.filter(move |r| filter_by_building(r, building_q))
}
fn buildings_on_roads(
    current_color_roads: impl Iterator<Item = RoadPosition>,
    size_r: BoardSize,
) -> impl Iterator<Item = BuildingPosition> {
    current_color_roads.flat_map(move |road| match road {
        RoadPosition::Both(p1, p2, _) => {
            let (p3, p4) = road.neighboring_two(Some(size_r.0));
            let make_town_pos = |p, option_p1: Option<_>, p2| {
                option_p1.and_then(|p1| BuildingPosition::new(p, p1, p2, Some(size_r.0)))
            };
            [(make_town_pos(p1, p3, p2)), (make_town_pos(p1, p4, p2))]
                .into_iter()
                .flatten()
        }
    })
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
            Transform::from_xyz(x * 28.0, y * 28., 0.0),
        )
    }

    fn resources() -> Resources {
        TOWN_RESOURCES
    }
}
