use std::ops::{Add, AddAssign};

use bevy::prelude::*;

use crate::{
    Layout,
    colors::{CatanColor, CurrentColor, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    development_card_actions::DevelopmentCardShow,
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
#[derive(Debug, Component, Clone, Copy, Default)]
pub struct DevelopmentCards {
    knight: u8,
    monopoly: u8,
    year_of_plenty: u8,
    road_building: u8,
    victory_point: u8,
}
impl DevelopmentCards {
    pub fn new_player() -> Self {
        Self::default()
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
    mut commands: Commands<'_, '_>,
    color_r: Res<'_, CurrentColor>,
    mut free_dev_cards: Query<'_, '_, (Entity, &DevelopmentCard), Without<CatanColor>>,
    mut player_resources_and_dev_cards: Query<
        '_,
        '_,
        (&mut Resources, &mut DevelopmentCards),
        With<CatanColor>,
    >,
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
    if let Ok((mut player_resources, mut player_dev_cards)) =
        player_resources_and_dev_cards.get_mut(color_r.0.entity)
    {
        let required_resources = DEVELOPMENT_CARD_RESOURCES;
        if !player_resources.contains(required_resources) {
            return;
        }
        let (_, interaction, mut color, mut button) = interaction_query.into_inner();
        match *interaction {
            Interaction::Pressed => {
                if let Some(card) = free_dev_cards.iter_mut().next() {
                    *player_resources -= required_resources;
                    *resources += required_resources;
                    *player_dev_cards.get_mut(*card.1) += 1;
                    commands.entity(card.0).despawn();
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
    res: Res<'_, CurrentColor>,
    mut commands: Commands<'_, '_>,
    layout: Res<'_, Layout>,
) {
    if let Ok(player_dev_cards) = player_dev_cards.get(res.0.entity) {
        commands
            .entity(layout.development_cards)
            .with_children(|builder| {
                builder
                    .spawn((Node {
                        display: Display::Grid,
                        grid_template_columns: vec![
                            GridTrack::percent(20.),
                            GridTrack::percent(20.),
                            GridTrack::percent(20.),
                            GridTrack::percent(20.),
                            GridTrack::percent(20.),
                        ],

                        ..default()
                    },))
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
                        let count = dev_button_iter.clone().count();
                        if player_dev_cards.get(DevelopmentCard::VictoryPoint) > 0 {
                            builder.spawn((
                                Node {
                                    display: Display::Grid,
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BorderColor(Color::BLACK),
                                Transform::from_rotation(Quat::from_rotation_z(
                                    ((0 as f32 - count as f32 / 2.) * 10.).to_radians(),
                                )),
                                DevelopmentCardShow,
                                DevelopmentCard::VictoryPoint,
                                children![Text(format!("{:?}", DevelopmentCard::VictoryPoint))],
                            ));
                        }
                        dev_button_iter
                            .enumerate()
                            .for_each(|(i, (card_kind, card_count))| {
                                println!("{card_count} {card_kind:?}");
                                builder.spawn((
                                    Node {
                                        display: Display::Grid,
                                        border: UiRect::all(Val::Px(1.)),
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    Transform::from_rotation(Quat::from_rotation_z(
                                        ((i as f32 - count as f32 / 2.) * 10.).to_radians(),
                                    )),
                                    DevelopmentCardShow,
                                    Button,
                                    *card_kind,
                                    children![Text(format!("{card_kind:?}"))],
                                ));
                            });
                    });
            });
    }
}
pub fn show_dev_cards(
    player_dev_cards: Query<'_, '_, (&CatanColor, &DevelopmentCards), Changed<DevelopmentCards>>,
    shown_cards: Query<'_, '_, (&Node, &DevelopmentCard)>,
    res: Res<'_, CurrentColor>,
    commands: Commands<'_, '_>,
) {
    if let Ok(player_dev_cards) = player_dev_cards.get(res.0.entity) {}
}
