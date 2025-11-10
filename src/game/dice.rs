use bevy::prelude::*;

use super::{
    CatanColor, GameState, Hexagon, Input, Number, Resources, Robber,
    cities::City,
    positions::{BuildingPosition, Position},
    towns::Town,
    turn_ui::DieButton,
};
fn roll_dice() -> (u8, u8, u8) {
    let dice1 = rand::random_range(1..=6);
    let dice2 = rand::random_range(1..=6);
    (dice1 + dice2, dice1, dice2)
}
pub fn full_roll_dice(
    player_resources: Query<'_, '_, &mut Resources, With<CatanColor>>,
    mut game_state: ResMut<'_, NextState<GameState>>,
    mut input: ResMut<'_, Input>,
) {
    let (roll, d1, d2) = roll_dice();
    // assumes two dice

    // TODO: what happens when 7 rolled
    if roll == 7 {
        if player_resources.iter().any(|r| r.count() > 7) {
            *input = Input::RobberDiscardInit;
        } else {
            game_state.set(GameState::PlaceRobber);
        }
    } else {
        // we only do this if no robber
        // if there is robber there a bunch of other states that me must go through
        game_state.set(GameState::Turn);
        *input = Input::Roll(roll, d1, d2);
    }
}

pub fn update_dice(
    die_q: &mut Query<'_, '_, (&mut Text, &mut Transform), With<DieButton>>,
    d1: u8,
    d2: u8,
) {
    die_q
        .iter_mut()
        .zip([d1, d2])
        .for_each(|(mut die_ui, new_roll)| {
            *die_ui.1 = die_ui
                .1
                .with_rotation(Quat::from_rotation_z(rand::random_range((-25.)..4.)));
            // TODO: maybe make dice move not only rotate

            **die_ui.0 = new_roll.to_string();
        });
}

pub fn distribute_resources<'a>(
    roll: u8,

    board: Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: Query<'_, '_, (&ChildOf, &Town, &BuildingPosition), With<CatanColor>>,
    cities: Query<'_, '_, (&ChildOf, &City, &BuildingPosition), With<CatanColor>>,
    mut player_resources: Query<'_, '_, &mut Resources, With<CatanColor>>,
    mut resources: ResMut<'_, Resources>,
    robber: Res<'_, Robber>,
) {
    // TODO: maybe each placed town/city should have entity pointing to all surrounding hexes
    let board = board
        .into_iter()
        .filter(|(_, number, p)| {
            p != &&robber.0 && matches!(number, Number::Number(n) if *n == roll)
        })
        .map(|(h, n, p)| (*h, *n, *p));
    fn on_board_with_hex<Building: Component + Copy>(
        board: impl Iterator<Item = (Hexagon, Number, Position)> + Clone,
        buildings: Query<'_, '_, (&ChildOf, &Building, &BuildingPosition), With<CatanColor>>,
    ) -> impl Iterator<Item = (Building, Entity, Hexagon)> {
        buildings.into_iter().filter_map(
            move |(catan_color_entity, b, BuildingPosition::All(p1, p2, p3))| {
                // does this need to be cloned
                board
                    .clone()
                    .find(|(_, _, pos)| pos == p1 || pos == p2 || pos == p3)
                    .map(|(hex, _, _)| (*b, catan_color_entity.parent(), hex))
            },
        )
    }

    fn hexagon_to_resources(hex: Hexagon) -> Resources {
        match hex {
            Hexagon::Wood => Resources {
                wood: 1,
                brick: 0,
                sheep: 0,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Brick => Resources {
                wood: 0,
                brick: 1,
                sheep: 0,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Sheep => Resources {
                wood: 0,
                brick: 0,
                sheep: 1,
                wheat: 0,
                ore: 0,
            },
            Hexagon::Wheat => Resources {
                wood: 0,
                brick: 0,
                sheep: 0,
                wheat: 1,
                ore: 0,
            },
            Hexagon::Ore => Resources {
                wood: 0,
                brick: 0,
                sheep: 0,
                wheat: 0,
                ore: 1,
            },
            Hexagon::Desert => todo!(),
            Hexagon::Water => todo!(),
            Hexagon::Port => todo!(),
            Hexagon::Empty => todo!(),
        }
    }
    for (_, color, hex) in on_board_with_hex(board.clone(), towns) {
        let player_resources = player_resources.get_mut(color).ok();
        if let Some(mut player_resources) = player_resources {
            let gained = hexagon_to_resources(hex);
            *player_resources += gained;
            *resources -= gained;
        }
    }
    for (_, color, hex) in on_board_with_hex(board, cities) {
        let player_resources = player_resources.get_mut(color).ok();
        if let Some(mut player_resources) = player_resources {
            let gained = hexagon_to_resources(hex) * 2;
            *player_resources += gained;
            *resources -= gained;
        }
    }
}
