use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    prelude::*,
};
use bevy_ggrs::AddRollbackCommandExtension;
use itertools::Itertools;

use crate::utils::NORMAL_BUTTON;

use super::{
    BoardSize, Building, GameState, Input, Left, UI,
    colors::{CatanColor, CurrentColor},
    common_ui::ButtonInteraction,
    development_card_actions::RoadBuildingState,
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

#[derive(Component, Clone, Copy, Debug)]
pub struct RoadPlaceButton(Resources, RoadPosition);
#[derive(Debug, Component, Clone, Copy, Default)]
pub struct Road;
/// if `RESOURCE_MULTIPLIER` is zero then its free (default is 1, normal price)
pub fn place_normal_road<const RESOURCE_MULTIPLIER: u8>(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    size_r: Res<'_, BoardSize>,
    road_free_q: Query<'_, '_, &Left<Road>, With<CatanColor>>,
    road_q: Query<'_, '_, RoadQuery>,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut game_state: ResMut<'_, NextState<GameState>>,
) {
    let unplaced_roads_correct_color = road_free_q.get(color_r.0.entity).ok();

    // no roads to place
    let Some(_) = unplaced_roads_correct_color.filter(|r| r.0 > 0) else {
        return;
    };

    let (current_color_roads, _): (Vec<_>, Vec<_>) =
        road_q.into_iter().partition(|r| *r.1 == color_r.0.color);

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
            RoadPosition::Both(position, position1, _) => unsafe {
                BuildingPosition::new_unchecked(*road1, *position, *position1)
            },
        };
        !building_q.any(|bp| &road_intersection == bp)
    }
    let possible_roads = possible_roads.filter(|r| {
        filter_by_building(
            r,
            building_q
                .iter()
                .filter_map(|(_, color, pos)| Some(pos).filter(|_| *color != color_r.0.color)),
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
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    RoadPlaceButton(ROAD_RESOURCES * RESOURCE_MULTIPLIER, p,),
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
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    RoadPlaceButton(Resources::default(), p,),
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
        println!("new row ui bundle");
        let (x, y) = pos.positon_to_pixel_coordinates();
        let mesh1 = meshes.add(Rectangle::new(10.0, 70.));
        (
            Mesh2d(mesh1),
            MeshMaterial2d(materials.add(color.to_bevy_color())),
            Transform::from_xyz(x * 77.0, y * 77., 0.0).with_rotation(Quat::from_rotation_z(
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
#[derive(SystemParam)]
pub struct PlaceRoadButtonState<'w, 's, C: Resource> {
    resources: ResMut<'w, Resources>,
    game_state: Res<'w, State<GameState>>,
    game_state_mut: ResMut<'w, NextState<GameState>>,
    color_r: Res<'w, C>,
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    kind_free_and_resources_q:
        Query<'w, 's, (&'static mut Resources, &'static mut Left<Road>), With<CatanColor>>,

    input: ResMut<'w, Input>,
    substate_mut: Option<ResMut<'w, NextState<RoadBuildingState>>>,
    substate: Option<Res<'w, State<RoadBuildingState>>>,
}
impl<C: Resource> ButtonInteraction<RoadPlaceButton> for PlaceRoadButtonState<'_, '_, C>
where
    CatanColor: From<C>,
    bevy::prelude::Entity: From<C>,
    C: Copy,
{
    fn interact(&mut self, RoadPlaceButton(cost, position): &RoadPlaceButton) {
        let PlaceRoadButtonState {
            resources,
            game_state,
            game_state_mut,
            color_r,
            commands,
            meshes,
            materials,
            kind_free_and_resources_q,
            substate_mut,
            substate,
            input,
        } = self;

        let color_r: &C = color_r;
        let current_color: CatanColor = (*color_r).into();
        let current_color_entity: Entity = (*color_r).into();
        let road = commands
            .spawn((Road, current_color, *position))
            .add_rollback()
            .id();
        println!("set input");
        // updating of the input should happen on the fly
        **input = Input::AddRoad(current_color_entity, *position, *cost);
        commands.entity(current_color_entity).add_child(road);
        let kind_left = kind_free_and_resources_q.get_mut(current_color_entity).ok();
        if let Some((mut resources, mut left)) = kind_left {
            *resources -= *cost;
            left.0 -= 1;
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
            | GameState::SetupTown
            | GameState::PlaceTown
            | GameState::PlaceCity
            | GameState::NotActive
            | GameState::NotActiveSetup
            | GameState::RobberPickColor => {}

            GameState::PlaceRoad => {
                game_state_mut.set(GameState::Turn);
            }
            GameState::RoadBuilding => {
                if let Some((substate_mut, substate)) =
                    substate_mut.as_deref_mut().zip(substate.as_ref())
                {
                    if *substate.get() == RoadBuildingState::Road1 {
                        substate_mut.set(RoadBuildingState::Road2);
                    } else {
                        game_state_mut.set(GameState::Turn);
                    }
                }
            }

            GameState::SetupRoad => game_state_mut.set(GameState::SetupTown),
        }
        // commands.spawn(RoadUI::bundle(*position, meshes, materials, current_color));
    }
}
