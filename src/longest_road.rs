use crate::{
    Building, VictoryPoints,
    colors::{CatanColor, CurrentColor},
    positions::{BuildingPosition, RoadPosition},
    roads::RoadQuery,
};
use bevy::{platform::collections::HashSet, prelude::*};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Resource)]
struct LongestRoad(Entity, u8);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Component)]
pub struct LongestRoadRef;
pub struct LongestRoadPlugin;
/// the players longest path (not if the player has longest road)
#[derive(Clone, PartialEq, Eq, Debug, Component)]
// TODO: maybe precompute len
pub struct PlayerLongestRoad(pub HashSet<RoadPosition>);
impl Plugin for LongestRoadPlugin {
    fn build(&self, app: &mut App) {
        // start at 2 so when someone gets 3 it will be updated
        app.insert_resource(LongestRoad(Entity::PLACEHOLDER, 4));
        app.add_systems(Update, longest_road_road_added);
        app.add_systems(Update, longest_road_town_added);
    }
}

// whenever a road is added the players longest road is recalulated
fn longest_road_road_added(
    road_q: Query<'_, '_, RoadQuery>,
    road_q_changed: Query<'_, '_, RoadQuery, Changed<RoadPosition>>,
    building_q: Query<'_, '_, (&'_ Building, &CatanColor, &'_ BuildingPosition)>,
    mut player_q: Query<'_, '_, (Entity, &mut VictoryPoints, &mut PlayerLongestRoad)>,
    mut current: ResMut<'_, LongestRoad>,
    mut commmands: Commands<'_, '_>,
    color: Res<'_, CurrentColor>,
) {
    // we could shortcut here if its current player who has longest road, but then total count
    // wouldn't be accurate
    if road_q_changed.iter().count() > 0 {
        return;
    }

    let roads_by_color = road_q.iter().filter(|q| *q.1 == color.0.color);
    let longest_road = longest_road(
        roads_by_color.collect_vec(),
        building_q.into_iter().filter(|b| *b.1 == color.0.color),
    );

    if let Some(new) = longest_road {
        if let Ok(mut player) = player_q.get_mut(color.0.entity) {
            let len = new.len();
            player.2.0 = new;
            if len as u8 > current.1 {
                commmands.entity(player.0).insert(LongestRoadRef);
                player.1.actual += 2;
                if let Ok(mut player) = player_q.get_mut(current.0) {
                    commmands.entity(player.0).remove::<LongestRoadRef>();
                    player.1.actual -= 2
                }
                *current = LongestRoad(color.0.entity, len as u8);
            }
        }
    }
}

fn longest_road<'a, 'b, 'c>(
    roads: Vec<crate::roads::RoadQueryItem<'_>>,
    buildings: impl Iterator<Item = (&'a Building, &'b CatanColor, &'c BuildingPosition)>,
) -> Option<HashSet<RoadPosition>> {
    // skip anyone how has road count equal to current longest road (if check_cut_off)
    // always check if less roads then 3
    // road cannot be used twice (so no loops in actual longest road)
    if roads.len() <= 4 {
        None
    } else {
        roads.iter().fold(
            // basically strongly connected componenets
            // only problem, is if you have loop with (other color) house in between it depeends
            // how the iteration
            vec![],
            |mut init: Vec<HashSet<_>>, road| {
                if let Some(i) = init.iter_mut().find(|x| false) {
                    i.insert(*road.2);
                } else {
                    init.push(HashSet::from([*road.2]));
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
    mut player_q: Query<
        '_,
        '_,
        (
            Entity,
            &mut VictoryPoints,
            &mut PlayerLongestRoad,
            &CatanColor,
        ),
    >,
    mut current: ResMut<'_, LongestRoad>,
    mut commmands: Commands<'_, '_>,
) {
    if current.0 == Entity::PLACEHOLDER || building_q_changed.iter().count() > 0 {
        return;
    }
    if let Some(mut current_interupted) =
        player_q.iter_mut().find(|player| {
            player.2.0.iter().tuple_combinations().any(|(r1, r2)| {
                todo!("merge roads into building postion and see if its new buildings make sure color mismatch between building and road)")
            })
        })
    {
        // updated interupted players longest road count
        let roads_by_color = road_q.iter().filter(|q| q.0.parent() == current.0);
        let new = longest_road(
            roads_by_color.into_iter().unzip::<_, _, Vec<_>, Vec<_>>().1,
            building_q
                .into_iter()
                .filter(|b| b.1 == current_interupted.3),
        );
        if let Some(new) = new {
            // update current holders longest road count
            current_interupted.2.0 = new
        }
        // if interuppted is the current holder than update longest road
        if current_interupted.0 == current.0 {
            // possible cases:
            // Here we have to distinguish between three cases:

            // If the player who up to this point had the Longest Road still meets the requirements for the Longest Road (either alone or together with another player), he keeps the card.
            //
            // If another player now meets the requirements for the Longest Road, he receives the card.
            //
            // If none of the players - or more than one player - meets the requirements for the Longest Road, none of the players receives the card.

            // we deducat the points here even if the current holder still holds it to get
            // around mutablitity issues
            current_interupted.1.actual -= 2;
            let mut new = player_q.iter_mut().max_set_by_key(|p| p.2.0.len());

            if let Some(mut new_current) = new.pop_if(|(e, _, _, _)| *e == current.0) {
                // same player but less roads (ussually)
                current.1 = new_current.2.0.len() as u8;

                new_current.1.actual += 2;
            } else {
                commmands.entity(current.0).remove::<LongestRoadRef>();
                if let [new_player] = &mut new[..] {
                    // no tie for second
                    *current = LongestRoad(new_player.0, new_player.2.0.len() as u8);
                    commmands.entity(new_player.0).insert(LongestRoadRef);
                    new_player.1.actual += 2
                } else {
                    // tie for longest road (not including current longest road holder)
                    *current = LongestRoad(Entity::PLACEHOLDER, 4);
                }
            }
        }
    }
}
