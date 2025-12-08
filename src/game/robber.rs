use bevy::prelude::*;
use bevy_ui_anchor::{AnchorPoint, AnchorUiConfig, AnchoredUiNodes};
use itertools::Itertools;

use crate::{
    game::NeedToRoll,
    utils::{BORDER_COLOR_ACTIVE, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
};

use super::{
    Building, GameState, Input, KatanComponent, LocalPlayer, PlayerHandle,
    colors::{CatanColor, CatanColorRef, CurrentColor},
    common_ui::{self, SpinnerButtonInteraction, Value},
    positions::{BuildingPosition, FPosition, Position, generate_postions},
    resources::{self, Resources, take_resource},
};

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
#[require(KatanComponent)]
// marker component to mark the 2d mesh that represent the robber
pub struct RobberHighlighter;
#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Robber(pub Position);
impl Default for Robber {
    fn default() -> Self {
        Self(Position { q: 0, r: 0, s: 0 })
    }
}

pub fn counter_text_update(
    mut interaction_query: Query<'_, '_, (&mut Text, &Value<RobberResourceSpinner>)>,
    discard: Res<'_, RobberDiscard>,
    robber_spinner_query: Query<'_, '_, &RobberResourceSpinner>,
) {
    for (mut text, resources) in &mut interaction_query {
        if let Ok(resources) = robber_spinner_query.get(resources.0) {
            **text = discard.0.get(resources.resource).to_string();
        }
    }
}
#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
#[require(KatanComponent)]
pub struct RobberButton;
pub fn place_robber(mut commands: Commands<'_, '_>, robber: Res<'_, Robber>) {
    let multiplier = 3.0;
    generate_postions(3)
        // skip current robber pos
        .filter(|p| *p != robber.0)
        .map(|p| {
            let pos: FPosition = p.into();
            let (x, y) = pos.hex_to_pixel();
            (x, y, p)
        })
        .for_each(|(x, y, p)| {
            // add button with positonn and RobberPosition struct
            commands.spawn((
                Transform::from_xyz(x * multiplier * 25.6, y * multiplier * 25.6, 0.0),
                AnchoredUiNodes::spawn_one((
                    AnchorUiConfig {
                        anchorpoint: AnchorPoint::middle(),
                        offset: None,
                        ..Default::default()
                    },
                    Button,
                    Node {
                        width: Val::VMin(2.0),
                        height: Val::VMin(2.0),
                        ..default()
                    },
                    p,
                    RobberButton,
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )),
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
    player_resources: Query<'_, '_, (&CatanColor, &Resources, &PlayerHandle)>,
    commands: Commands<'_, '_>,
    state: ResMut<'_, NextState<GameState>>,
    input: ResMut<'_, Input>,
    still_needs_to_roll: Option<Res<'_, NeedToRoll>>,
) {
    for (interaction, position, mut button, mut color) in &mut robber_places_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.set_changed();
                choose_player_to_take_from(
                    position,
                    *current_color,
                    building_q,
                    player_resources,
                    commands,
                    state,
                    input,
                    still_needs_to_roll,
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
#[require(KatanComponent)]
pub struct RobberChooseColorButton;
fn choose_player_to_take_from(
    position: &Position,
    color: CurrentColor,
    building_q: Query<'_, '_, (&ChildOf, &CatanColor, &'_ BuildingPosition), With<Building>>,
    player_resources: Query<'_, '_, (&CatanColor, &Resources, &PlayerHandle)>,
    mut commands: Commands<'_, '_>,
    mut state: ResMut<'_, NextState<GameState>>,
    mut input: ResMut<'_, Input>,
    still_needs_to_roll: Option<Res<'_, NeedToRoll>>,
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
        let (_, other_color_resources, _) = player_resources.get(other_color.entity).unwrap();
        if let Some(resource) = take_resource(other_color_resources) {
            *input = Input::Knight(other_color.entity, resource, *position);
        }

        knight_next_time(&mut commands, &mut state, &still_needs_to_roll);
    } else if colors.is_empty() {
        *input = Input::MoveKnight(*position);
        // if no one to steal from go to turn

        knight_next_time(&mut commands, &mut state, &still_needs_to_roll);
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
                    *position,
                    BorderRadius::MAX,
                    BackgroundColor(color.to_bevy_color()),
                )],
            ));
        }
        state.set(GameState::RobberPickColor);
    }
}

fn knight_next_time(
    commands: &mut Commands<'_, '_>,
    state: &mut ResMut<'_, NextState<GameState>>,
    still_needs_to_roll: &Option<Res<'_, NeedToRoll>>,
) {
    if still_needs_to_roll.is_some() {
        commands.remove_resource::<NeedToRoll>();
        state.set(GameState::Roll);
    } else {
        state.set(GameState::Turn);
    }
}

pub fn choose_player_to_take_from_interaction(
    player_resources: Query<'_, '_, (&CatanColor, &Resources)>,
    mut robber_taking_query: Query<
        '_,
        '_,
        (
            &Interaction,
            &CatanColorRef,
            &mut Button,
            &mut BackgroundColor,
            &Position,
        ),
        (Changed<Interaction>, With<RobberChooseColorButton>),
    >,
    mut state: ResMut<'_, NextState<GameState>>,
    mut input: ResMut<'_, Input>,
    still_needs_to_roll: Option<Res<'_, NeedToRoll>>,
    mut commands: Commands<'_, '_>,
) {
    for (interaction, color, mut button, mut button_color, new_robber_positon) in
        &mut robber_taking_query
    {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();

                let (_, other_color_resources) = player_resources.get(color.entity).unwrap();
                if let Some(resource) = take_resource(other_color_resources) {
                    *input = Input::Knight(color.entity, resource, *new_robber_positon);
                }
                // either we are coming from roll(7) or in middle of turn(dev card) but we always go back to
                // turn
                //
                knight_next_time(&mut commands, &mut state, &still_needs_to_roll);
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
pub fn take_extra_resources(
    mut commands: Commands<'_, '_>,
    player_resources: Query<'_, '_, (Entity, &CatanColor, &mut Resources)>,
    local_player: Res<'_, LocalPlayer>,
) {
    if let Some(r) = player_resources
        .get(local_player.0.entity)
        .ok()
        .filter(|resources| resources.2.count() > 7)
    {
        setup_take_extra_resources(&mut commands, *r.2, r.2.count() / 2);
    }
}

pub fn counter_sumbit_interaction(
    interaction_query: Single<
        '_,
        '_,
        (
            &Interaction,
            &mut Button,
            &mut BackgroundColor,
            &ChildOf,
            &SumbitButton,
        ),
        Changed<Interaction>,
    >,
    mut commands: Commands<'_, '_>,
    mut mut_state: ResMut<'_, NextState<GameState>>,
    state: Res<'_, State<GameState>>,
    mut input: ResMut<'_, Input>,
    mut discard: ResMut<'_, RobberDiscard>,
) {
    let (interaction, mut button, mut color, discard_node, max) = interaction_query.into_inner();

    if discard.0.count() == max.new_max_resources {
        match *interaction {
            Interaction::Pressed => {
                *input = Input::RobberDiscard(discard.0);
                *discard = RobberDiscard::default();
                *color = PRESSED_BUTTON.into();
                commands.entity(discard_node.0).despawn();
                if state.get() == &GameState::RobberDiscardResourcesInActive {
                    mut_state.set(GameState::NotActive);
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

pub fn done_discarding(
    player_resources: Query<'_, '_, &mut Resources, With<CatanColor>>,
    mut mut_state: ResMut<'_, NextState<GameState>>,
) {
    if player_resources.iter().all(|r| r.count() <= 7) {
        mut_state.set(GameState::PlaceRobber);
    }
}
fn setup_take_extra_resources(
    commands: &mut Commands<'_, '_>,
    resources: Resources,
    resources_needed: u8,
) {
    let spawn_related_bundle = children![
        resource_slider(commands, resources::Resource::Wood, resources.wood),
        resource_slider(commands, resources::Resource::Brick, resources.brick),
        resource_slider(commands, resources::Resource::Sheep, resources.sheep),
        resource_slider(commands, resources::Resource::Wheat, resources.wheat),
        resource_slider(commands, resources::Resource::Ore, resources.ore),
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
            },
            BackgroundColor(NORMAL_BUTTON),
            BorderColor::all(Color::BLACK),
            Text::new("confirm".to_string()),
        ),
    ];
    commands.spawn((
        Node {
            display: Display::Grid,
            border: UiRect::all(Val::Px(5.0)),
            margin: UiRect::all(Val::Auto),
            padding: UiRect::all(Val::Percent(2.)),
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
        BorderColor::all(BORDER_COLOR_ACTIVE),
        BackgroundColor(NORMAL_BUTTON.with_alpha(0.9)),
        spawn_related_bundle,
    ));
}
#[derive(Component)]
#[require(KatanComponent)]
pub struct SumbitButton {
    new_max_resources: u8,
}
#[derive(Debug, Component, Clone, Copy)]
#[require(KatanComponent)]
pub struct RobberResourceSpinner {
    resource: resources::Resource,
    max: u8,
}
#[derive(Debug, Resource, Clone, Copy, Default)]
pub struct RobberDiscard(Resources);
impl SpinnerButtonInteraction<RobberResourceSpinner> for ResMut<'_, RobberDiscard> {
    fn increment(&mut self, resource: &RobberResourceSpinner) {
        *self.0.get_mut(resource.resource) += 1;
    }
    fn decrement(&mut self, resource: &RobberResourceSpinner) {
        *self.0.get_mut(resource.resource) -= 1;
    }

    fn can_increment(&mut self, resource: &RobberResourceSpinner) -> bool {
        self.0.get(resource.resource) < resource.max
    }
    fn can_decrement(&mut self, resource: &RobberResourceSpinner) -> bool {
        self.0.get(resource.resource) > 0
    }
}
fn resource_slider(
    commands: &mut Commands<'_, '_>,
    resource: resources::Resource,
    max: u8,
) -> impl Bundle {
    let entity = commands.spawn(RobberResourceSpinner { resource, max }).id();
    common_ui::spinner_bundle::<RobberResourceSpinner>(entity, Text::new(format!("{resource:?}")))
}
