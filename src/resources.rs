use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use bevy::prelude::*;
use rand::seq::IteratorRandom;

pub const CITY_RESOURCES: Resources = Resources {
    wood: 0,
    brick: 0,
    sheep: 0,
    wheat: 2,
    ore: 3,
};
pub const TOWN_RESOURCES: Resources = Resources {
    wood: 1,
    brick: 1,
    sheep: 1,
    wheat: 1,
    ore: 0,
};
pub const ROAD_RESOURCES: Resources = Resources {
    wood: 1,
    brick: 1,
    sheep: 0,
    wheat: 0,
    ore: 0,
};
pub const DEVELOPMENT_CARD_RESOURCES: Resources = Resources {
    wood: 1,
    brick: 1,
    sheep: 0,
    wheat: 0,
    ore: 0,
};
#[derive(Debug, Component, Resource, Clone, Copy, Default)]
pub struct Resources {
    pub wood: u8,
    pub brick: u8,
    pub sheep: u8,
    pub wheat: u8,
    pub ore: u8,
}

impl Sub for Resources {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            wood: self.wood - rhs.wood,
            brick: self.brick - rhs.brick,
            sheep: self.sheep - rhs.sheep,
            wheat: self.wheat - rhs.wheat,
            ore: self.ore - rhs.ore,
        }
    }
}
impl Mul<u8> for Resources {
    type Output = Self;

    fn mul(self, rhs: u8) -> Self::Output {
        Self {
            wood: self.wood * rhs,
            brick: self.brick * rhs,
            sheep: self.sheep * rhs,
            wheat: self.wheat * rhs,
            ore: self.ore * rhs,
        }
    }
}
impl SubAssign for Resources {
    fn sub_assign(&mut self, rhs: Self) {
        self.wood -= rhs.wood;
        self.brick -= rhs.brick;
        self.sheep -= rhs.sheep;
        self.wheat -= rhs.wheat;
        self.ore -= rhs.ore;
    }
}
impl AddAssign for Resources {
    fn add_assign(&mut self, rhs: Self) {
        self.wood += rhs.wood;
        self.brick += rhs.brick;
        self.sheep += rhs.sheep;
        self.wheat += rhs.wheat;
        self.ore += rhs.ore;
    }
}
impl Add for Resources {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            wood: self.wood + rhs.wood,
            brick: self.brick + rhs.brick,
            sheep: self.sheep + rhs.sheep,
            wheat: self.wheat + rhs.wheat,
            ore: self.ore + rhs.ore,
        }
    }
}

impl Resources {
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        // applicative would be really nice for this no need for deep nesting
        self.wood.checked_sub(rhs.wood).and_then(|wood| {
            self.brick.checked_sub(rhs.brick).and_then(|brick| {
                self.sheep.checked_sub(rhs.sheep).and_then(|sheep| {
                    self.ore.checked_sub(rhs.ore).and_then(|ore| {
                        self.wheat.checked_sub(rhs.wheat).map(|wheat| Self {
                            wood,
                            brick,
                            sheep,
                            wheat,
                            ore,
                        })
                    })
                })
            })
        })
    }
    const fn count(self) -> u8 {
        self.wood + self.brick + self.sheep + self.wheat + self.ore
    }
    #[must_use]
    pub const fn contains(self, rhs: Self) -> bool {
        self.wood >= rhs.wood
            && self.brick >= rhs.brick
            && self.sheep >= rhs.sheep
            && self.wheat >= rhs.wheat
            && self.ore >= rhs.ore
    }
    #[must_use]
    pub const fn new_player() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }
    #[must_use]
    pub const fn new_game() -> Self {
        Self::new(19, 19, 19, 19, 19)
    }
    #[must_use]
    pub const fn new(wood: u8, brick: u8, sheep: u8, wheat: u8, ore: u8) -> Self {
        Self {
            wood,
            brick,
            sheep,
            wheat,
            ore,
        }
    }
}
/// assumption: other player has at least on resource
pub fn take_resource(
    current_color_resource: &mut Resources,
    other_color_resources: &mut Resources,
) {
    let possible_resources_to_take = [
        Resources {
            wood: 1,
            brick: 0,
            sheep: 0,
            wheat: 0,
            ore: 0,
        },
        Resources {
            wood: 0,
            brick: 1,
            sheep: 0,
            wheat: 0,
            ore: 0,
        },
        Resources {
            wood: 0,
            brick: 0,
            sheep: 1,
            wheat: 0,
            ore: 0,
        },
        Resources {
            wood: 0,
            brick: 0,
            sheep: 0,
            wheat: 1,
            ore: 0,
        },
        Resources {
            wood: 0,
            brick: 0,
            sheep: 0,
            wheat: 0,
            ore: 1,
        },
    ]
    .into_iter()
    // verifiing that the player has the resources we are trying to take randomly
    .filter(|r| other_color_resources.checked_sub(*r).is_some())
    .choose(&mut rand::rng());
    let resources_to_take = possible_resources_to_take.unwrap();
    *other_color_resources -= resources_to_take;
    *current_color_resource += resources_to_take;
}
