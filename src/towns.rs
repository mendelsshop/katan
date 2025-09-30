use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    BoardSize, Building, GameState, Left, Port, UI, VictoryPoints,
    colors::{CatanColor, CurrentColor, CurrentSetupColor, NORMAL_BUTTON},
    common_ui::ButtonInteraction,
    positions::{BuildingPosition, RoadPosition},
    resources::{Resources, TOWN_RESOURCES},
    roads::{RoadQuery, RoadQueryItem},
    setup_game::Ports,
};

#[derive(Component, Clone, Copy, Debug)]
pub struct TownPlaceButton(Resources, BuildingPosition);
#[derive(Debug, Component, Clone, Copy, Default)]
#[require(Building)]
pub struct Town;
pub fn place_normal_town(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    town_free_q: Query<'_, '_, &Left<Town>, With<CatanColor>>,
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
                    TownPlaceButton(TOWN_RESOURCES, p),
                    Button,
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
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
        get_possible_town_placements(color_r.0.color, BoardSize(size_r.0), road_q, building_q);
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
                    TownPlaceButton(Resources::default(), p),
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
#[derive(SystemParam)]
pub struct PlaceTownButtonState<'w, 's, C: Resource> {
    resources: ResMut<'w, Resources>,
    game_state: Res<'w, State<GameState>>,
    game_state_mut: ResMut<'w, NextState<GameState>>,
    color_r: Res<'w, C>,
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    kind_free_ports_and_resources_q: Query<
        'w,
        's,
        (
            &'static mut Resources,
            &'static mut Ports,
            &'static mut Left<Town>,
            &'static mut VictoryPoints,
        ),
        With<CatanColor>,
    >,
    ports: Query<'w, 's, (&'static BuildingPosition, &'static Port)>,
}
impl<C: Resource> ButtonInteraction<TownPlaceButton> for PlaceTownButtonState<'_, '_, C>
where
    CatanColor: From<C>,
    bevy::prelude::Entity: From<C>,
    C: Copy,
{
    fn interact(&mut self, TownPlaceButton(cost, position): &TownPlaceButton) {
        let PlaceTownButtonState {
            resources,
            game_state,
            game_state_mut,
            color_r,
            commands,
            meshes,
            materials,
            kind_free_ports_and_resources_q,
            ports,
        } = self;

        let color_r: &C = &color_r;
        let current_color: CatanColor = (*color_r).into();
        let current_color_entity: Entity = (*color_r).into();
        commands
            .entity(current_color_entity)
            .with_child((Town, current_color, *position));
        let player_resources_ports_and_towns_left = kind_free_ports_and_resources_q
            .get_mut(current_color_entity)
            .ok();
        if let Some((mut resources, mut player_ports, mut left, mut points)) =
            player_resources_ports_and_towns_left
        {
            *resources -= *cost;
            left.0 -= 1;
            points.actual += 1;

            if let Some((_, port)) = ports
                .iter()
                .find(|(port_position, _)| port_position == &position)
            {
                *player_ports += *port;
            }
        }
        **resources += *cost;
        match *game_state.get() {
            GameState::Nothing
            | GameState::Monopoly
            | GameState::YearOfPlenty
            | GameState::Start
            | GameState::Roll
            | GameState::Turn
            | GameState::PlaceRobber
            | GameState::RobberDiscardResources
            | GameState::RoadBuilding
            | GameState::PlaceRoad
            | GameState::PlaceCity
            | GameState::SetupRoad
            | GameState::RobberPickColor => {}
            GameState::PlaceTown => {
                game_state_mut.set(GameState::Turn);
            }

            GameState::SetupTown => game_state_mut.set(GameState::SetupRoad),
        }
        commands.spawn(TownUI::bundle(*position, meshes, materials, current_color));
    }
}
