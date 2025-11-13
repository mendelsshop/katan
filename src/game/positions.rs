use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    mem,
    ops::{Add, Div},
};

use super::KatanComponent;
use bevy::prelude::*;
#[derive(
    Component, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct Position {
    pub q: i8,
    pub r: i8,
    pub s: i8,
}
pub fn generate_postions_ring(n: i8) -> impl Iterator<Item = Position> {
    let has_big_coordinate = move |i: i8| i == -n || i == n;
    generate_postions(n + 1).filter(move |q| {
        has_big_coordinate(q.q) || has_big_coordinate(q.r) || has_big_coordinate(q.s)
    })
}
pub fn generate_postions(n: i8) -> impl Iterator<Item = Position> {
    (0..3)
        .map(|_| -n + 1..n)
        .multi_cartesian_product()
        .filter(|q| q[0] + q[1] + q[2] == 0)
        .map(|i| Position {
            q: i[0],
            r: i[1],
            s: i[2],
        })
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FPosition {
    pub q: f32,
    pub r: f32,
    pub s: f32,
}
impl From<Position> for FPosition {
    fn from(Position { q, r, s }: Position) -> Self {
        Self {
            q: f32::from(q),
            r: f32::from(r),
            s: f32::from(s),
        }
    }
}
impl FPosition {
    pub const fn filter_coordinate(mut self, coordinate: Coordinate) -> Self {
        match coordinate {
            Coordinate::Q => self.q = 0.,
            Coordinate::R => self.r = 0.,
            Coordinate::S => self.s = 0.,
        }
        self
    }
    pub const fn get_shared_coordinate(&self, other: &Self) -> Option<Coordinate> {
        if self.q == other.q {
            Some(Coordinate::Q)
        } else if self.r == other.r {
            Some(Coordinate::R)
        } else if self.s == other.s {
            Some(Coordinate::S)
        } else {
            None
        }
    }
    pub fn intersect(self, other: Self) -> Option<Self> {
        self.get_shared_coordinate(&other)
            .map(|shared_coordinate| self.interesect_with_coordinate(other, shared_coordinate))
    }

    pub const fn interesect_with_coordinate(
        self,
        Self {
            q: q1,
            r: r1,
            s: s1,
        }: Self,
        shared_coordinate: Coordinate,
    ) -> Self {
        let Self { q, r, s } = self;
        match shared_coordinate {
            Coordinate::Q => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    q,
                    r: f32::midpoint(r, r1),
                    s: f32::midpoint(s, s1),
                }
            }
            Coordinate::R => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    r,
                    q: f32::midpoint(q, q1),
                    s: f32::midpoint(s, s1),
                }
            }
            Coordinate::S => {
                // ideas is that the midpoint will be here the road is between two hexes
                // doesn't seem to be working
                Self {
                    s,
                    r: f32::midpoint(r, r1),
                    q: f32::midpoint(q, q1),
                }
            }
        }
    }
    pub fn hex_to_pixel(self) -> (f32, f32) {
        // let x = 3f32.sqrt().mul_add(self.q, 3f32.sqrt() / 2. * self.r);
        let y = -((3. / 2.) * self.r);
        let x = 3f32.sqrt().mul_add(self.q, (3f32.sqrt() / 2.) * self.r);
        (x, y)
    }
}
// maybe do size const generics?
impl Position {
    pub const DIRECTION_VECTORS: [Self; 6] = [
        Self { q: 1, r: 0, s: -1 },
        Self { q: 1, r: -1, s: 0 },
        Self { q: 0, r: -1, s: 1 },
        Self { q: -1, r: 0, s: 1 },
        Self { q: -1, r: 1, s: 0 },
        Self { q: 0, r: 1, s: -1 },
    ];
    pub fn rotate_right(&self) -> Self {
        let Self { q, r, s } = self;
        Self {
            q: -r,
            r: -s,
            s: -q,
        }
    }
    fn rotate_right_n(&self, n: u8) -> Self {
        (0..n).fold(*self, |this, _| this.rotate_right())
    }
    pub fn building_positions_around(&self) -> [BuildingPosition; 6] {
        Self::DIRECTION_VECTORS.map(|p| {
            let p1 = p.rotate_right();
            unsafe { BuildingPosition::new_unchecked(*self, p + *self, p1 + *self) }
        })
    }
    pub fn all_points_are(&self, mut f: impl FnMut(i8) -> bool) -> bool {
        f(self.q) && f(self.r) && f(self.s)
    }
    pub fn any_points_is(&self, mut f: impl FnMut(i8) -> bool) -> bool {
        f(self.q) || f(self.r) || f(self.s)
    }
    pub const fn get_shared_coordinate(&self, other: &Self) -> Option<Coordinate> {
        if self.q == other.q {
            Some(Coordinate::Q)
        } else if self.r == other.r {
            Some(Coordinate::R)
        } else if self.s == other.s {
            Some(Coordinate::S)
        } else {
            None
        }
    }

    // TODO: maybe this should be a result as their are two possiblities for failure
    // 1) it doesn't add uo to 0
    // 2) its out of the board
    pub fn new(q: i8, r: i8, s: i8, size: Option<u8>) -> Option<Self> {
        const fn in_between(bound: u8, point: i8) -> bool {
            let bound = (bound) as i8;
            -bound <= point && point <= bound
        }
        (q + r + s == 0
            && size.is_none_or(|size| {
                in_between(size, q) && in_between(size, r) && in_between(size, s)
            }))
        .then_some(Self { q, r, s })
    }
}
impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}
impl Div<f32> for FPosition {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            q: self.q / rhs,
            r: self.r / rhs,
            s: self.s / rhs,
        }
    }
}
impl Add for FPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
            s: self.s + rhs.s,
        }
    }
}
#[derive(Component, Clone, Copy, Debug, Hash, Eq, Deserialize, Serialize)]
#[require(KatanComponent)]
pub enum RoadPosition {
    /// Dont use this constructor use `Self::new`
    Both(Position, Position, Coordinate),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum Coordinate {
    Q,
    R,
    S,
}
impl RoadPosition {
    // for creating none edge roads
    pub fn new(p1: Position, p2: Position, size: Option<u8>) -> Option<Self> {
        let not_off_board = size.is_none_or(|size| {
            p1.all_points_are(|p| -(size as i8) < p && p < size as i8)
                || p2.all_points_are(|p| -(size as i8) < p && p < size as i8)
        });
        // veifies that the two roads boredering each other
        let c = p1
            .get_shared_coordinate(&p2)
            .filter(|c| other_point_is_close(p1, p2, c) && not_off_board);
        c.map(|c| Self::Both(p1, p2, c))
    }
    pub fn intersect(&self, other: &Self, size: Option<u8>) -> Option<BuildingPosition> {
        match (self, other) {
            (
                Self::Both(position_road, position1_road, coordinate_road),
                Self::Both(position_road1, position1_road1, coordinate_road1),
            ) => {
                if coordinate_road == coordinate_road1 {
                    None
                } else if position_road == position_road1 || position1_road == position_road1 {
                    BuildingPosition::new(*position_road, *position1_road, *position1_road1, size)
                } else if position_road == position1_road1 || position1_road == position1_road1 {
                    BuildingPosition::new(*position_road, *position1_road, *position_road1, size)
                } else {
                    None
                }
            }
        }
    }
    pub fn neighboring_two(&self, size: Option<u8>) -> (Option<Position>, Option<Position>) {
        match self {
            Self::Both(p1, p2, coordinate) => {
                // maybe just do permutations of two other point that add up to 0
                match coordinate {
                    Coordinate::Q => (
                        Position::new(p1.q + 1, p1.r.min(p2.r), p1.s.min(p2.s), size),
                        Position::new(p1.q - 1, p1.r.max(p2.r), p1.s.max(p2.s), size),
                    ),
                    Coordinate::R => (
                        Position::new(p1.q.min(p2.q), p1.r + 1, p1.s.min(p2.s), size),
                        Position::new(p1.q.max(p2.q), p1.r - 1, p1.s.max(p2.s), size),
                    ),
                    Coordinate::S => (
                        Position::new(p1.q.min(p2.q), p1.r.min(p2.r), p1.s + 1, size),
                        Position::new(p1.q.max(p2.q), p1.r.max(p2.r), p1.s - 1, size),
                    ),
                }
            }
        }
    }
    pub const fn shared_coordinate(&self) -> Coordinate {
        match self {
            Self::Both(_, _, coordinate) => *coordinate,
        }
    }
    pub fn positon_to_pixel_coordinates(&self) -> (f32, f32) {
        match self {
            Self::Both(position, position1, coordinate) => {
                let fposition: FPosition = (*position).into();
                let fposition1: FPosition = (*position1).into();
                let fposition2 = (fposition1 + fposition) / 2.;
                // maybe issue is you cant do math like this and expect pixel to hex to still work?

                fposition2.hex_to_pixel()
                // fposition
                //     .interesect_with_coordinate((*position1).into(), *coordinate)
                //     .hex_to_pixel()
            }
        }
    }
}

const fn other_point_is_close(p1: Position, p2: Position, c: &Coordinate) -> bool {
    match c {
        Coordinate::Q => p1.r.abs_diff(p2.r) <= 1 && p1.s.abs_diff(p2.s) <= 1,
        Coordinate::R => p1.q.abs_diff(p2.q) <= 1 && p1.s.abs_diff(p2.s) <= 1,
        Coordinate::S => p1.r.abs_diff(p2.r) <= 1 && p1.q.abs_diff(p2.q) <= 1,
    }
}

impl PartialEq for RoadPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Both(l0, l1, l2), Self::Both(r0, r1, r2)) => {
                ((l0 == r0 && l1 == r1) || (l0 == r1 && l1 == r0)) && l2 == r2
            }
            _ => false,
        }
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Deserialize, Serialize, Debug, Resource)]
#[require(KatanComponent)]
pub enum BuildingPosition {
    /// Do not use this
    /// so we can garuntee p1 > p2 > p3
    All(Position, Position, Position),
}

impl BuildingPosition {
    pub fn new(p1: Position, p2: Position, p3: Position, size: Option<u8>) -> Option<Self> {
        let not_off_board = size.is_none_or(|size| {
            p1.all_points_are(|p| -(size as i8) < p && p < size as i8)
                || p2.all_points_are(|p| -(size as i8) < p && p < size as i8)
                || p3.all_points_are(|p| -(size as i8) < p && p < size as i8)
        });
        let do_share_points = p1.get_shared_coordinate(&p2).is_some_and(|p1p2| {
            other_point_is_close(p1, p3, &p1p2)
                && p2.get_shared_coordinate(&p3).is_some_and(|p2p3| {
                    other_point_is_close(p2, p1, &p2p3)
                        && p1p2 != p2p3
                        && p1.get_shared_coordinate(&p3).is_some_and(|p1p3| {
                            other_point_is_close(p3, p2, &p1p3) && p1p2 != p1p3 && p1p3 != p2p3
                        })
                })
        });
        (not_off_board && do_share_points).then_some(unsafe { Self::new_unchecked(p1, p2, p3) })
    }
    pub fn rotate_right(&self) -> Self {
        match self {
            Self::All(position, position1, position2) => unsafe {
                Self::new_unchecked(
                    position.rotate_right(),
                    position1.rotate_right(),
                    position2.rotate_right(),
                )
            },
        }
    }

    pub fn rotate_right_n(&self, n: u8) -> Self {
        match self {
            Self::All(position, position1, position2) => unsafe {
                Self::new_unchecked(
                    position.rotate_right_n(n),
                    position1.rotate_right_n(n),
                    position2.rotate_right_n(n),
                )
            },
        }
    }
    pub unsafe fn new_unchecked(mut p1: Position, mut p2: Position, mut p3: Position) -> Self {
        if p2 < p3 {
            mem::swap(&mut p2, &mut p3);
        }
        if p1 < p2 {
            mem::swap(&mut p1, &mut p2);
        }
        // we swap p2 and p3 as p1 might've been < than p3
        if p2 < p3 {
            mem::swap(&mut p2, &mut p3);
        }
        Self::All(p1, p2, p3)
    }
    pub fn positon_to_pixel_coordinates(&self) -> (f32, f32) {
        match self {
            Self::All(position, position1, position2) => {
                let fposition: FPosition = (*position).into();
                let fposition1: FPosition = (*position1).into();
                let fposition2: FPosition = (*position2).into();
                ((fposition + fposition1 + fposition2) / 3.).hex_to_pixel()
            }
        }
    }
    pub fn contains(&self, pos: &Position) -> bool {
        match self {
            Self::All(position, position1, position2) => {
                position == pos || position1 == pos || position2 == pos
            }
        }
    }
}

impl Add<Position> for BuildingPosition {
    type Output = Self;
    fn add(self, rhs: Position) -> Self::Output {
        match self {
            Self::All(position, position1, position2) => unsafe {
                Self::new_unchecked(position + rhs, position1 + rhs, position2 + rhs)
            },
        }
    }
}
