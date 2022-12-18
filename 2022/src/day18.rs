use std::ops::{Add, Sub};

use nom::bytes::complete::tag;
use nom::multi::many0;
use nom::sequence::{terminated, tuple};
use nom::IResult;
use nom::{character::complete::i32 as parse_i32, combinator::map};
use rustc_hash::FxHashSet;

use crate::parsers::newline;
use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let droplets = parsers::parse(many0(terminated(parse_pos, newline)), input)?;

    // TODO: evaluate using Array3 rather than hash set
    let dropset: FxHashSet<Vec3<i32>> = droplets.into_iter().collect();

    let out: usize = dropset
        .iter()
        .map(|drop| {
            SIDES
                .iter()
                .filter(|side| !dropset.contains(&(*drop + **side)))
                .count()
        })
        .sum();

    Ok(out.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let droplets = parsers::parse(many0(terminated(parse_pos, newline)), input)?;

    let (min, max) =
        droplets
            .iter()
            .fold((Vec3::default(), Vec3::default()), |(min, max), droplet| {
                (
                    min.zip_with(*droplet, Ord::min),
                    max.zip_with(*droplet, Ord::max),
                )
            });

    let dropset: FxHashSet<Vec3<i32>> = droplets.into_iter().collect();

    let mut todo = vec![];

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            todo.push(Vec3::new(x, y, min.z));
            todo.push(Vec3::new(x, y, max.z));
        }
    }
    for x in min.x..=max.x {
        for z in min.z..=max.z {
            todo.push(Vec3::new(x, min.y, z));
            todo.push(Vec3::new(x, max.y, z));
        }
    }
    for y in min.y..=max.y {
        for z in min.z..=max.z {
            todo.push(Vec3::new(min.x, y, z));
            todo.push(Vec3::new(max.x, y, z));
        }
    }

    let mut outside: FxHashSet<Vec3<i32>> = FxHashSet::default();

    for i in (0..todo.len()).rev() {
        if dropset.contains(&todo[i]) {
            // Not on the outside
            todo.swap_remove(i);
        } else {
            // On the outside
            outside.insert(todo[i]);
        }
    }

    while let Some(cube) = todo.pop() {
        // cube was on the outside, process neighbors
        for side in SIDES {
            let neighbor = cube + side;

            let in_range = neighbor.zip_with(min, |n, bound| n >= bound).and()
                && neighbor.zip_with(max, |n, bound| n <= bound).and();

            if in_range && !dropset.contains(&neighbor) && !outside.contains(&neighbor) {
                outside.insert(neighbor);
                todo.push(neighbor);
            }
        }
    }

    let is_outside = |cube: Vec3<i32>| {
        let in_range = cube.zip_with(min, |n, bound| n >= bound).and()
            && cube.zip_with(max, |n, bound| n <= bound).and();

        !in_range || outside.contains(&cube)
    };

    let out: usize = dropset
        .iter()
        .map(|drop| {
            SIDES
                .iter()
                .filter(|side| {
                    let neighbor = *drop + **side;
                    let is_air = !dropset.contains(&neighbor);
                    is_air && is_outside(neighbor)
                })
                .count()
        })
        .sum();

    Ok(out.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn zip_with<S, R>(self, other: Vec3<S>, mut f: impl FnMut(T, S) -> R) -> Vec3<R> {
        Vec3 {
            x: f(self.x, other.x),
            y: f(self.y, other.y),
            z: f(self.z, other.z),
        }
    }

    pub fn all(self, mut f: impl FnMut(&T) -> bool) -> bool {
        f(&self.x) && f(&self.y) && f(&self.z)
    }
}

impl Vec3<bool> {
    pub fn and(self) -> bool {
        self.x && self.y && self.z
    }
}

impl<T: Default> Default for Vec3<T> {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
        }
    }
}

impl<T: Add<T>> Add<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T::Output>;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        self.zip_with(rhs, Add::add)
    }
}

impl<T: Sub<T>> Sub<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T::Output>;

    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        self.zip_with(rhs, Sub::sub)
    }
}

static SIDES: [Vec3<i32>; 6] = [
    Vec3::new(1, 0, 0),
    Vec3::new(-1, 0, 0),
    Vec3::new(0, 1, 0),
    Vec3::new(0, -1, 0),
    Vec3::new(0, 0, 1),
    Vec3::new(0, 0, -1),
];

fn parse_pos(input: &[u8]) -> IResult<&[u8], Vec3<i32>> {
    map(
        tuple((
            (terminated(parse_i32, tag(","))),
            (terminated(parse_i32, tag(","))),
            parse_i32,
        )),
        |(x, y, z)| Vec3 { x, y, z },
    )(input)
}

crate::test_day!(RUN, "day18", "3494", "2062");
