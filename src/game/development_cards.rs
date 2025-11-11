use std::ops::{Add, AddAssign};

use bevy::prelude::*;
use itertools::Itertools;

use crate::{
    game::LocalPlayer,
    utils::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
};

use super::{
    Input, Layout,
    colors::{CatanColor, CurrentColor},
    development_card_actions::DevelopmentCardShow,
    resources::{DEVELOPMENT_CARD_RESOURCES, Resources},
    turn_ui::DevelopmentCardButton,
};

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevelopmentCard {
    Knight,
    Monopoly,
    YearOfPlenty,
    RoadBuilding,
    VictoryPoint,
}
impl From<DevelopmentCard> for DevelopmentCards {
    fn from(value: DevelopmentCard) -> Self {
        match value {
            DevelopmentCard::Knight => Self {
                knight: 1,
                monopoly: 0,
                year_of_plenty: 0,
                road_building: 0,
                victory_point: 0,
            },
            DevelopmentCard::Monopoly => Self {
                knight: 0,
                monopoly: 1,
                year_of_plenty: 0,
                road_building: 0,
                victory_point: 0,
            },
            DevelopmentCard::YearOfPlenty => Self {
                knight: 0,
                monopoly: 0,
                year_of_plenty: 1,
                road_building: 0,
                victory_point: 0,
            },
            DevelopmentCard::RoadBuilding => Self {
                knight: 0,
                monopoly: 0,
                year_of_plenty: 0,
                road_building: 1,
                victory_point: 0,
            },
            DevelopmentCard::VictoryPoint => Self {
                knight: 0,
                monopoly: 0,
                year_of_plenty: 0,
                road_building: 0,
                victory_point: 1,
            },
        }
    }
}
impl DevelopmentCards {
    pub const fn get(&self, selector: DevelopmentCard) -> u8 {
        match selector {
            DevelopmentCard::Knight => self.knight,
            DevelopmentCard::Monopoly => self.monopoly,
            DevelopmentCard::YearOfPlenty => self.year_of_plenty,
            DevelopmentCard::RoadBuilding => self.road_building,
            DevelopmentCard::VictoryPoint => self.victory_point,
        }
    }
    pub const fn get_mut(&mut self, selector: DevelopmentCard) -> &mut u8 {
        match selector {
            DevelopmentCard::Knight => &mut self.knight,
            DevelopmentCard::Monopoly => &mut self.monopoly,
            DevelopmentCard::YearOfPlenty => &mut self.year_of_plenty,
            DevelopmentCard::RoadBuilding => &mut self.road_building,
            DevelopmentCard::VictoryPoint => &mut self.victory_point,
        }
    }
}
#[derive(Debug, Resource, Clone, Default)]
pub struct DevelopmentCardsPile(pub Vec<DevelopmentCard>);
#[derive(Debug, Component, Clone, Copy, Default)]
pub struct DevelopmentCards {
    knight: u8,
    monopoly: u8,
    year_of_plenty: u8,
    road_building: u8,
    victory_point: u8,
}
impl DevelopmentCards {
    pub const fn count(self) -> u8 {
        self.knight + self.monopoly + self.year_of_plenty + self.road_building + self.victory_point
    }
    pub fn new_player() -> Self {
        Self {
            knight: 4,
            monopoly: core::default::Default::default(),
            year_of_plenty: 1,
            road_building: 1,
            victory_point: core::default::Default::default(),
        }
    }
}

impl AddAssign for DevelopmentCards {
    fn add_assign(&mut self, rhs: Self) {
        self.knight += rhs.knight;
        self.monopoly += rhs.monopoly;
        self.year_of_plenty += rhs.year_of_plenty;
        self.road_building += rhs.road_building;
        self.victory_point += rhs.victory_point;
    }
}
impl Add for DevelopmentCards {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            knight: self.knight + rhs.knight,
            monopoly: self.monopoly + rhs.monopoly,
            year_of_plenty: self.year_of_plenty + rhs.year_of_plenty,
            road_building: self.road_building + rhs.road_building,
            victory_point: self.victory_point + rhs.victory_point,
        }
    }
}

pub fn buy_development_card_interaction(
    color_r: Res<'_, CurrentColor>,
    free_dev_cards: Query<'_, '_, (Entity, &DevelopmentCard), Without<CatanColor>>,
    mut player_resources_and_dev_cards: Query<'_, '_, &mut Resources, With<CatanColor>>,
    interaction_query: Single<
        '_,
        '_,
        (
            &DevelopmentCardButton,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
        ),
        Changed<Interaction>,
    >,
    mut input: ResMut<'_, Input>,
) {
    if let Ok(player_resources) = player_resources_and_dev_cards.get_mut(color_r.0.entity) {
        let required_resources = DEVELOPMENT_CARD_RESOURCES;
        if !player_resources.contains(required_resources) {
            return;
        }
        let (_, interaction, mut color, mut button) = interaction_query.into_inner();
        match *interaction {
            Interaction::Pressed => {
                if free_dev_cards.count() > 0 {
                    *input = Input::TakeDevelopmentCard;
                }
                *color = PRESSED_BUTTON.into();
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
pub fn setup_show_dev_cards(
    player_dev_cards: Query<'_, '_, &DevelopmentCards, With<CatanColor>>,
    res: Res<'_, LocalPlayer>,
    mut commands: Commands<'_, '_>,
    layout: Res<'_, Layout>,
) {
    if let Ok(player_dev_cards) = player_dev_cards.get(res.0.entity) {
        commands
            .entity(layout.development_cards)
            .with_children(|builder| {
                let dev_button_iter = [
                    DevelopmentCard::Monopoly,
                    DevelopmentCard::Knight,
                    DevelopmentCard::RoadBuilding,
                    DevelopmentCard::YearOfPlenty,
                ]
                .iter()
                .filter_map(|card_type| {
                    let count = player_dev_cards.get(*card_type);
                    (count > 0).then_some((card_type, count))
                });
                let player_vps = player_dev_cards.get(DevelopmentCard::VictoryPoint);
                let count = dev_button_iter.clone().count() + usize::from(player_vps > 0);
                if player_vps > 0 {
                    builder.spawn((
                        Node {
                            display: Display::Grid,
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BorderColor::all(Color::BLACK),
                        Transform::from_rotation(Quat::from_rotation_z(
                            ((0 as f32 - count as f32 / 2.) * 10.).to_radians(),
                        )),
                        DevelopmentCardShow(player_vps),
                        DevelopmentCard::VictoryPoint,
                        children![Text(format!("{:?}", DevelopmentCard::VictoryPoint))],
                    ));
                }
                dev_button_iter
                    .enumerate()
                    .for_each(|(i, (card_kind, card_count))| {
                        builder.spawn((
                            Node {
                                display: Display::Grid,
                                border: UiRect::all(Val::Px(1.)),
                                ..default()
                            },
                            BorderColor::all(Color::BLACK),
                            Transform::from_rotation(Quat::from_rotation_z(
                                (((i + usize::from(player_vps > 0)) as f32 - count as f32 / 2.)
                                    * 10.)
                                    .to_radians(),
                            )),
                            DevelopmentCardShow(card_count),
                            Button,
                            *card_kind,
                            children![Text(format!("{card_kind:?}"))],
                        ));
                    });
            });
    }
}
pub fn show_dev_cards(
    player_dev_cards: Query<
        '_,
        '_,
        &DevelopmentCards,
        (With<CatanColor>, Changed<DevelopmentCards>),
    >,
    mut shown_cards: Query<'_, '_, (&DevelopmentCard, &mut DevelopmentCardShow), With<Node>>,
    res: Res<'_, LocalPlayer>,
    mut commands: Commands<'_, '_>,
    layout: Res<'_, Layout>,
) {
    if let Ok(player_dev_cards) = player_dev_cards.get(res.0.entity) {
        let new_cards = [
            DevelopmentCard::Monopoly,
            DevelopmentCard::Knight,
            DevelopmentCard::RoadBuilding,
            DevelopmentCard::YearOfPlenty,
            DevelopmentCard::VictoryPoint,
        ]
        .iter()
        .filter_map(|card_type| {
            let count = player_dev_cards.get(*card_type);
            (count > 0).then_some((card_type, count))
        })
        .filter(|(card_type, count)| {
            let this = shown_cards.iter_mut().find(|(card, _)| card == card_type);
            if let Some((_, mut development_card_show)) = this {
                development_card_show.0 = *count;
                false
            } else {
                true
            }
        })
        .enumerate()
        .collect_vec();

        let count = new_cards.clone().len();
        commands
            .entity(layout.development_cards)
            .with_children(|builder| {
                for (i, (card_kind, card_count)) in new_cards {
                    builder.spawn((
                        Node {
                            display: Display::Grid,
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BorderColor::all(Color::BLACK),
                        Transform::from_rotation(Quat::from_rotation_z(
                            ((i as f32 - count as f32 / 2.) * 10.).to_radians(),
                        )),
                        DevelopmentCardShow(card_count),
                        *card_kind,
                        children![Text(format!("{card_kind:?}"))],
                    ));
                }
            });
    }
}
