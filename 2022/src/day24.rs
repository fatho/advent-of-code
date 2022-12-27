use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fmt::{Display, Write},
    ops::{Index, IndexMut},
};

use anyhow::Context;
use nom::{
    bytes::complete::take,
    combinator::map_opt,
    multi::{fold_many1, fold_many_m_n, many0},
    sequence::terminated,
    IResult,
};
use rustc_hash::FxHashSet;

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(parse_map, input)?;

    let features = extract_features(&map)?;

    let mut maps_over_time = Vec::new();
    let steps = compute_path(
        &mut maps_over_time,
        &features,
        State {
            time: 0,
            pos: features.entrance,
        },
        features.exit,
    )
    .context("no path")?
    .time;

    Ok(steps.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(parse_map, input)?;
    let features = extract_features(&map)?;

    // iterative deepening sarch
    let mut maps_over_time = Vec::new();

    let initial = State {
        time: 0,
        pos: features.entrance,
    };
    let at_exit = compute_path(&mut maps_over_time, &features, initial, features.exit)
        .context("no path to exit")?;

    let at_entrance = compute_path(&mut maps_over_time, &features, at_exit, features.entrance)
        .context("no path back to entrace")?;

    let at_exit_again = compute_path(&mut maps_over_time, &features, at_entrance, features.exit)
        .context("no path back to exit")?;

    Ok(at_exit_again.time.to_string())
}

/// Abstract representation of the map.
struct MapFeatures {
    width: u32,
    height: u32,
    entrance: (u32, u32),
    exit: (u32, u32),
    blizzards: Vec<(Dir, (u32, u32))>,
}

/// A*: pathing through changing 2D map is interpreted as pathing through
/// static 3D map (with time being the third dimension).
fn compute_path(
    maps_over_time: &mut Vec<Map<Tile>>,
    features: &MapFeatures,
    from: State,
    to: (u32, u32),
) -> Option<State> {
    let mut open = BinaryHeap::new();
    let mut closed = FxHashSet::default();

    open.push((Reverse(0), from));

    while let Some((_, cur)) = open.pop() {
        if !closed.insert(cur) {
            continue;
        }
        // goal
        if cur.pos == to {
            return Some(cur);
        }
        // compute blizzard positions at next step
        let new_time = cur.time + 1;
        precompute_maps(maps_over_time, new_time, features);
        // visit neighbours or wait
        let (x, y) = cur.pos;

        for (dx, dy) in [(1, 0), (0, 1), (0, 0), (0, -1), (-1, 0)] {
            let nx = (x as i32) + dx;
            let ny = (y as i32) + dy;
            if nx < 0 || ny < 0 || nx >= features.width as i32 || ny >= features.height as i32 {
                continue;
            }
            let nx = nx as u32;
            let ny = ny as u32;

            // check if viable
            let new_map = &maps_over_time[new_time as usize];
            if matches!(new_map[(nx, ny)], Tile::Open) {
                let new_dist = new_time + manhattan((nx, ny), features.exit);
                open.push((
                    Reverse(new_dist),
                    State {
                        time: new_time,
                        pos: (nx, ny),
                    },
                ))
            }
        }
    }
    None
}

fn extract_features(map: &Map<Tile>) -> anyhow::Result<MapFeatures> {
    let entrance_x = (0..map.width - 1)
        .map(|x| map[(x, 0)])
        .position(|tile| matches!(tile, Tile::Open))
        .context("no entry")? as u32;
    let exit_x = (0..map.width - 1)
        .map(|x| map[(x, map.height - 1)])
        .position(|tile| matches!(tile, Tile::Open))
        .context("no exit")? as u32;

    let entrance = (entrance_x, 0);
    let exit = (exit_x, map.height - 1);

    let mut blizzards = Vec::new();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            if let Tile::Blizzard(dir) = map[(x, y)] {
                blizzards.push((dir, (x, y)));
            }
        }
    }

    Ok(MapFeatures {
        width: map.width,
        height: map.height,
        entrance,
        exit,
        blizzards,
    })
}

fn precompute_maps(maps_over_time: &mut Vec<Map<Tile>>, max_time: u32, features: &MapFeatures) {
    let width = features.width;
    let height = features.height;

    for time in maps_over_time.len()..max_time as usize + 1 {
        let mut tmp = Map::new(width, height, Tile::Open);
        for y in 0..height {
            tmp[(0, y)] = Tile::Wall;
            tmp[(width - 1, y)] = Tile::Wall;
        }
        for x in 0..width {
            tmp[(x, 0)] = Tile::Wall;
            tmp[(x, height - 1)] = Tile::Wall;
        }
        tmp[features.entrance] = Tile::Open;
        tmp[features.exit] = Tile::Open;
        for (dir, orig) in features.blizzards.iter() {
            let new_pos = match dir {
                Dir::Up => (
                    orig.0,
                    1 + (orig.1 as i32 - 1 - time as i32).rem_euclid(height as i32 - 2) as u32,
                ),
                Dir::Down => (
                    orig.0,
                    1 + (orig.1 as i32 - 1 + time as i32).rem_euclid(height as i32 - 2) as u32,
                ),
                Dir::Left => (
                    1 + (orig.0 as i32 - 1 - time as i32).rem_euclid(width as i32 - 2) as u32,
                    orig.1,
                ),
                Dir::Right => (
                    1 + (orig.0 as i32 - 1 + time as i32).rem_euclid(width as i32 - 2) as u32,
                    orig.1,
                ),
            };
            tmp[new_pos] = Tile::Blizzard(*dir);
        }
        maps_over_time.push(tmp);
    }
}

fn manhattan(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

fn parse_map(input: &[u8]) -> IResult<&[u8], Map<Tile>> {
    let (rest, mut data) = terminated(many0(parse_tile), newline)(input)?;
    let width = data.len() as u32;
    let mut height = 1;

    let (rest, ()) = fold_many1(
        terminated(
            fold_many_m_n(
                width as usize,
                width as usize,
                parse_tile,
                || (),
                |(), tile| {
                    data.push(tile);
                },
            ),
            newline,
        ),
        || (),
        |(), ()| {
            height += 1;
        },
    )(rest)?;

    Ok((
        rest,
        Map {
            data,
            width,
            height,
        },
    ))
}

fn parse_tile(input: &[u8]) -> IResult<&[u8], Tile> {
    map_opt(take(1usize), |tile: &[u8]| match tile[0] {
        b'#' => Some(Tile::Wall),
        b'.' => Some(Tile::Open),
        b'^' => Some(Tile::Blizzard(Dir::Up)),
        b'<' => Some(Tile::Blizzard(Dir::Left)),
        b'>' => Some(Tile::Blizzard(Dir::Right)),
        b'v' => Some(Tile::Blizzard(Dir::Down)),
        _ => None,
    })(input)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct State {
    time: u32,
    pos: (u32, u32),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Tile {
    Open,
    Wall,
    Blizzard(Dir),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

struct Map<T> {
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T: Clone> Map<T> {
    fn new(width: u32, height: u32, init: T) -> Self {
        Map {
            data: vec![init; width as usize * height as usize],
            width,
            height,
        }
    }

    fn offset(&self, x: u32, y: u32) -> usize {
        (x as usize) + (self.width as usize) * (y as usize)
    }
}

impl<T: Clone> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[self.offset(index.0, index.1)]
    }
}

impl<T: Clone> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        let offset = self.offset(index.0, index.1);
        &mut self.data[offset]
    }
}

impl Display for Map<Tile> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.chunks(self.width as usize) {
            for col in row {
                f.write_char(match col {
                    Tile::Open => '.',
                    Tile::Wall => '#',
                    Tile::Blizzard(dir) => match dir {
                        Dir::Up => '^',
                        Dir::Down => 'v',
                        Dir::Left => '<',
                        Dir::Right => '>',
                    },
                })?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

crate::test_day!(RUN, "day24", "251", "758");
