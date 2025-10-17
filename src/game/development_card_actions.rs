use std::mem;

use bevy::prelude::*;

use crate::utils::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

use super::{
    GameState, Knights, Layout,
    colors::{CatanColor, CurrentColor},
    development_cards::{DevelopmentCard, DevelopmentCards},
    resources::{self, Resources},
};

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DevelopmentCardShow(pub u8);
pub fn development_card_action_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
            &DevelopmentCard,
        ),
        (Changed<Interaction>, With<DevelopmentCardShow>),
    >,

    mut player_dev_cards_and_knights: Query<
        '_,
        '_,
        (&mut DevelopmentCards, &mut Knights),
        With<CatanColor>,
    >,
    res: Res<'_, CurrentColor>,
    layout: Res<'_, Layout>,
    state: ResMut<'_, NextState<GameState>>,
    mut commands: Commands<'_, '_>,
) {
    let development_cards = player_dev_cards_and_knights.get_mut(res.0.entity);
    if let Ok((mut development_cards, knights)) = development_cards {
        for (entity, interaction, mut color, mut button, development_card) in &mut interaction_query
        {
            match interaction {
                Interaction::Pressed => {
                    *development_cards.get_mut(*development_card) -= 1;

                    let development_card = *development_card;
                    // little hack because this interaction is with a changed, only the
                    // interactions that are hovered over are in the query
                    // so keep the effect of change only have to do the work if its hovered over
                    // but still being able to clear the buttons we just clear all the button in
                    // the dev card part of the layout
                    commands
                        .entity(layout.development_cards)
                        .remove_recursive::<Children, (Button, Interaction)>();
                    *color = PRESSED_BUTTON.into();
                    button.set_changed();
                    if development_cards.get(development_card) == 0 {
                        println!("removing {entity} compleletly");
                        commands.entity(entity).despawn();
                    }
                    match development_card {
                        DevelopmentCard::Knight => robber(state, knights),
                        DevelopmentCard::Monopoly => monopoly(state),
                        DevelopmentCard::YearOfPlenty => year_of_plenty(state),
                        DevelopmentCard::RoadBuilding => road_building(state),
                        DevelopmentCard::VictoryPoint => unimplemented!("you cannot play a vp"),
                    }
                    break;
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                    button.set_changed();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}

pub fn robber(mut state: ResMut<'_, NextState<GameState>>, mut knights: Mut<'_, Knights>) {
    knights.0 += 1;
    state.set(GameState::PlaceRobber);
}

#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::RoadBuilding)]

pub enum RoadBuildingState {
    #[default]
    Road1,
    Road2,
}
pub fn road_building(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::RoadBuilding);
}
pub fn monopoly(mut state: ResMut<'_, NextState<GameState>>) {
    state.set(GameState::Monopoly);
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
pub fn monopoly_interaction(
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
    mut state: ResMut<'_, NextState<GameState>>,
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

                if let Ok((_, mut resources)) = player_resources.get_mut(current_color.0.entity) {
                    // we reassign because when we go through the resources we also go through
                    // current color's resources
                    *resources.get_mut(kind.0) = taken;
                }
                state.set(GameState::Turn);
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

pub enum YearOfPlentyState {
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
pub fn year_of_plenty_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &YearOfPlentyButton,
        ),
        (Changed<Interaction>,),
    >,

    current_color: Res<'_, CurrentColor>,
    mut player_resources: Query<'_, '_, &mut Resources, With<CatanColor>>,
    mut state: ResMut<'_, NextState<GameState>>,
    mut substate_mut: ResMut<'_, NextState<YearOfPlentyState>>,
    substate: Res<'_, State<YearOfPlentyState>>,
) {
    for (interaction, mut button, mut color, kind) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.set_changed();
                if let Ok(mut resources) = player_resources.get_mut(current_color.0.entity) {
                    // we reassign because when we go through the resources we also go through
                    // current color's resources
                    *resources.get_mut(kind.0) += 1;
                    if *substate.get() == YearOfPlentyState::Resource1 {
                        substate_mut.set(YearOfPlentyState::Resource2);
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
