#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let map = parsers::parse(p_map, input)?;

    let mut total_risk = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let height = map.get(x, y);
            if map.neighbours(x, y).all(|h| h > height) {
                total_risk += 1 + height as u32;
            }
        }
    }

    Ok(total_risk as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    parsers::parse(|_| Ok((b"", 0)), input)
}

fn p_map(input: &[u8]) -> IResult<&[u8], Map> {
    flat_map(
        terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
        |first_line| {
            let width = first_line.len();
            fold_many0(
                map_opt(
                    terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
                    move |line| {
                        if line.len() == width {
                            Some(line)
                        } else {
                            None
                        }
                    },
                ),
                move || Map {
                    data: first_line.iter().map(|c| c - b'0').collect(),
                    width: width as u32,
                    height: 1,
                },
                |mut map, line| {
                    map.height += 1;
                    map.data.extend(line.iter().map(|c| c - b'0'));
                    map
                },
            )
        },
    )(input)
}

#[derive(Debug)]
struct Map {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Map {
    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.data[(y * self.width + x) as usize]
    }

    pub fn neighbours(&self, x: u32, y: u32) -> impl Iterator<Item = u8> + '_ {
        let positions = [(x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)];
        positions
            .into_iter()
            .filter(|(nx, ny)| (0..self.width).contains(nx) && (0..self.height).contains(ny))
            .map(|(nx, ny)| self.get(nx, ny))
    }
}

crate::test_day!(crate::day9::RUN, "day9", 0, 0);
