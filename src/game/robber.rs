use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    window,
};
use itertools::Itertools;

use crate::{
    game::PlayerHandle,
    utils::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
};

use super::{
    Building, GameState,
    colors::{CatanColor, CatanColorRef, CurrentColor},
    common_ui::{self, SpinnerButtonInteraction, Value},
    positions::{BuildingPosition, FPosition, Position, generate_postions},
    resources::{self, Resources, take_resource},
    resources_management::{self, ResourceRef},
};

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Robber(pub Position);
impl Default for Robber {
    fn default() -> Self {
        Self(Position { q: 0, r: 0, s: 0 })
    }
}

pub fn counter_text_update(
    mut interaction_query: Query<'_, '_, (&mut Text, &Value<RobberResourceSpinner>)>,
    counter_query: Query<'_, '_, &Resources, Changed<Resources>>,
    robber_spinner_query: Query<'_, '_, &RobberResourceSpinner>,
) {
    for (mut text, resources) in &mut interaction_query {
        if let Ok(resources) = robber_spinner_query.get(resources.0)
            && let Ok(counter) = counter_query.get(resources.resource.0)
        {
            **text = counter.get(resources.resource.1).to_string();
        }
    }
}
#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub struct RobberButton;
pub fn place_robber(mut commands: Commands<'_, '_>, robber: Res<'_, Robber>) {
    generate_postions(3)
        // TODO: skip current robber pos
        .filter(|p| *p != robber.0)
        .map(|p| {
            let pos: FPosition = p.into();
            let (x, y) = pos.hex_to_pixel();
            (x, y, p)
        })
        .for_each(|(x, y, p)| {
            // add button with positonn and RobberPosition struct
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
                    RobberButton,
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        left: Val::Px(x * 77.),
                        bottom: Val::Px(y * 77.),
                        ..default()
                    },
                    p,
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )],
            ));
        });

    // show ui to place robber
    // on every hex besides for current robber hex
    // the make interaction function, that when clicked:
    // 1) moves the robber there, set the robber postion
    // 2) tries to take a resource from other player, or show ui to choose which player to pick
    //    from
}
pub fn place_robber_interaction(
    mut robber_places_query: Query<
        '_,
        '_,
        (&Interaction, &Position, &mut Button, &mut BackgroundColor),
        (Changed<Interaction>, With<RobberButton>),
    >,
    building_q: Query<'_, '_, (&ChildOf, &CatanColor, &'_ BuildingPosition), With<Building>>,
    current_color: Res<'_, CurrentColor>,
    mut robber: ResMut<'_, Robber>,
    player_resources: Query<'_, '_, (&CatanColor, &mut Resources, &PlayerHandle)>,
    commands: Commands<'_, '_>,
    state: ResMut<'_, NextState<GameState>>,
) {
    for (interaction, position, mut button, mut color) in &mut robber_places_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.set_changed();
                *robber = Robber(*position);
                choose_player_to_take_from(
                    position,
                    *current_color,
                    building_q,
                    player_resources,
                    commands,
                    state,
                );
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
#[derive(Component)]
pub struct RobberChooseColorButton;
fn choose_player_to_take_from<'a>(
    position: &Position,
    color: CurrentColor,
    building_q: Query<'_, '_, (&ChildOf, &CatanColor, &'_ BuildingPosition), With<Building>>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources, &PlayerHandle)>,
    mut commands: Commands<'_, '_>,
    mut state: ResMut<'_, NextState<GameState>>,
) {
    // TODO: eventually buildings/roads will be linked to the main player entity, at which point
    // find with color won't be needed
    let mut colors = building_q
        .iter()
        .filter_map(|(p, c, b)| {
            (c != &color.0.color
                && b.contains(position)
                // we check that are enough resources to steal instead of later on, becuase if
                // there are no one to steal from them we need to go back to turn, and its much
                // easer to check that here than later on espicially if there are mutlitple players
                // surrounding the hex
                && player_resources.get(p.parent()).ok().is_some_and(|r| r.1.count() > 0))
            .then_some(CatanColorRef {
                entity: p.parent(),
                color: *c,
                handle: *player_resources.get(p.parent()).unwrap().2,
            })
        })
        .unique()
        .collect_vec();
    if colors.len() == 1 {
        let other_color = colors.remove(0);
        let (_, mut other_color_resources, _) =
            player_resources.get_mut(other_color.entity).unwrap();
        let put_resources = take_resource(&mut other_color_resources);
        let mut current_resources = player_resources.get_mut(color.0.entity).unwrap();
        put_resources(&mut current_resources.1);

        // either we are coming from roll(7) or in middle of turn(dev card) but we always go back to
        // turn
        state.set(GameState::Turn);
    } else if colors.is_empty() {
        // if no one to steal from go to turn

        // either we are coming from roll(7) or in middle of turn(dev card) but we always go back to
        // turn
        state.set(GameState::Turn);
    } else {
        // show options of how to pick from
        for (i, color) in colors.iter().enumerate() {
            commands.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    Button,
                    Node {
                        width: Val::Px(25.0),
                        height: Val::Px(25.0),
                        bottom: Val::Px(35.),
                        left: Val::Px((i * 30) as f32),
                        ..default()
                    },
                    RobberChooseColorButton,
                    *color,
                    BorderRadius::MAX,
                    BackgroundColor(color.to_bevy_color()),
                )],
            ));
        }
        state.set(GameState::RobberPickColor);
    }
}

pub fn choose_player_to_take_from_interaction(
    current_color: Res<'_, CurrentColor>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
    mut robber_taking_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &CatanColorRef,
            &mut Button,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<RobberChooseColorButton>),
    >,
    mut state: ResMut<'_, NextState<GameState>>,
) {
    for (interaction, color, mut button, mut button_color) in &mut robber_taking_query {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();

                let (_, mut other_color_resources) =
                    player_resources.get_mut(color.entity).unwrap();
                let put_resources = take_resource(&mut other_color_resources);
                let mut current_resources =
                    player_resources.get_mut(current_color.0.entity).unwrap();
                put_resources(&mut current_resources.1);
                // either we are coming from roll(7) or in middle of turn(dev card) but we always go back to
                // turn
                state.set(GameState::Turn);
                break;
            }
            Interaction::Hovered => {
                *button_color = (if color.color == CatanColor::White {
                    button_color.0.darker(0.2)
                } else {
                    button_color.0.lighter(0.1)
                })
                .into();

                button.set_changed();
            }
            Interaction::None => {
                *button_color = color.to_bevy_color().into();
            }
        }
    }
}

#[derive(Component)]
pub struct WindowRef(Entity);

pub fn take_extra_resources(
    mut commands: Commands<'_, '_>,
    player_resources: Query<'_, '_, (Entity, &CatanColor, &mut Resources)>,
    mut left: ResMut<'_, PreRobberDiscardLeft>,
) {
    player_resources
        .iter()
        .filter(|resources| resources.2.count() > 7)
        .for_each(|r| {
            left.0 += 1;
            let window = commands
                .spawn(Window {
                    title: format!("{:?}", r.0),
                    ..default()
                })
                .id();
            let camera = commands
                .spawn((
                    Camera2d,
                    Camera {
                        target: RenderTarget::Window(window::WindowRef::Entity(window)),
                        ..default()
                    },
                    RenderLayers::layer(1),
                ))
                .id();

            setup_take_extra_resources(&mut commands, camera, window, *r.2, r.0, r.2.count() / 2);
        });
}

#[derive(Resource)]
pub struct PreRobberDiscardLeft(pub u8);
pub fn counter_sumbit_interaction(
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &WindowRef,
            &SumbitButton,
        ),
        Changed<Interaction>,
    >,
    mut commands: Commands<'_, '_>,
    mut left: ResMut<'_, PreRobberDiscardLeft>,
    mut state: ResMut<'_, NextState<GameState>>,
    counter_query: Query<'_, '_, &Resources>,
) {
    for (interaction, mut button, mut color, window, max) in &mut interaction_query {
        if let Ok(resource) = counter_query.get(max.resource_ref)
            && resource.count() == max.new_max_resources
        {
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    commands.entity(window.0).despawn();
                    left.0 -= 1;
                    if left.0 == 0 {
                        state.set(GameState::PlaceRobber);
                    }
                    button.set_changed();
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                    button.set_changed();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                    button.set_changed();
                }
            }
        }
    }
}

fn setup_take_extra_resources(
    commands: &mut Commands<'_, '_>,
    camera: Entity,
    window: Entity,
    resources: Resources,
    resources_entity: Entity,
    resources_needed: u8,
) {
    let spawn_related_bundle = children![
        resource_slider(
            commands,
            ResourceRef(resources_entity, resources::Resource::Wood),
            resources.wood,
        ),
        resource_slider(
            commands,
            ResourceRef(resources_entity, resources::Resource::Brick,),
            resources.brick,
        ),
        resource_slider(
            commands,
            ResourceRef(resources_entity, resources::Resource::Sheep,),
            resources.sheep,
        ),
        resource_slider(
            commands,
            ResourceRef(resources_entity, resources::Resource::Wheat),
            resources.wheat,
        ),
        resource_slider(
            commands,
            ResourceRef(resources_entity, resources::Resource::Ore,),
            resources.ore,
        ),
        (
            Node {
                display: Display::Grid,
                align_self: AlignSelf::Center,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            Button,
            SumbitButton {
                new_max_resources: resources_needed,
                resource_ref: resources_entity,
            },
            BackgroundColor(NORMAL_BUTTON),
            BorderColor::all(Color::BLACK),
            WindowRef(window),
            Text::new("confirm".to_string()),
        ),
    ];
    commands.spawn((
        UiTargetCamera(camera),
        Node {
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::min_content(),
                GridTrack::flex(1.),
            ],

            ..default()
        },
        spawn_related_bundle,
    ));
}
#[derive(Component)]
pub struct SumbitButton {
    new_max_resources: u8,
    resource_ref: Entity,
}
#[derive(Debug, Component, Clone, Copy)]
pub struct RobberResourceSpinner {
    resource: ResourceRef,
    max: u8,
}
impl SpinnerButtonInteraction<RobberResourceSpinner> for Query<'_, '_, &'static mut Resources> {
    fn increment(&mut self, resource: &RobberResourceSpinner) {
        if let Ok(mut resources) = self.get_mut(resource.resource.0) {
            *resources.get_mut(resource.resource.1) += 1;
        }
    }
    fn decrement(&mut self, resource: &RobberResourceSpinner) {
        if let Ok(mut resources) = self.get_mut(resource.resource.0) {
            *resources.get_mut(resource.resource.1) -= 1;
        }
    }

    fn can_increment(&mut self, resource: &RobberResourceSpinner) -> bool {
        if let Ok(resources) = self.get(resource.resource.0) {
            resources.get(resource.resource.1) < resource.max
        } else {
            false
        }
    }
    fn can_decrement(&mut self, resource: &RobberResourceSpinner) -> bool {
        if let Ok(resources) = self.get(resource.resource.0) {
            resources.get(resource.resource.1) > 0
        } else {
            false
        }
    }
}
fn resource_slider(
    commands: &mut Commands<'_, '_>,
    resource: resources_management::ResourceRef,
    max: u8,
) -> impl Bundle {
    let entity = commands.spawn(RobberResourceSpinner { resource, max }).id();
    common_ui::spinner_bundle::<RobberResourceSpinner>(
        entity,
        Text::new(format!("{:?}", resource.1)),
    )
}
