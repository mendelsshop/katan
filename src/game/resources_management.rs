use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy::{ecs::system::SystemParam, prelude::*};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    GameState, Input, Layout,
    colors::{CatanColor, CatanColorRef, CurrentColor},
    common_ui::{self, ButtonInteraction, SpinnerButtonInteraction, Value},
    resources::{self, Resources},
    setup_game::Ports,
};
#[derive(Component, Clone, Copy, Debug)]
pub struct TradeButton;
#[derive(Component, Clone, Copy, Debug)]
pub struct BankTradeButton;
#[derive(SystemParam)]
struct TradeState<'w, 's> {
    trade: Res<'w, TradingResources>,
    commands: Commands<'w, 's>,
    // maybe a with<..> just in case there more entities that also have color
    colors: Query<'w, 's, &'static CatanColorRef>,
    current_color: Res<'w, CurrentColor>,
}

// TODO: clicking this despawns its parent too
// only active if both players have the resources
// if clicked then do trade
#[derive(Component)]
pub struct AcceptTrade {
    color: CatanColorRef,
    trade: TradingResources,
}
// TODO: clicking this despawns its parent too
#[derive(Component)]
pub struct RejectTrade {
    color: CatanColorRef,
    trade: TradingResources,
}
// TODO: if no more children then despawn
// maybe also spawn button on main player to cancel trade
#[derive(Component)]
pub struct Trade(TradingResources);
impl ButtonInteraction<TradeButton> for TradeState<'_, '_> {
    fn interact(&mut self, _: &TradeButton) {
        self.commands
            .spawn(Trade(*self.trade))
            .with_related_entities::<TradeChildOf>(|b| {
                self.colors
                    .iter()
                    .filter(|c| **c != self.current_color.0)
                    .for_each(|color| {
                        // TODO: spawn in specific players window
                        b.spawn((
                            Node {
                                display: Display::Grid,
                                grid_template_columns: vec![
                                    GridTrack::auto(),
                                    GridTrack::auto(),
                                    GridTrack::auto(),
                                ],
                                ..Default::default()
                            },
                            Visibility::Hidden,
                            children![
                                Text::new(self.trade.to_string()),
                                (
                                    Button,
                                    Text::new("ok"),
                                    RejectTrade {
                                        color: *color,
                                        trade: *self.trade
                                    },
                                ),
                                (
                                    Button,
                                    Text::new("x"),
                                    AcceptTrade {
                                        color: *color,
                                        trade: *self.trade
                                    }
                                )
                            ],
                        ));
                    });
            });
    }
    fn verify(&mut self, _: &TradeButton) -> bool {
        let (giving, taking) = self.trade.given_and_taken();
        !giving.is_empty() && !taking.is_empty()
    }
}
#[derive(Component)]
#[relationship(relationship_target = TradeChildren)]
pub struct TradeChildOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TradeChildOf, linked_spawn)]
pub struct TradeChildren(Vec<Entity>);
#[derive(SystemParam)]
struct BankTradeState<'w, 's> {
    trading_resources: Res<'w, TradingResources>,
    current_color: Res<'w, CurrentColor>,
    player_resources_and_ports:
        Query<'w, 's, (&'static mut Resources, &'static Ports), With<CatanColor>>,
    input: ResMut<'w, Input>,
}

impl ButtonInteraction<BankTradeButton> for BankTradeState<'_, '_> {
    fn verify(&mut self, _: &BankTradeButton) -> bool {
        let (giving, taking) = self.trading_resources.given_and_taken();
        let taking: i8 = taking.iter().map(|(_, count)| count).sum();
        let ports = self
            .player_resources_and_ports
            .get(self.current_color.0.entity)
            .map(|(_, ports)| ports);

        if let Ok(ports) = ports {
            let giving: i8 = giving
                .iter()
                .filter_map(|(k, count)| {
                    let trade_rate = ports.get_trade_rate(*k);
                    (*count % trade_rate as i8 == 0).then_some(count / trade_rate as i8)
                })
                .sum();
            // TODO: verify there is enough resources in bank
            // TODO: port
            println!("{giving} -> {taking}");

            (giving == -(taking)) && giving != 0 && taking != 0
        } else {
            false
        }
    }
    fn interact(&mut self, _: &BankTradeButton) {
        *self.input = Input::BankTrade(*self.trading_resources);
    }
}
pub fn show_player_trade(
    resources: Res<'_, TradingResources>,
    mut text_query: Single<'_, '_, (&TradingText, &mut Text)>,
) {
    if resources.is_changed() {
        **text_query.1 = format!("{}", resources.into_inner());
    }
}
pub fn show_player_resources(
    player_resources: Query<'_, '_, (Entity, (&CatanColor, &Resources)), Changed<Resources>>,
    player_resources_nodes: Query<'_, '_, (&mut Text, &Value<TradingResourceSpinner>)>,
    sliders: Query<'_, '_, &TradingResourceSpinner>,
    res: Res<'_, CurrentColor>,
) {
    for resource in player_resources {
        println!("{resource:?}");
    }
    if let Ok((_, resources)) = player_resources.get(res.0.entity) {
        for (mut text, slider_ref) in player_resources_nodes {
            if let Ok(resource_slider) = sliders.get(slider_ref.0) {
                **text = resources.1.get(resource_slider.0).to_string();
            }
        }
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub struct TradingText;
pub fn setup_players_resources(mut commands: Commands<'_, '_>, layout: Res<'_, Layout>) {
    let children = children![
        resource_slider(&mut commands, resources::Resource::Wood),
        resource_slider(&mut commands, resources::Resource::Brick),
        resource_slider(&mut commands, resources::Resource::Sheep),
        resource_slider(&mut commands, resources::Resource::Wheat),
        resource_slider(&mut commands, resources::Resource::Ore),
    ];
    commands.entity(layout.resources).insert((children![
        (
            Node {
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::percent(70.),
                    GridTrack::percent(10.),
                    GridTrack::percent(10.),
                    GridTrack::percent(10.)
                ],
                ..default()
            },
            children![
                (
                    TradingText,
                    Node {
                        display: Display::Grid,
                        ..default()
                    },
                    TextFont {
                        font_size: 10.,
                        ..default()
                    },
                    Text::new("")
                ),
                (
                    Button,
                    TradingResourceResetButton,
                    Node {
                        display: Display::Grid,
                        ..default()
                    },
                    TextFont {
                        font_size: 10.,
                        ..default()
                    },
                    Text::new("x")
                ),
                (
                    Button,
                    Node {
                        display: Display::Grid,
                        ..default()
                    },
                    TextFont {
                        font_size: 10.,
                        ..default()
                    },
                    TradeButton,
                    // trade button
                    Text::new("t")
                ),
                (
                    Button,
                    Node {
                        display: Display::Grid,
                        ..default()
                    },
                    TextFont {
                        font_size: 10.,
                        ..default()
                    },
                    BankTradeButton,
                    // bank button
                    Text::new("b")
                ),
            ]
        ),
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
            children
        )
    ],));
}
fn resource_slider(commands: &mut Commands<'_, '_>, kind: resources::Resource) -> impl Bundle {
    let entity = commands.spawn(TradingResourceSpinner(kind)).id();
    common_ui::spinner_bundle::<TradingResourceSpinner>(entity, Text::new(format!("{kind:?}")))
}

#[derive(Resource, Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradingResources {
    pub wood: i8,
    pub brick: i8,
    pub sheep: i8,
    pub wheat: i8,
    pub ore: i8,
}
impl Add<TradingResources> for Resources {
    type Output = Self;

    fn add(self, rhs: TradingResources) -> Self::Output {
        Self {
            wood: (self.wood as i8 + rhs.wood) as u8,
            brick: (self.brick as i8 + rhs.brick) as u8,
            sheep: (self.sheep as i8 + rhs.sheep) as u8,
            wheat: (self.wheat as i8 + rhs.wheat) as u8,
            ore: (self.ore as i8 + rhs.ore) as u8,
        }
    }
}
impl Sub<TradingResources> for Resources {
    type Output = Self;

    fn sub(self, rhs: TradingResources) -> Self::Output {
        Self {
            wood: (self.wood as i8 - rhs.wood) as u8,
            brick: (self.brick as i8 - rhs.brick) as u8,
            sheep: (self.sheep as i8 - rhs.sheep) as u8,
            wheat: (self.wheat as i8 - rhs.wheat) as u8,
            ore: (self.ore as i8 - rhs.ore) as u8,
        }
    }
}
impl AddAssign<TradingResources> for Resources {
    fn add_assign(&mut self, rhs: TradingResources) {
        *self = *self + rhs;
    }
}
impl SubAssign<TradingResources> for Resources {
    fn sub_assign(&mut self, rhs: TradingResources) {
        *self = *self - rhs;
    }
}

impl fmt::Display for TradingResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (giving, taking): (Vec<_>, Vec<_>) = self.given_and_taken();
        // TODO: should we show if giving or taking is 0
        write!(
            f,
            "{} -> {}",
            giving
                .iter()
                .map(|(kind, count)| format!("{} {kind:?}", count.abs()))
                .join(", "),
            taking
                .iter()
                .map(|(kind, count)| format!("{count} {kind:?}"))
                .join(", ")
        )
    }
}
impl TradingResources {
    pub const fn get(&self, selector: resources::Resource) -> i8 {
        match selector {
            resources::Resource::Wood => self.wood,
            resources::Resource::Brick => self.brick,
            resources::Resource::Sheep => self.sheep,
            resources::Resource::Wheat => self.wheat,
            resources::Resource::Ore => self.ore,
        }
    }
    pub const fn get_mut(&mut self, selector: resources::Resource) -> &mut i8 {
        match selector {
            resources::Resource::Wood => &mut self.wood,
            resources::Resource::Brick => &mut self.brick,
            resources::Resource::Sheep => &mut self.sheep,
            resources::Resource::Wheat => &mut self.wheat,
            resources::Resource::Ore => &mut self.ore,
        }
    }

    fn given_and_taken(
        &self,
    ) -> (
        Vec<(resources::Resource, i8)>,
        Vec<(resources::Resource, i8)>,
    ) {
        [
            resources::Resource::Wood,
            resources::Resource::Brick,
            resources::Resource::Sheep,
            resources::Resource::Wheat,
            resources::Resource::Ore,
        ]
        .map(|r| (r, self.get(r)))
        .into_iter()
        .filter(|(_, count)| *count != 0)
        .partition(|(_, count)| *count < 0)
    }
}
#[derive(Debug, Component, Clone, Copy)]
pub struct TradingResourceSpinner(resources::Resource);
#[derive(SystemParam)]
struct TradingSpinnerState<'w, 's> {
    trading_resources: ResMut<'w, TradingResources>,
    current_color: Res<'w, CurrentColor>,
    player_resources_q: Query<'w, 's, &'static Resources, With<CatanColor>>,
}

impl SpinnerButtonInteraction<TradingResourceSpinner> for TradingSpinnerState<'_, '_> {
    fn increment(&mut self, resource: &TradingResourceSpinner) {
        *self.trading_resources.get_mut(resource.0) += 1;
    }
    fn decrement(&mut self, resource: &TradingResourceSpinner) {
        *self.trading_resources.get_mut(resource.0) -= 1;
    }
    fn can_decrement(&mut self, resource: &TradingResourceSpinner) -> bool {
        let current_value = self.trading_resources.get(resource.0);
        current_value > 0
            || self
                .player_resources_q
                .get(self.current_color.0.entity)
                .is_ok_and(|r| r.get(resource.0) > current_value.unsigned_abs())
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct TradingResourceResetButton;
impl ButtonInteraction<TradingResourceResetButton> for ResMut<'_, TradingResources> {
    fn interact(&mut self, _: &TradingResourceResetButton) {
        **self = TradingResources::default();
    }
}
pub fn reset_trading_resources(mut resources: ResMut<'_, TradingResources>) {
    *resources = TradingResources::default();
}

// manages card trading and showing users resources
pub struct ResourceManagmentPlugin;
impl Plugin for ResourceManagmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TradingResources::default())
            .add_systems(OnEnter(GameState::Start), setup_players_resources)
            .add_systems(Update, show_player_resources)
            .add_systems(Update, show_player_trade)
            // TODO: maybe remove the Changed<Resources> for this one, so new players cards always show
            .add_systems(OnEnter(GameState::Roll), show_player_resources)
            .add_systems(OnEnter(GameState::Roll), reset_trading_resources)
            .add_systems(
                Update,
                (common_ui::spinner_buttons_interactions::<
                    TradingResourceSpinner,
                    TradingSpinnerState<'_, '_>,
                >(),)
                    .run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                (common_ui::button_system_with_generic::<TradeButton, TradeState<'_, '_>>,)
                    .run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                (common_ui::button_system_with_generic::<BankTradeButton, BankTradeState<'_, '_>>,)
                    .run_if(in_state(GameState::Turn)),
            )
            .add_systems(
                Update,
                (common_ui::button_system_with_generic::<
                    TradingResourceResetButton,
                    ResMut<'_, TradingResources>,
                >,)
                    .run_if(in_state(GameState::Turn)),
            );
    }
}
