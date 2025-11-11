use bevy::prelude::*;

use super::{KatanComponent, Knights, VictoryPoints};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Resource)]
struct LargetArmy(pub u8, pub Entity);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Component)]
#[require(KatanComponent)]
pub struct LargetArmyRef;

fn update_larget_army(
    players: Query<'_, '_, (Entity, &Knights), Changed<Knights>>,
    mut points: Query<'_, '_, &mut VictoryPoints>,
    mut current_largest_army: ResMut<'_, LargetArmy>,
    mut commands: Commands<'_, '_>,
) {
    for (entity, knights) in players {
        if knights.0 > current_largest_army.0 {
            if current_largest_army.1 != Entity::PLACEHOLDER {
                commands
                    .entity(current_largest_army.1)
                    .remove::<LargetArmyRef>();
                if let Ok(mut points) = points.get_mut(current_largest_army.1) {
                    points.actual -= 2;
                }
            }
            current_largest_army.0 = knights.0;
            current_largest_army.1 = entity;
            commands.entity(entity).insert(LargetArmyRef);
            if let Ok(mut points) = points.get_mut(entity) {
                points.actual += 2;
            }
        }
    }
}

pub struct LargestArmyPlugin;
impl Plugin for LargestArmyPlugin {
    fn build(&self, app: &mut App) {
        // start at two so when there is 3 it will be updated
        app.insert_resource(LargetArmy(2, Entity::PLACEHOLDER))
            .add_systems(Update, update_larget_army);
    }
}
