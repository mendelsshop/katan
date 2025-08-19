use bevy::prelude::*;

use crate::{
    Layout,
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    resources::{self, Resources},
};
pub fn show_player_resources(
    player_dev_cards: Query<'_, '_, (&CatanColor, &Resources), Changed<Resources>>,
    res: Res<'_, CurrentColor>,
    commands: Commands<'_, '_>,
) {
    if let Some(resources) = crate::find_with_color(&res.0, player_dev_cards.iter()) {}
}
pub fn setup_players_resources(mut commands: Commands<'_, '_>, layout: Res<'_, Layout>) {
    let trade_resources = Resources::default();
    let resources_entity = commands.spawn(trade_resources).id();
    commands.entity(layout.resources).with_child((
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(10.), GridTrack::percent(90.)],

            ..default()
        },
        children![
            Node {
                display: Display::Grid,
                ..default()
            },
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::percent(20.),
                        GridTrack::percent(20.),
                        GridTrack::percent(20.),
                        GridTrack::percent(20.),
                        GridTrack::percent(20.)
                    ],

                    ..default()
                },
                children![
                    slider_bundle(0, resources_entity, resources::Resource::Wood, false,),
                    slider_bundle(0, resources_entity, resources::Resource::Brick, false,),
                    slider_bundle(0, resources_entity, resources::Resource::Sheep, false,),
                    slider_bundle(0, resources_entity, resources::Resource::Wheat, false,),
                    slider_bundle(0, resources_entity, resources::Resource::Ore, false,),
                ]
            )
        ],
    ));
}
#[derive(Component)]
pub struct UpButton {
    max_individual: Option<u8>,
}
#[derive(Component)]
pub struct DownButton;

#[derive(Component)]
pub struct ResourcesRef(pub Entity, pub resources::Resource);
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
