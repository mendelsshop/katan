use bevy::prelude::*;

use crate::{
    BuildingPosition, CatanColor, GameState, Hexagon, Number, Position, Resources, Robber, Town,
    cities::City, turn_ui::DieButton,
};
use itertools::Itertools;
fn roll_dice() -> (u8, u8, u8) {
    let dice1 = rand::random_range(1..=6);
    let dice2 = rand::random_range(1..=6);
    (dice1 + dice2, dice1, dice2)
}
pub fn full_roll_dice(
    board: &Query<'_, '_, (&Hexagon, &Number, &Position)>,
    towns: &Query<'_, '_, (&Town, &CatanColor, &BuildingPosition)>,
    cities: &Query<'_, '_, (&City, &CatanColor, &BuildingPosition)>,
    player_resources: &mut Query<'_, '_, (&CatanColor, &mut Resources)>,
    resources: &mut ResMut<'_, Resources>,
    robber: Res<'_, Robber>,
    die_q: &mut Query<'_, '_, (&mut Text, &mut Transform), With<DieButton>>,
    game_state: &mut ResMut<'_, NextState<GameState>>,
) {
    let (roll, d1, d2) = roll_dice();
    // assumes two dice
    die_q
        .iter_mut()
        .zip([d1, d2])
        .for_each(|(mut die_ui, new_roll)| {
            *die_ui.1 = die_ui
                .1
                .with_rotation(Quat::from_rotation_z(rand::random_range((-25.)..(4.))));
            // TODO: maybe make dice move not only rotate

            **die_ui.0 = new_roll.to_string();
        });

    // TODO: what happens when 7 rolled
    if roll == 7 {
        if player_resources.iter().any(|r| r.1.count() > 7) {
            game_state.set(GameState::RobberDiscardResources);
        } else {
            game_state.set(GameState::PlaceRobber);
        }
    } else {
        // we only do this if no robber
        // if there is robber there a bunch of other states that me must go through
        game_state.set(GameState::Turn);
        distribute_resources(
            roll,
            board.iter().map(|(h, n, p)| (*h, *n, *p)),
            towns.iter().map(|(b, c, p)| (*b, *c, *p)),
            cities.iter().map(|(b, c, p)| (*b, *c, *p)),
            player_resources
                .iter_mut()
                .map(|(c, r)| (*c, r.into_inner())),
            resources,
            &robber,
        );
    }
}

fn distribute_resources<'a>(
    roll: u8,
    board: impl Iterator<Item = (Hexagon, Number, Position)> + Clone,
    towns: impl Iterator<Item = (Town, CatanColor, BuildingPosition)>,
    cities: impl Iterator<Item = (City, CatanColor, BuildingPosition)>,
    player_resources: impl Iterator<Item = (CatanColor, &'a mut Resources)>,
    resources: &mut Resources,
    robber: &Robber,
) {
    let mut player_resources = player_resources.collect_vec();
    let board = board.filter(|(_, number, p)| {
        p != &robber.0 && matches!(number, Number::Number(n) if *n == roll)
    });
    fn on_board_with_hex<Building>(
        board: impl Iterator<Item = (Hexagon, Number, Position)> + Clone,
        buildings: impl Iterator<Item = (Building, CatanColor, BuildingPosition)>,
    ) -> impl Iterator<Item = (Building, CatanColor, Hexagon)> {
        buildings.filter_map(move |(b, catan_color, BuildingPosition::All(p1, p2, p3))| {
            // does this need to be cloned
            board
                .clone()
                .find(|(_, _, pos)| pos == &p1 || pos == &p2 || pos == &p3)
                .map(|(hex, _, _)| (b, catan_color, hex))
        })
    }
    fn get_by_color<'a, T: 'a>(
        color: &CatanColor,
        mut things: impl Iterator<Item = &'a mut (CatanColor, T)>,
    ) -> Option<&'a mut T> {
        things.find(|(c, _)| c == color).map(|(_, t)| t)
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
        let player_resources = get_by_color(&color, player_resources.iter_mut());
        if let Some(player_resources) = player_resources {
            let gained = hexagon_to_resources(hex);
            **player_resources += gained;
            *resources -= gained;
        }
    }
    for (_, color, hex) in on_board_with_hex(board, cities) {
        let player_resources = get_by_color(&color, player_resources.iter_mut());
        if let Some(player_resources) = player_resources {
            let gained = hexagon_to_resources(hex) * 2;
            **player_resources += gained;
            *resources -= gained;
        }
    }
}
