use crate::AppState;
use bevy::{
    input_focus::{InputDispatchPlugin, InputFocus},
    prelude::*,
};
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputPlugin, TextInputSystem, TextInputTextColor,
    TextInputTextFont,
};
const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);
const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);

pub struct LobbyPlugin;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::Menu)]

pub enum MenuState {
    #[default]
    Lobby,
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Server;
#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Room;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputDispatchPlugin)
            .add_plugins(TextInputPlugin)
            .add_systems(Update, focus.before(TextInputSystem))
            .add_systems(OnEnter(AppState::Menu), setup_lobby);
    }
}
pub fn setup_lobby(mut commands: Commands<'_, '_>) {
    commands.spawn((
        DespawnOnExit(MenuState::Lobby),
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(50.), GridTrack::percent(50.)],
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        children![
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                        GridTrack::percent(25.),
                    ],
                    ..Default::default()
                },
                bevy_ui_widgets::observe(background_node_click),
                children![
                    Text::new("server:"),
                    (
                        Server,
                        Node {
                            width: Val::Px(200.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        TextInputInactive(true),
                        BorderColor::all(BORDER_COLOR_ACTIVE),
                        BackgroundColor(BACKGROUND_COLOR),
                        TextInput,
                        TextInputTextFont(TextFont {
                            font_size: 34.,
                            ..default()
                        }),
                        bevy_ui_widgets::observe(text_input_click),
                        TextInputTextColor(TextColor(TEXT_COLOR)),
                    ),
                    Text::new("room:"),
                    (
                        Room,
                        Node {
                            width: Val::Px(200.0),
                            border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        TextInputInactive(true),
                        BorderColor::all(BORDER_COLOR_INACTIVE),
                        BackgroundColor(BACKGROUND_COLOR),
                        TextInput,
                        TextInputTextFont(TextFont {
                            font_size: 34.,
                            ..default()
                        }),
                        bevy_ui_widgets::observe(text_input_click),
                        TextInputTextColor(TextColor(TEXT_COLOR)),
                    ),
                ]
            ),
            (Text::new("join"), Button)
        ],
    ));
}

fn focus(
    focus: Res<'_, InputFocus>,
    mut text_inputs: Query<'_, '_, (Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    if !focus.is_changed() {
        return;
    }

    for (entity, mut inactive, mut border_color) in &mut text_inputs {
        if focus.0 == Some(entity) {
            inactive.0 = false;
            *border_color = BORDER_COLOR_ACTIVE.into();
        } else {
            inactive.0 = true;
            *border_color = BORDER_COLOR_INACTIVE.into();
        }
    }
}
fn background_node_click(
    mut trigger: On<'_, '_, Pointer<Over>>,
    mut focus: ResMut<'_, InputFocus>,
) {
    focus.0 = None;
    trigger.propagate(false);
}

fn text_input_click(mut trigger: On<'_, '_, Pointer<Over>>, mut focus: ResMut<'_, InputFocus>) {
    focus.0 = Some(trigger.event_target());
    trigger.propagate(false);
}
