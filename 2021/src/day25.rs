#![allow(unused_imports)]

use std::fmt::Display;
use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut map = parsers::parse(p_map, input)?;
    let mut next = Map::new(map.width, map.height, Field::Empty);

    let mut steps = 0;
    loop {
        let mut any_moved = false;

        for y in 0..map.height {
            for x in 0..map.width {
                match map[(x, y)] {
                    Field::EastCucumber => {
                        let nx = if x == map.width - 1 { 0 } else { x + 1 };
                        if matches!(map[(nx, y)], Field::Empty) {
                            any_moved = true;
                            next[(nx, y)] = Field::EastCucumber;
                        } else {
                            next[(x, y)] = Field::EastCucumber;
                        }
                    }
                    Field::SouthCucumber => {
                        next[(x, y)] = Field::SouthCucumber;
                    }
                    Field::Empty => {}
                }
            }
        }
        std::mem::swap(&mut map, &mut next);
        next.clear(Field::Empty);
        for y in 0..map.height {
            for x in 0..map.width {
                match map[(x, y)] {
                    Field::EastCucumber => {
                        next[(x, y)] = Field::EastCucumber;
                    }
                    Field::SouthCucumber => {
                        let ny = if y == map.height - 1 { 0 } else { y + 1 };
                        if matches!(map[(x, ny)], Field::Empty) {
                            any_moved = true;
                            next[(x, ny)] = Field::SouthCucumber;
                        } else {
                            next[(x, y)] = Field::SouthCucumber;
                        }
                    }
                    Field::Empty => {}
                }
            }
        }
        std::mem::swap(&mut map, &mut next);
        next.clear(Field::Empty);

        steps += 1;
        if !any_moved {
            break;
        }
    }

    Ok(steps.to_string())
}

pub fn part2(_input: &[u8]) -> anyhow::Result<String> {
    Ok("not needed".to_owned())
}

fn p_map(input: &[u8]) -> IResult<&[u8], Map<Field>> {
    flat_map(
        terminated(
            take_while(|c| matches!(c, b'.' | b'v' | b'>')),
            parsers::newline,
        ),
        |first_line| {
            let width = first_line.len();
            fold_many0(
                map_opt(
                    terminated(
                        take_while(|c| matches!(c, b'.' | b'v' | b'>')),
                        parsers::newline,
                    ),
                    move |line| {
                        if line.len() == width {
                            Some(line)
                        } else {
                            None
                        }
                    },
                ),
                move || Map {
                    data: first_line
                        .iter()
                        .map(|c| Field::parse(*c).unwrap())
                        .collect(),
                    width: width as u32,
                    height: 1,
                },
                |mut map, line| {
                    map.height += 1;
                    map.data
                        .extend(line.iter().map(|c| Field::parse(*c).unwrap()));
                    map
                },
            )
        },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Empty,
    EastCucumber,
    SouthCucumber,
}

impl Field {
    pub fn parse(ch: u8) -> Option<Field> {
        match ch {
            b'.' => Some(Field::Empty),
            b'>' => Some(Field::EastCucumber),
            b'v' => Some(Field::SouthCucumber),
            _ => None,
        }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Empty => write!(f, "."),
            Field::EastCucumber => write!(f, ">"),
            Field::SouthCucumber => write!(f, "v"),
        }
    }
}

#[derive(Debug)]
struct Map<T> {
    // Presumably no need to bother with Z-order curve here since whole data
    // (100 bytes) fits into two cache lines already (typically 128 bytes).
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Map<T>
where
    T: Copy,
{
    pub fn new(width: u32, height: u32, value: T) -> Self {
        Self {
            data: vec![value; width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn clear(&mut self, value: T) {
        self.data.iter_mut().for_each(|f| *f = value);
    }
}

impl<T> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1 * self.width) as usize]
    }
}

impl<T> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1 * self.width) as usize]
    }
}

impl<T> Display for Map<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{}", self[(x, y)])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

crate::test_day!(crate::day25::RUN, "day25", "530", "not needed");
