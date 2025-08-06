use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    Building,
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
pub fn place_robber(mut commands: Commands<'_, '_>, robber: Res<'_, Robber>) {
    generate_postions(3)
        // TODO: skip current robber pos
        .filter(|p| *p != robber.0)
        .filter_map(|p| {
            let pos: FPosition = p.into();
            let (x, y) = pos.hex_to_pixel();
            (x != 0. || y != 0.).then_some((x, y, p))
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
    // show ui to place robber
    // on every hex besides for current robber hex
    // the make interaction function, that when clicked:
    // 1) moves the robber there, set the robber postion
    // 2) tries to take a resource from other player, or show ui to choose which player to pick
    //    from
}
fn place_robber_interaction(
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
fn choose_player_to_take_from<'a>(
    position: &Position,
    color: CurrentColor,
    used_buildings: impl Iterator<Item = (&'a CatanColor, &'a BuildingPosition)> + Clone,
    resources: &mut Query<'_, '_, (&CatanColor, &mut Resources)>,
) {
    let mut colors = used_buildings
        .filter_map(|(c, b)| (c != &color.0 && b.contains(position)).then_some(c))
        .unique()
        .collect_vec();
    let (used_resources, other_resources): (Vec<_>, Vec<_>) =
        resources.iter_mut().partition(|r| colors.contains(&r.0));
    if colors.len() == 1 {
        let other_color = colors.remove(0);
        let (_, mut other_color_resources) =
            find_with(other_color, used_resources.into_iter()).unwrap();
        if other_color_resources.count() > 0 {
            let (_, mut resources) = find_with(&color.0, other_resources.into_iter()).unwrap();
            take_resource(&mut resources, &mut other_color_resources);
        }
    } else {
        // show options of how to pick from
    }
}
fn find_with<'a>(
    c: &CatanColor,
    mut resources: impl Iterator<Item = (&'a CatanColor, Mut<'a, Resources>)>,
) -> Option<(&'a CatanColor, Mut<'a, Resources>)> {
    resources.find(|r| r.0 == c)
}
const fn choose_player_to_take_from_interaction() {}
