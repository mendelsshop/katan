use bevy::prelude::*;

use crate::{
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    resources::{DEVELOPMENT_CARD_RESOURCES, Resources},
    turn_ui::DevelopmentCardButton,
};

#[derive(Debug, Component, Clone, Copy)]
pub enum DevelopmentCard {
    Knight,
    Monopoly,
    YearOfPlenty,
    RoadBuilding,
    VictoryPoint,
}
pub fn buy_development_card_interaction(
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    mut free_dev_cards: Query<'_, '_, (Entity, &DevelopmentCard), Without<CatanColor>>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    mut resources: ResMut<'_, Resources>,
    interaction_query: Single<
        '_,
        (
            &DevelopmentCardButton,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
        ),
        Changed<Interaction>,
    >,
) {
    let player_resources = player_resources.iter_mut().find(|x| x.1 == &color_r.0);
    if let Some((mut player_resources, _)) = player_resources {
        let required_resources = DEVELOPMENT_CARD_RESOURCES;
        if !player_resources.contains(required_resources) {
            return;
        }
        let (_, interaction, mut color, mut button) = interaction_query.into_inner();
        match *interaction {
            Interaction::Pressed => {
                *player_resources -= required_resources;
                *resources += required_resources;

                *color = PRESSED_BUTTON.into();

                if let Some(card) = free_dev_cards.iter_mut().next() {
                    commands.entity(card.0).insert(color_r.0);
                }

                button.set_changed();
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
