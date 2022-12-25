use std::{
    collections::VecDeque,
    ops::{Add, Sub},
};

use anyhow::Context;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::Day;

pub static RUN: Day = Day { part1, part2 };

// TODO: optimize - hashmaps are probably slow

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut elves_vec = parse_input(input);

    let mut dirs: VecDeque<_> = vec![Dir::North, Dir::South, Dir::West, Dir::East].into();

    let mut elves: FxHashSet<Vec2<i32>> = elves_vec.drain(..).collect();
    let mut proposed = FxHashMap::default();
    let mut proposals = FxHashMap::default();

    //debug_print(&elves);

    // Rounds
    for _ in 0..10 {
        // Phase 1 - Propose
        for elve in elves.iter() {
            let has_neighbors = NEIGHBORS
                .into_iter()
                .map(|offset| *elve + offset)
                .any(|neighbor| elves.contains(&neighbor));

            let mut proposal = None;
            if has_neighbors {
                // check directions
                for dir in dirs.iter() {
                    let (walk, neighborhood) = match dir {
                        Dir::North => (Vec2::new(0, -1), &NORTH_NEIGHBORS),
                        Dir::East => (Vec2::new(1, 0), &EAST_NEIGHBORS),
                        Dir::South => (Vec2::new(0, 1), &SOUTH_NEIGHBORS),
                        Dir::West => (Vec2::new(-1, 0), &WEST_NEIGHBORS),
                    };
                    let is_free = !neighborhood
                        .iter()
                        .map(|offset| *elve + *offset)
                        .any(|neighbor| elves.contains(&neighbor));

                    if is_free {
                        proposal = Some(*elve + walk);
                        break;
                    }
                }
            }

            if let Some(proposal) = proposal {
                proposals.insert(*elve, proposal);
                proposed
                    .entry(proposal)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        // Phase 2 - execute
        for elve in elves.drain() {
            match proposals.get(&elve) {
                Some(new_pos) if proposed.get(new_pos).copied().unwrap_or(0) <= 1 => {
                    // move
                    elves_vec.push(*new_pos);
                }
                _ => {
                    // stay
                    elves_vec.push(elve);
                }
            }
        }

        // Clean up
        proposals.clear();
        proposed.clear();
        for elve in elves_vec.drain(..) {
            let unique = elves.insert(elve);
            assert!(unique);
        }

        //debug_print(&elves);

        dirs.rotate_left(1);
    }

    // compute AABB
    let (min, max) = aabb(elves.iter()).context("no elves, no aabb")?;

    let free = (max.x - min.x + 1) * (max.y - min.y + 1) - elves.len() as i32;

    Ok(free.to_string())
}

fn aabb<'a>(mut elves_iter: impl Iterator<Item = &'a Vec2<i32>>) -> Option<(Vec2<i32>, Vec2<i32>)> {
    let first = elves_iter.next().copied()?;
    let (min, max) = elves_iter.fold((first, first), |(min, max), elve| {
        (min.zip_with(*elve, Ord::min), max.zip_with(*elve, Ord::max))
    });
    Some((min, max))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut elves_vec = parse_input(input);

    let mut dirs: VecDeque<_> = vec![Dir::North, Dir::South, Dir::West, Dir::East].into();

    let mut elves: FxHashSet<Vec2<i32>> = elves_vec.drain(..).collect();
    let mut proposed = FxHashMap::default();
    let mut proposals = FxHashMap::default();

    // Rounds
    let mut round = 0;
    let mut any_moved = true;
    while any_moved {
        round += 1;
        any_moved = false;

        // Phase 1 - Propose
        for elve in elves.iter() {
            let has_neighbors = NEIGHBORS
                .into_iter()
                .map(|offset| *elve + offset)
                .any(|neighbor| elves.contains(&neighbor));

            let mut proposal = None;
            if has_neighbors {
                // check directions
                for dir in dirs.iter() {
                    let (walk, neighborhood) = match dir {
                        Dir::North => (Vec2::new(0, -1), &NORTH_NEIGHBORS),
                        Dir::East => (Vec2::new(1, 0), &EAST_NEIGHBORS),
                        Dir::South => (Vec2::new(0, 1), &SOUTH_NEIGHBORS),
                        Dir::West => (Vec2::new(-1, 0), &WEST_NEIGHBORS),
                    };
                    let is_free = !neighborhood
                        .iter()
                        .map(|offset| *elve + *offset)
                        .any(|neighbor| elves.contains(&neighbor));

                    if is_free {
                        proposal = Some(*elve + walk);
                        break;
                    }
                }
            }

            if let Some(proposal) = proposal {
                proposals.insert(*elve, proposal);
                proposed
                    .entry(proposal)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        // Phase 2 - execute
        for elve in elves.drain() {
            match proposals.get(&elve) {
                Some(new_pos) if proposed.get(new_pos).copied().unwrap_or(0) <= 1 => {
                    // move
                    elves_vec.push(*new_pos);
                    any_moved = true;
                }
                _ => {
                    // stay
                    elves_vec.push(elve);
                }
            }
        }

        // Clean up
        proposals.clear();
        proposed.clear();
        for elve in elves_vec.drain(..) {
            let unique = elves.insert(elve);
            assert!(unique);
        }

        //debug_print(&elves);

        dirs.rotate_left(1);
    }

    Ok(round.to_string())
}

fn parse_input(input: &[u8]) -> Vec<Vec2<i32>> {
    let mut elves_vec = Vec::new();
    for (y, line) in input.split(|ch| *ch == b'\n').enumerate() {
        for (x, ch) in line.iter().enumerate() {
            if *ch == b'#' {
                elves_vec.push(Vec2::new(x as i32, y as i32))
            }
        }
    }
    elves_vec
}

#[allow(unused)]
fn debug_print(elves: &FxHashSet<Vec2<i32>>) {
    let (min, max) = aabb(elves.iter()).unwrap();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            print!(
                "{}",
                if elves.contains(&Vec2::new(x, y)) {
                    '#'
                } else {
                    '.'
                }
            );
        }
        println!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    North,
    East,
    South,
    West,
}

const NEIGHBORS: [Vec2<i32>; 8] = [
    Vec2::new(-1, -1),
    Vec2::new(0, -1),
    Vec2::new(1, -1),
    Vec2::new(1, 0),
    Vec2::new(1, 1),
    Vec2::new(0, 1),
    Vec2::new(-1, 1),
    Vec2::new(-1, 0),
];
const NORTH_NEIGHBORS: [Vec2<i32>; 3] = [Vec2::new(-1, -1), Vec2::new(0, -1), Vec2::new(1, -1)];
const EAST_NEIGHBORS: [Vec2<i32>; 3] = [Vec2::new(1, -1), Vec2::new(1, 0), Vec2::new(1, 1)];
const WEST_NEIGHBORS: [Vec2<i32>; 3] = [Vec2::new(-1, -1), Vec2::new(-1, 0), Vec2::new(-1, 1)];
const SOUTH_NEIGHBORS: [Vec2<i32>; 3] = [Vec2::new(1, 1), Vec2::new(0, 1), Vec2::new(-1, 1)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn zip_with<S, R>(self, other: Vec2<S>, mut f: impl FnMut(T, S) -> R) -> Vec2<R> {
        Vec2 {
            x: f(self.x, other.x),
            y: f(self.y, other.y),
        }
    }

    pub fn map<R>(self, mut f: impl FnMut(T) -> R) -> Vec2<R> {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

impl<T: Default> Default for Vec2<T> {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

impl<T: Add<T>> Add<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;

    fn add(self, rhs: Vec2<T>) -> Self::Output {
        self.zip_with(rhs, Add::add)
    }
}

impl<T: Sub<T>> Sub<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;

    fn sub(self, rhs: Vec2<T>) -> Self::Output {
        self.zip_with(rhs, Sub::sub)
    }
}

crate::test_day!(RUN, "day23", "4249", "980");
