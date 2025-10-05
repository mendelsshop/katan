use crate::{
    Building, VictoryPoints,
    colors::{CatanColor, CurrentColor},
    positions::{BuildingPosition, RoadPosition},
    roads::RoadQuery,
};
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Resource)]
struct LongestRoad(Entity, u8);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Component)]
pub struct LongestRoadRef;
pub struct LongestRoadPlugin;
impl Plugin for LongestRoadPlugin {
    fn build(&self, app: &mut App) {
        // start at 2 so when someone gets 3 it will be updated
        app.insert_resource(LongestRoad(Entity::PLACEHOLDER, 2));
        app.add_systems(Update, longest_road_road_added);
        app.add_systems(Update, longest_road_town_added);
    }
}

fn longest_road_road_added(
    road_q: Query<'_, '_, RoadQuery>,
    road_q_changed: Query<'_, '_, RoadQuery, Changed<RoadPosition>>,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut player_q: Query<'_, '_, (Entity, &mut VictoryPoints)>,
    mut current: ResMut<'_, LongestRoad>,
    mut commmands: Commands<'_, '_>,
    color: Res<'_, CurrentColor>,
) {
    if current.0 == color.0.entity || road_q_changed.iter().count() > 0 {
        return;
    }

    let roads_by_color = road_q.iter().filter(|q| *q.1 == color.0.color);
    let longest_road = longest_road(
        roads_by_color.collect_vec(),
        current.1,
        true,
        building_q.into_iter().filter(|b| *b.1 == color.0.color),
    );

    if let Some(new) = longest_road.filter(|new| *new > current.1) {
        if let Ok(mut player) = player_q.get_mut(current.0) {
            commmands.entity(player.0).remove::<LongestRoadRef>();
            player.1.actual -= 2
        }
        *current = LongestRoad(color.0.entity, new);
        if let Ok(mut player) = player_q.get_mut(color.0.entity) {
            commmands.entity(player.0).insert(LongestRoadRef);
            player.1.actual += 1
        }
    }
}

fn longest_road<'a, 'b, 'c>(
    roads: Vec<crate::roads::RoadQueryItem<'_>>,
    current: u8,
    check_cut_off: bool,
    buildings: impl Iterator<Item = (&'a Building, &'b CatanColor, &'c BuildingPosition)>,
) -> Option<u8> {
    // skip anyone how has road count equal to current longest road (if check_cut_off)
    // always check if less roads then 3
    if roads.len() <= current as usize && check_cut_off || roads.len() <= 2 {
        None
    } else {
        roads.iter().fold(
            // basically strongly connected componenets
            // only problem, is if you have loop with (other color) house in between it depeends
            // how the iteration
            vec![],
            |mut init: Vec<Vec<_>>, road| {
                if let Some(i) = init.iter_mut().find(|x| false) {
                    i.push(road);
                } else {
                    init.push(vec![road]);
                }
                init
            },
        );
        None
    }
}
fn longest_road_town_added(
    road_q: Query<'_, '_, (&ChildOf, RoadQuery)>,
    building_q_changed: Query<
        '_,
        '_,
        (&'_ Building, &CatanColor, &'_ BuildingPosition),
        Changed<BuildingPosition>,
    >,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut player_q: Query<'_, '_, (Entity, &mut VictoryPoints)>,
    mut current: ResMut<'_, LongestRoad>,
    color: Res<'_, CurrentColor>,
    mut commmands: Commands<'_, '_>,
) {
    if current.0 == color.0.entity
        || current.0 == Entity::PLACEHOLDER
        || building_q_changed.iter().count() > 0
    {
        return;
    }
    let roads_by_color = road_q.iter().into_group_map_by(|(c, r)| (c.parent(), *r.1));

    let new = roads_by_color
        .into_iter()
        .filter_map(|((entity, color), roads)| {
            longest_road(
                roads.into_iter().unzip::<_, _, Vec<_>, Vec<_>>().1,
                current.1,
                false,
                building_q.into_iter().filter(|b| *b.1 == color),
            )
            .map(|long| (entity, long))
        })
        .max_by_key(|long| long.1);

    // possible cases:
    // house cut into current longest road (either no one has longest road (someone had 5 and it
    // got cut off) or someone has longest road, proced normally)

    if let Ok(mut player) = player_q.get_mut(current.0) {
        commmands.entity(player.0).remove::<LongestRoadRef>();
        player.1.actual -= 2
    }
    if let Some(new) = new {
        *current = LongestRoad(new.0, new.1);
        if let Ok(mut player) = player_q.get_mut(new.0) {
            commmands.entity(player.0).insert(LongestRoadRef);
            player.1.actual += 1
        }
    } else {
        *current = LongestRoad(Entity::PLACEHOLDER, 2);
    }
}
