use bevy::prelude::*;

use crate::{
    Layout,
    colors::{CatanColor, CurrentColor},
    common_ui::{self, SliderButtonInteraction, Value},
    development_cards::DevelopmentCard,
    resources::{self, Resources},
};
pub fn show_player_resources(
    player_resources: Query<'_, '_, (&CatanColor, &Resources), Changed<Resources>>,
    player_resources_nodes: Query<'_, '_, (&mut Text, &Value<TradingResourcesSlider>)>,
    sliders: Query<'_, '_, &TradingResourcesSlider>,
    res: Res<'_, CurrentColor>,
) {
    for resource in player_resources {
        println!("{resource:?}")
    }
    if let Some(resources) = player_resources.get(res.0.entity).ok() {
        for (mut text, slider_ref) in player_resources_nodes {
            if let Some(resource_slider) = sliders.get(slider_ref.0).ok() {
                **text = resources.1.get(resource_slider.0).to_string();
            }
        }
    }
}
pub fn setup_players_resources(mut commands: Commands<'_, '_>, layout: Res<'_, Layout>) {
    let children = children![
        resource_slider(&mut commands, resources::Resource::Wood),
        resource_slider(&mut commands, resources::Resource::Brick),
        resource_slider(&mut commands, resources::Resource::Sheep),
        resource_slider(&mut commands, resources::Resource::Wheat),
        resource_slider(&mut commands, resources::Resource::Ore),
    ];
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
                children
            )
        ],
    ));
}
fn resource_slider(commands: &mut Commands<'_, '_>, kind: resources::Resource) -> impl Bundle {
    let entity = commands.spawn(TradingResourcesSlider(kind)).id();
    common_ui::slider_bundle::<TradingResourcesSlider>(entity)
}

#[derive(Component)]
pub struct ResourcesRef(pub Entity, pub resources::Resource);
#[derive(Resource)]
pub struct TradingResources {
    pub wood: i8,
    pub brick: i8,
    pub sheep: i8,
    pub wheat: i8,
    pub ore: i8,
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
}
#[derive(Debug, Component, Clone, Copy)]
pub struct TradingResourcesSlider(resources::Resource);
impl SliderButtonInteraction<TradingResourcesSlider> for ResMut<'_, TradingResources> {
    fn increment(&mut self, resource: &TradingResourcesSlider) {
        *self.get_mut(resource.0) += 1;
    }
    fn decrement(&mut self, resource: &TradingResourcesSlider) {
        *self.get_mut(resource.0) -= 1;
    }
}
