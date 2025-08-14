use bevy::prelude::*;

use crate::GameState;

pub fn robber(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::PlaceRobber);
}

#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::RoadBuilding)]

pub(crate) enum RoadBuildingState {
    #[default]
    Road1,
    Road2,
}
pub fn road_building(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::RoadBuilding);
}
pub fn monopoly() {}
pub fn year_of_plenty() {}
