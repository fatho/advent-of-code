#![allow(unused)]

use std::{
    fmt::{Display, Write},
    ops::{Index, IndexMut},
};

use anyhow::bail;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::{many0, many1, many_m_n},
    sequence::{pair, separated_pair, terminated},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let paths = parsers::parse(many0(terminated(parse_path, newline)), input)?;
    let (width, height) = paths
        .iter()
        .flat_map(|p| p.iter())
        .fold((501, 0), |(w, h), pos| (w.max(pos.x + 1), h.max(pos.y + 1)));

    let mut map = Map::new(width, height, Cell::Air);
    for path in paths.iter() {
        map.draw_path(path, Cell::Rock)
    }

    let mut count = 0;
    while let Fall::Settled = propagate_sand(&mut map, Pos { x: 500, y: 0 }) {
        count += 1
    }

    Ok(count.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut paths = parsers::parse(many0(terminated(parse_path, newline)), input)?;
    let (maxx, maxy) = paths
        .iter()
        .flat_map(|p| p.iter())
        .fold((501, 0), |(mx, my), pos| (mx.max(pos.x), my.max(pos.y)));

    let floory = maxy + 2;
    let height = floory + 1;

    // extend width to suitable size
    let max_sand_x = 500 + height;
    let min_sand_x = 500 - height as i32;

    // correct for negativev coordinates
    let offsetx = if height > 500 { height - 500 } else { 0 };

    let width = maxx.max(max_sand_x) + 1 + offsetx;
    let origin = Pos {
        x: 500 + offsetx,
        y: 0,
    };
    // correct paths
    for path in paths.iter_mut() {
        for pos in path.iter_mut() {
            pos.x += offsetx;
        }
    }

    let mut map = Map::new(width, height, Cell::Air);
    for path in paths.iter() {
        map.draw_path(path, Cell::Rock)
    }
    // draw floor
    for x in 0..map.width {
        map[Pos { x, y: height - 1 }] = Cell::Rock;
    }

    let mut count = 0;
    while !matches!(
        propagate_sand(&mut map, Pos { x: 500, y: 0 }),
        Fall::Blocked
    ) {
        count += 1
    }

    Ok(count.to_string())
}

fn parse_pos(input: &[u8]) -> IResult<&[u8], Pos> {
    map(
        separated_pair(
            nom::character::complete::u32,
            tag(","),
            nom::character::complete::u32,
        ),
        |(x, y)| Pos { x, y },
    )(input)
}

fn parse_path(input: &[u8]) -> IResult<&[u8], Path> {
    map(
        pair(many1(terminated(parse_pos, tag(" -> "))), parse_pos),
        |(mut init, last)| {
            init.push(last);
            init
        },
    )(input)
}

#[derive(Debug, Clone, Copy)]
enum Fall {
    Settled,
    Abyss,
    Blocked,
}

fn propagate_sand(map: &mut Map<Cell>, origin: Pos) -> Fall {
    if !matches!(map[origin], Cell::Air) {
        return Fall::Blocked;
    }

    let mut current = origin;
    while map.contains(current) {
        let down = Pos {
            y: current.y + 1,
            ..current
        };
        let ldown = Pos {
            y: current.y + 1,
            x: current.x - 1,
        };
        let rdown = Pos {
            y: current.y + 1,
            x: current.x + 1,
        };

        if !map.contains(down) || matches!(map[down], Cell::Air) {
            current = down;
        } else if !map.contains(ldown) || matches!(map[ldown], Cell::Air) {
            current = ldown;
        } else if !map.contains(rdown) || matches!(map[rdown], Cell::Air) {
            current = rdown;
        } else {
            map[current] = Cell::Sand;
            return Fall::Settled;
        }
    }
    Fall::Abyss
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: u32,
    y: u32,
}

type Path = Vec<Pos>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Rock,
    Air,
    Sand,
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

    fn draw_path(&mut self, path: &Path, color: T) {
        for (from, to) in path.iter().zip(path.iter().skip(1)) {
            self.draw_line(*from, *to, color.clone());
        }
    }

    fn draw_line(&mut self, from: Pos, to: Pos, color: T) {
        if from.x == to.x {
            let x = from.x;
            for y in from.y.min(to.y)..=from.y.max(to.y) {
                self[Pos { x, y }] = color.clone();
            }
        } else {
            let y = from.y;
            for x in from.x.min(to.x)..=from.x.max(to.x) {
                self[Pos { x, y }] = color.clone();
            }
        }
    }

    fn contains(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}

impl<T: Clone> Index<Pos> for Map<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.data[self.offset(index.x, index.y)]
    }
}

impl<T: Clone> IndexMut<Pos> for Map<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        let offset = self.offset(index.x, index.y);
        &mut self.data[offset]
    }
}

impl Display for Map<Cell> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.chunks(self.width as usize) {
            for col in row {
                f.write_char(match col {
                    Cell::Rock => '#',
                    Cell::Air => '.',
                    Cell::Sand => 'o',
                })?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

crate::test_day!(RUN, "day14", "1298", "25585");
