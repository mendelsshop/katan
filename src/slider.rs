use crate::{
    Building, GameState,
    colors::{
        CatanColor, CatanColorRef, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON,
    },
    common_ui::ButtonInteraction,
    positions::{BuildingPosition, FPosition, Position, generate_postions},
    resources::{self, Resources, take_resource},
    resources_management::{self, ResourcesRef},
};
use bevy::{
    ecs::system::{SystemParam, SystemParamItem},
    prelude::*,
};

#[derive(Component)]
pub struct Value;
pub fn slider_bundle(
    resource_count: u8,
    resources: Entity,
    specific_resource: resources::Resource,
    set_max: bool,
) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            margin: UiRect::all(Val::Px(3.0)),
            grid_template_rows: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BorderColor(Color::BLACK),
        children![
            (
                UpButton {
                    max_individual: set_max.then_some(resource_count),
                },
                Node {
                    display: Display::Grid,
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                Button,
                BackgroundColor(NORMAL_BUTTON),
                Text::new("+".to_string()),
                ResourcesRef(resources, specific_resource),
            ),
            (
                Node {
                    justify_self: JustifySelf::Center,
                    display: Display::Grid,
                    ..default()
                },
                Value,
                Text::new(resource_count.to_string()),
                ResourcesRef(resources, specific_resource),
            ),
            (
                DownButton,
                Node {
                    display: Display::Grid,
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                Button,
                BackgroundColor(NORMAL_BUTTON),
                Text::new("-".to_string()),
                ResourcesRef(resources, specific_resource),
            )
        ],
    )
}
pub fn slider_down_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &ResourcesRef,
        ),
        (Changed<Interaction>, With<DownButton>),
    >,
    mut counter_query: Query<'_, '_, &mut Resources>,
) {
    for (interaction, mut button, mut color, resources) in &mut interaction_query {
        if let Ok(resource) = counter_query
            .get_mut(resources.0)
            .map(bevy::prelude::Mut::into_inner)
            .map(|r| r.get_mut(resources.1))
            && *resource > 0
        {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    *resource -= 1;
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
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}
pub fn slider_up_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &ResourcesRef,
            &UpButton,
        ),
        (Changed<Interaction>,),
    >,

    mut counter_query: Query<'_, '_, &mut Resources>,
) {
    for (interaction, mut button, mut color, resources, max) in &mut interaction_query {
        if let Ok(resource) = counter_query
            .get_mut(resources.0)
            .map(bevy::prelude::Mut::into_inner)
            .map(|r| r.get_mut(resources.1))
            && max.max_individual.is_none_or(|max| *resource < max)
        {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    *resource += 1;
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
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}
#[derive(Component)]
pub struct UpButton {
    max_individual: Option<u8>,
}
pub trait UpButtonTrait: Component {
    // const CAN_INCREMENT: impl System;
    fn increment(&mut self) -> impl System<Out = bool>;
}
#[derive(Component)]
pub struct DownButton;
pub fn counter_text_update(
    mut interaction_query: Query<'_, '_, (&mut Text, &ResourcesRef), With<Value>>,
    counter_query: Query<'_, '_, &Resources>,
) {
    for (mut text, resources) in &mut interaction_query {
        if let Ok(counter) = counter_query.get(resources.0) {
            **text = counter.get(resources.1).to_string();
        }
    }
}
