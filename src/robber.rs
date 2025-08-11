use bevy::{
    prelude::*,
    render::{camera::RenderTarget, view::RenderLayers},
    window::WindowRef,
};
use itertools::Itertools;

use crate::{
    Building, GameState,
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    positions::{BuildingPosition, FPosition, Position, generate_postions},
    resources::{Resources, take_resource},
};

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Robber(pub Position);
impl Default for Robber {
    fn default() -> Self {
        Self(Position { q: 0, r: 0, s: 0 })
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub struct RobberButton;
pub fn place_robber(
    mut commands: Commands<'_, '_>,
    robber: Res<'_, Robber>,
    state: &mut ResMut<'_, NextState<GameState>>,
) {
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
                        width: Val::Px(15.0),
                        height: Val::Px(15.0),
                        left: Val::Px(x * 28.),
                        bottom: Val::Px(y * 28.),
                        ..default()
                    },
                    p,
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )],
            ));
        });

    state.set(GameState::PlaceRobber);
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
    building_q: Query<'_, '_, (&CatanColor, &'_ BuildingPosition), With<Building>>,
    current_color: Res<'_, CurrentColor>,
    mut robber: ResMut<'_, Robber>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
    mut commands: Commands<'_, '_>,
    mut state: ResMut<'_, NextState<GameState>>,
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
                    building_q.into_iter(),
                    &mut player_resources,
                    &mut commands,
                    &mut state,
                );
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
    used_buildings: impl Iterator<Item = (&'a CatanColor, &'a BuildingPosition)> + Clone,
    resources: &mut Query<'_, '_, (&CatanColor, &mut Resources)>,
    commands: &mut Commands<'_, '_>,
    state: &mut ResMut<'_, NextState<GameState>>,
) {
    let mut colors = used_buildings
        .filter_map(|(c, b)| {
            (c != &color.0
                && b.contains(position)
                // we check that are enough resources to steal instead of later on, becuase if
                // there are no one to steal from them we need to go back to turn, and its much
                // easer to check that here than later on espicially if there are mutlitple players
                // surrounding the hex
                && find_with_color(c, resources.iter()).is_some_and(|r| r.1.count() > 0))
            .then_some(c)
        })
        .unique()
        .collect_vec();
    if colors.len() == 1 {
        let other_color = colors.remove(0);
        let (_, mut other_color_resources) =
            find_with_color(other_color, resources.iter_mut()).unwrap();
        let put_resources = take_resource(&mut other_color_resources);
        let mut current_resources = find_with_color(&color.0, resources.iter_mut()).unwrap();
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
                    **color,
                    BorderRadius::MAX,
                    BackgroundColor(color.to_bevy_color()),
                )],
            ));
        }
        state.set(GameState::RobberPickColor);
    }
}
fn find_with_color<'a, T>(
    c: &CatanColor,
    mut resources: impl Iterator<Item = (&'a CatanColor, T)>,
) -> Option<(&'a CatanColor, T)> {
    resources.find(|r| r.0 == c)
}
pub fn choose_player_to_take_from_interaction(
    current_color: Res<'_, CurrentColor>,
    mut player_resources: Query<'_, '_, (&CatanColor, &mut Resources)>,
    mut robber_taking_query: Query<
        '_,
        '_,
        (&Interaction, &CatanColor, &mut Button, &mut BackgroundColor),
        (Changed<Interaction>, With<RobberChooseColorButton>),
    >,
    mut state: ResMut<'_, NextState<GameState>>,
) {
    for (interaction, color, mut button, mut button_color) in &mut robber_taking_query {
        match *interaction {
            Interaction::Pressed => {
                button.set_changed();
                let mut other_resources =
                    find_with_color(color, player_resources.iter_mut()).unwrap();
                let put_resources = take_resource(&mut other_resources.1);
                let mut current_resources =
                    find_with_color(&current_color.0, player_resources.iter_mut()).unwrap();
                put_resources(&mut current_resources.1);
                // either we are coming from roll(7) or in middle of turn(dev card) but we always go back to
                // turn
                state.set(GameState::Turn);
                break;
            }
            Interaction::Hovered => {
                *button_color = (if color == &CatanColor::White {
                    color.to_bevy_color().darker(0.2)
                } else {
                    color.to_bevy_color().lighter(0.1)
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
pub struct CameraID(Entity);

#[derive(Bundle)]
pub struct ResourceDiscarder {
    camera_window: CameraID,
    color: CatanColor,
    resources: Resources, // doesnt change
    changed_resources: Resources,
}
fn take_extra_resources(
    commands: &mut Commands<'_, '_>,
    player_resources: Query<'_, '_, (&CatanColor, &Resources)>,
) {
    player_resources
        .iter()
        .filter(|resources| resources.1.count() > 7)
        .for_each(|r| {
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
                        target: RenderTarget::Window(WindowRef::Entity(window)),
                        ..default()
                    },
                    RenderLayers::layer(1),
                ))
                .id();
            commands.spawn(ResourceDiscarder {
                camera_window: CameraID(camera),
                color: *r.0,
                resources: *r.1,
                changed_resources: *r.1,
            });
        });
}
