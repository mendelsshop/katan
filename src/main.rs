#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

mod cities;
mod colors;
mod development_card_actions;
mod development_cards;
mod dice;
mod positions;
mod resources;
mod roads;
mod robber;
mod setup_game;
mod towns;
mod turn_ui;

use bevy::prelude::*;

use crate::{
    colors::{
        CatanColor, ColorIterator, CurrentColor, CurrentSetupColor, HOVERED_BUTTON, NORMAL_BUTTON,
        PRESSED_BUTTON, SetupColorIterator,
    },
    development_card_actions::{MonopolyButton, RoadBuildingState, YearOfPlentyButton},
    positions::{BuildingPosition, Position, RoadPosition},
    resources::Resources,
    roads::{Road, RoadUI},
    robber::{PreRobberDiscardLeft, Robber, RobberButton, RobberChooseColorButton},
    towns::{Town, TownUI},
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,));
    app.init_state::<GameState>();
    app.insert_resource(BoardSize(3));
    app.init_resource::<Robber>();
    app.insert_resource(Resources::new_game());
    // TODO: is there way to init resource
    // without giving a value
    app.insert_resource(PreRobberDiscardLeft(0));
    app.insert_resource(CurrentColor(CatanColor::White));
    app.insert_resource(CurrentSetupColor(CatanColor::White));
    app.add_systems(Startup, setup);

    app.add_systems(OnEnter(GameState::SetupRoad), roads::place_setup_road);
    app.add_systems(OnEnter(GameState::SetupTown), towns::place_setup_town);

    app.add_systems(OnEnter(GameState::PlaceRobber), robber::place_robber);
    app.add_systems(
        OnEnter(GameState::RobberDiscardResources),
        robber::take_extra_resources,
    );
    app.add_systems(OnExit(GameState::SetupRoad), cleanup_button::<RoadPosition>);
    app.add_systems(OnExit(GameState::SetupRoad), cleanup_button::<RoadPosition>);
    app.add_systems(
        OnExit(GameState::SetupTown),
        cleanup_button::<BuildingPosition>,
    );
    app.add_systems(OnExit(GameState::PlaceRoad), cleanup_button::<RoadPosition>);
    app.add_systems(
        OnExit(RoadBuildingState::Road1),
        (
            cleanup_button::<RoadPosition>,
            // will this work if we are also exiting GameState::RoadBuilding
            (|mut state: ResMut<'_, NextState<GameState>>,
              mut sub_state: ResMut<'_, NextState<RoadBuildingState>>| {
                state.set(GameState::RoadBuilding);
                // because we share the logic with general road placement we have to handle the
                // state switch seperatly
                sub_state.set(RoadBuildingState::Road2)
            }),
        ),
    );
    app.add_systems(
        OnExit(RoadBuildingState::Road2),
        cleanup_button::<RoadPosition>,
    );
    app.add_systems(
        OnExit(GameState::PlaceTown),
        cleanup_button::<BuildingPosition>,
    );
    app.add_systems(
        OnExit(GameState::PlaceCity),
        cleanup_button::<BuildingPosition>,
    );

    app.add_systems(
        OnExit(GameState::PlaceRobber),
        cleanup_button::<RobberButton>,
    );
    app.add_systems(
        OnExit(GameState::RobberPickColor),
        cleanup_button::<RobberChooseColorButton>,
    );
    app.add_systems(
        OnExit(GameState::Monopoly),
        cleanup_button::<MonopolyButton>,
    );
    app.add_systems(
        OnEnter(GameState::Monopoly),
        development_card_actions::monopoly_setup,
    );

    app.add_systems(
        Update,
        development_card_actions::monopoly_interaction.run_if(in_state(GameState::Monopoly)),
    );
    app.add_systems(
        OnExit(GameState::YearOfPlenty),
        cleanup_button::<YearOfPlentyButton>,
    );
    app.add_systems(
        OnEnter(GameState::YearOfPlenty),
        development_card_actions::setup_year_of_plenty,
    );

    app.add_systems(
        Update,
        development_card_actions::year_of_plenty_interaction
            .run_if(in_state(GameState::YearOfPlenty)),
    );

    app.add_systems(OnEnter(GameState::PlaceRoad), roads::place_normal_road::<1>);
    app.add_systems(Update, development_cards::show_dev_cards);
    app.add_systems(
        OnEnter(RoadBuildingState::Road1),
        roads::place_normal_road::<0>,
    );
    app.add_systems(
        OnEnter(RoadBuildingState::Road2),
        roads::place_normal_road::<0>,
    );
    app.add_systems(OnEnter(GameState::PlaceTown), towns::place_normal_town);
    app.add_systems(OnEnter(GameState::PlaceCity), cities::place_normal_city);
    app.add_systems(
        OnTransition {
            // you might think, that we would do this after the last town (with SetupTown), but due
            // to how the color/player changing logic for setup its not acutally so
            exited: GameState::SetupRoad,
            entered: GameState::Roll,
        },
        turn_ui::show_turn_ui,
    );

    app.add_systems(
        Update,
        turn_ui::turn_ui_road_interaction.run_if(in_state(GameState::Turn)),
    );

    app.add_systems(
        Update,
        turn_ui::turn_ui_town_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui::turn_ui_city_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        turn_ui::turn_ui_roll_interaction.run_if(in_state(GameState::Roll)),
    );

    app.add_systems(
        Update,
        robber::choose_player_to_take_from_interaction.run_if(in_state(GameState::RobberPickColor)),
    );
    app.add_systems(
        Update,
        robber::place_robber_interaction.run_if(in_state(GameState::PlaceRobber)),
    );
    app.add_systems(
        Update,
        // TODO: if in turn or place state
        turn_ui::turn_ui_next_interaction.run_if({
            move |current_state: Option<Res<'_, State<GameState>>>| match current_state {
                Some(current_state) => ![
                    GameState::PlaceRobber,
                    GameState::Monopoly,
                    GameState::YearOfPlenty,
                    GameState::RobberPickColor,
                    GameState::Roll,
                    GameState::RobberDiscardResources,
                    GameState::RoadBuilding,
                ]
                .contains(&current_state),
                None => true,
            }
        }),
    );
    app.add_systems(OnEnter(GameState::SetupRoad), colors::set_setup_color);
    app.add_systems(OnEnter(GameState::Roll), colors::set_color);
    app.add_systems(
        Update,
        place_normal_interaction::<Road, RoadPosition, RoadUI, CurrentSetupColor>
            .run_if(in_state(GameState::SetupRoad)),
    );
    app.add_systems(
        Update,
        development_cards::buy_development_card_interaction.run_if(in_state(GameState::Turn)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Town, BuildingPosition, TownUI, CurrentSetupColor>
            .run_if(in_state(GameState::SetupTown)),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Road, RoadPosition, RoadUI, CurrentColor>
            .run_if(in_state(GameState::PlaceRoad).or(in_state(GameState::RoadBuilding))),
    );
    app.add_systems(
        Update,
        place_normal_interaction::<Town, BuildingPosition, TownUI, CurrentColor>
            .run_if(in_state(GameState::PlaceTown)),
    );
    app.add_systems(
        Update,
        (
            robber::counter_up_interaction,
            robber::counter_down_interaction,
            robber::counter_sumbit_interaction,
            robber::counter_text_update,
        )
            .run_if(in_state(GameState::RobberDiscardResources)),
    );

    app.add_systems(
        Update,
        cities::place_normal_city_interaction.run_if(in_state(GameState::PlaceCity)),
    );
    app.run();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    RobberDiscardResources,
    Nothing,
    Start,
    PlaceRoad,
    Roll,
    Turn,
    PlaceTown,
    PlaceCity,
    SetupRoad,
    SetupTown,
    RoadBuilding, // (dev card)
    YearOfPlenty, // (dev card)
    Monopoly,
    // picking which color to pick from
    RobberPickColor,
    // picking which place to put robber on
    PlaceRobber,
}
#[derive(Component, PartialEq, Debug, Clone, Copy)]
enum Number {
    Number(u8),
    None,
}

#[derive(Debug, Component, Clone, Copy)]
// our hexagons are pointy
enum Hexagon {
    Wood = 0,
    Brick,
    Sheep,
    Wheat,
    Ore,
    Desert,
    Water,
    Port,
    Empty,
}
impl From<u8> for Hexagon {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Wood,
            1 => Self::Brick,
            2 => Self::Sheep,
            3 => Self::Wheat,
            4 => Self::Ore,
            5 => Self::Desert,
            6 => Self::Water,
            7 => Self::Port,
            8 => Self::Empty,
            _ => Self::Empty,
        }
    }
}
impl Hexagon {
    fn color(&self) -> Color {
        match self {
            Self::Wood => Color::srgb_u8(161, 102, 47),
            Self::Brick => Color::srgb_u8(198, 74, 60),
            Self::Sheep => Color::srgb_u8(0, 255, 0),
            Self::Wheat => Color::srgb_u8(255, 191, 0),
            Self::Ore => Color::srgb_u8(67, 67, 65),
            Self::Desert => Color::srgb_u8(194, 178, 128),
            Self::Water => Color::srgb_u8(0, 0, 255),
            Self::Port => Color::srgb_u8(0, 0, 255),
            Self::Empty => Color::BLACK.with_alpha(-1.),
        }
    }
}
#[derive(Debug)]
enum Port {}
#[derive(Resource, Clone, Copy)]
struct BoardSize(u8);

fn cleanup_button<T: Component>(
    mut commands: Commands<'_, '_>,
    mut interaction_query: Query<'_, '_, Entity, (With<T>, With<Button>)>,
) {
    for entity in &mut interaction_query {
        commands.entity(entity).despawn();
    }
}

pub trait UI {
    type Pos;
    fn bundle(
        pos: Self::Pos,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<ColorMaterial>>,
        color: CatanColor,
    ) -> impl Bundle;
    fn resources() -> Resources;
}
// should interaction be doing the ui update for showing the roads/towns
fn place_normal_interaction<
    Kind: Component + Default + std::fmt::Debug,
    Pos: Component + Copy,
    U: UI<Pos = Pos>,
    // TODO: unify the different types of color for setup and during the game
    // one way would be to make a color enum that has variant for setup and one for the rest of the game
    // another way would be make the type of color be a marker struct that `#[requires(CatanColor)]`
    // and then we could just look for CatanColor, and when we need the more specific one we specify
    // via the marker struct
    C: Into<CatanColor> + Resource + Copy,
>(
    mut resources: ResMut<'_, Resources>,
    mut player_resources: Query<'_, '_, (&mut Resources, &CatanColor)>,
    game_state: Res<'_, State<GameState>>,
    mut game_state_mut: ResMut<'_, NextState<GameState>>,
    color_r: Res<'_, C>,
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<ColorMaterial>>,
    mut kind_free_q: Query<'_, '_, (&Kind, &CatanColor, &mut Left)>,
    mut interaction_query: Query<
        '_,
        '_,
        (
            &Pos,
            &Interaction,
            &mut BackgroundColor,
            &mut Button,
            &Resources,
        ),
        (Changed<Interaction>, Without<CatanColor>),
    >,
) {
    let current_color: CatanColor = (*color_r.into_inner()).into();
    for (entity, interaction, mut color, mut button, required_resources) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                button.set_changed();

                commands.spawn((Kind::default(), current_color, *entity));
                let kind_left = kind_free_q.iter_mut().find(|x| x.1 == &current_color);
                if let Some((_, _, mut left)) = kind_left {
                    *left = Left(left.0 - 1);
                }
                let player_resources = player_resources.iter_mut().find(|x| x.1 == &current_color);
                if let Some((mut resources, _)) = player_resources {
                    *resources -= *required_resources;
                }
                *resources += *required_resources;
                match *game_state.get() {
                    GameState::Nothing
                    | GameState::Monopoly
                    | GameState::YearOfPlenty
                    | GameState::Start
                    | GameState::Roll
                    | GameState::RoadBuilding
                    | GameState::Turn
                    | GameState::PlaceRobber
                    | GameState::RobberDiscardResources
                    | GameState::RobberPickColor => {}
                    GameState::PlaceRoad | GameState::PlaceTown | GameState::PlaceCity => {
                        game_state_mut.set(GameState::Turn);
                    }
                    GameState::SetupRoad => game_state_mut.set(GameState::SetupTown),
                    GameState::SetupTown => game_state_mut.set(GameState::SetupRoad),
                }
                commands.spawn(U::bundle(
                    *entity,
                    &mut meshes,
                    &mut materials,
                    current_color,
                ));
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

// not for initial game setup where the are no roads yet
// TODO: maybe we should impose an order on postions for stuff like roads so that comparing them is
// easeier (i.e. first postion is smallest ....)

#[derive(Component, PartialEq, Eq, Debug)]
struct Left(pub u8);

// town city "enherit" from building make some quries easier
// i think right way to do it with is with `[require(..)]`
#[derive(Component, PartialEq, Default, Clone, Copy)]
struct Building;

fn setup(
    mut next_state: ResMut<'_, NextState<GameState>>,
    mut commands: Commands<'_, '_>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let layout = layout(&mut commands);
    commands.insert_resource(layout);

    setup_game::setup(&mut commands, meshes, materials, layout);
    next_state.set(GameState::SetupRoad);

    // this has to be set dynamically
    commands.insert_resource(ColorIterator(
        vec![
            CatanColor::White,
            CatanColor::Red,
            CatanColor::Blue,
            CatanColor::Green,
        ]
        .into_iter()
        .cycle(),
    ));
    commands.insert_resource(SetupColorIterator(
        vec![
            CatanColor::White,
            CatanColor::Red,
            CatanColor::Blue,
            CatanColor::Green,
        ]
        .into_iter()
        .chain(
            vec![
                CatanColor::White,
                CatanColor::Red,
                CatanColor::Blue,
                CatanColor::Green,
            ]
            .into_iter()
            .rev(),
        ),
    ));
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct Layout {
    pub player_banner: Entity,
    pub development_cards: Entity,
    pub resources: Entity,
    pub board: Entity,
    pub ui: Entity,
    pub chat: Entity,
}
fn layout(commands: &mut Commands<'_, '_>) -> Layout {
    let player_banner_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(Color::BLACK),
            children![Text("banner".to_string()),],
        ))
        .id();
    let development_cards_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(Color::BLACK),
            children![Text("dev".to_string()),],
        ))
        .id();
    let resources_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(Color::BLACK),
            children![Text("res".to_string()),],
        ))
        .id();
    let mut card_layout = commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(85.), GridTrack::percent(15.)],
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BorderColor(Color::BLACK),
    ));
    card_layout.add_children(&[development_cards_layout, resources_layout]);
    let card_layout = card_layout.id();
    let board_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(Color::BLACK),
            children![Text("board".to_string()),],
        ))
        .id();
    let ui_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor(Color::BLACK),
            children![Text("ui".to_string()),],
        ))
        .id();
    let mut main_ui_layout = commands.spawn((
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::percent(90.), GridTrack::percent(10.)],
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BorderColor(Color::BLACK),
    ));
    main_ui_layout.add_children(&[board_layout, ui_layout]);
    let main_ui_layout = main_ui_layout.id();
    let chat_layout = commands
        .spawn((
            Node {
                display: Display::Grid,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            children![Text("chat".to_string()),],
            BorderColor(Color::BLACK),
        ))
        .id();
    let mut main_layout = commands.spawn((Node {
        display: Display::Grid,
        grid_template_columns: vec![
            GridTrack::percent(25.),
            GridTrack::percent(50.),
            GridTrack::percent(50.),
        ],
        ..default()
    },));

    main_layout.add_children(&[card_layout, main_ui_layout, chat_layout]);
    let main_layout = main_layout.id();
    let mut layout = commands.spawn((Node {
        display: Display::Grid,

        width: Val::Percent(100.),
        height: Val::Percent(100.),
        grid_template_rows: vec![GridTrack::percent(10.), GridTrack::percent(90.)],
        ..default()
    },));
    layout.add_children(&[player_banner_layout, main_layout]);
    Layout {
        player_banner: player_banner_layout,
        development_cards: development_cards_layout,
        resources: resources_layout,
        board: board_layout,
        ui: ui_layout,
        chat: chat_layout,
    }
}
pub fn find_with_color<'a, T>(
    c: &CatanColor,
    mut resources: impl Iterator<Item = (&'a CatanColor, T)>,
) -> Option<(&'a CatanColor, T)> {
    resources.find(|r| r.0 == c)
}
