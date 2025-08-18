use std::mem;

use bevy::prelude::*;

use crate::{
    GameState,
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, PRESSED_BUTTON},
    find_with_color,
    resources::{self, Resources},
};

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
pub fn monopoly(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::RoadBuilding);
}
#[derive(Component, Debug, Clone, Copy)]
pub struct MonopolyButton(resources::Resource);
pub fn monopoly_setup(mut commands: Commands<'_, '_>) {
    [
        resources::Resource::Wood,
        resources::Resource::Brick,
        resources::Resource::Sheep,
        resources::Resource::Wheat,
        resources::Resource::Ore,
    ]
    .iter()
    .enumerate()
    .for_each(|(i, r)| {
        commands.spawn((
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

                    bottom: Val::Px(35.),
                    left: Val::Px((i * 30) as f32),
                    ..default()
                },
                MonopolyButton(*r),
                BorderRadius::MAX,
                BackgroundColor(r.color()),
            )],
        ));
    });
}
pub(crate) fn monopoly_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &MonopolyButton,
        ),
        (Changed<Interaction>,),
    >,

    current_color: Res<'_, CurrentColor>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
) {
    for (interaction, mut button, mut color, kind) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.set_changed();
                let taken = player_resources
                    .iter_mut()
                    .map(|mut r| {
                        let mut taken = 0;
                        let original = r.1.get_mut(kind.0);
                        mem::swap(&mut taken, original);
                        taken
                    })
                    .sum::<u8>();
                if let Some((_, mut resources)) =
                    find_with_color(&current_color.0, player_resources.iter_mut())
                {
                    // we reassign because when we go through the resources we also go through
                    // current color's resources
                    *resources.get_mut(kind.0) = taken;
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                button.set_changed();
            }
            Interaction::None => {
                *color = kind.0.color().into();
            }
        }
    }
}
#[derive(Component, Debug, Clone, Copy)]
pub struct YearOfPlentyButton(resources::Resource);
#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::YearOfPlenty)]

pub(crate) enum YearOrPlentyState {
    #[default]
    Resource1,
    Resource2,
}
pub fn year_of_plenty(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::YearOfPlenty);
}

pub fn setup_year_of_plenty(mut commands: Commands<'_, '_>) {
    [
        resources::Resource::Wood,
        resources::Resource::Brick,
        resources::Resource::Sheep,
        resources::Resource::Wheat,
        resources::Resource::Ore,
    ]
    .iter()
    .enumerate()
    .for_each(|(i, r)| {
        commands.spawn((
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

                    bottom: Val::Px(35.),
                    left: Val::Px((i * 30) as f32),
                    ..default()
                },
                YearOfPlentyButton(*r),
                BorderRadius::MAX,
                BackgroundColor(r.color()),
            )],
        ));
    });
}
pub(crate) fn year_of_plenty_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            Entity,
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &MonopolyButton,
        ),
        (Changed<Interaction>,),
    >,

    current_color: Res<'_, CurrentColor>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
    mut state: ResMut<'_, NextState<GameState>>,
    mut substate_mut: ResMut<'_, NextState<YearOrPlentyState>>,
    substate: Res<'_, State<YearOrPlentyState>>,
    mut commands: Commands<'_, '_>,
) {
    for (entity, interaction, mut button, mut color, kind) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.set_changed();
                if let Some((_, mut resources)) =
                    find_with_color(&current_color.0, player_resources.iter_mut())
                {
                    // we reassign because when we go through the resources we also go through
                    // current color's resources
                    *resources.get_mut(kind.0) += 1;
                    commands.entity(entity).despawn();
                    if *substate.get() == YearOrPlentyState::Resource1 {
                        substate_mut.set(YearOrPlentyState::Resource2);
                    } else {
                        state.set(GameState::Turn);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                button.set_changed();
            }
            Interaction::None => {
                *color = kind.0.color().into();
            }
        }
    }
}
