use anyhow::bail;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0};
use nom::sequence::{separated_pair, terminated};
use nom::IResult;

use crate::{parsers, Day};
use std::cmp::Ordering;
use std::collections::binary_heap::Iter;
use std::collections::HashMap;

pub static RUN: Day = Day { part1, part2 };

pub fn part1<'a>(input: &'a [u8]) -> anyhow::Result<i64> {
    let lines = parsers::parse(many0(terminated(Line::parse, parsers::newline)), input)?;

    let mut map = HashMap::<(i32, i32), u32>::new();

    for line in lines {
        if line.p1.x == line.p2.x {
            let (from, to) = if line.p1.y < line.p2.y {
                (line.p1.y, line.p2.y)
            } else {
                (line.p2.y, line.p1.y)
            };
            for y in from..=to {
                let entry = map.entry((line.p1.x, y)).or_default();
                *entry += 1;
            }
        } else if line.p1.y == line.p2.y {
            let (from, to) = if line.p1.x < line.p2.x {
                (line.p1.x, line.p2.x)
            } else {
                (line.p2.x, line.p1.x)
            };
            for x in from..=to {
                let entry = map.entry((x, line.p1.y)).or_default();
                *entry += 1;
            }
        } else {
            // skip non-axis aligned lines
        }
    }

    Ok(map.values().filter(|lines| **lines >= 2).count() as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let lines = parsers::parse(many0(terminated(Line::parse, parsers::newline)), input)?;
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(parsers::i32, tag(","), parsers::i32),
            |(x, y)| Self { x, y },
        )(input)
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

struct Line {
    p1: Point,
    p2: Point,
}

impl Line {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        map(
            separated_pair(Point::parse, tag(" -> "), Point::parse),
            |(p1, p2)| Self { p1, p2 },
        )(input)
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        let dx = self.p2.x - self.p1.x;
        let dy = self.p2.y - self.p1.y;
        assert!(
            dx == 0 || dy == 0 || dx == dy,
            "lines can only be horizontal, vertical or diagonal"
        );
        let steps = dx.abs().max(dy.abs());
        let stepx = dx.signum();
        let stepy = dy.signum();

        let mut state = self.p1;
        std::iter::from_fn(move || {
            let cur = state;
            state = cur.offset(stepx, stepy);
            Some(cur)
        })
        .take(steps as usize)
    }
}

crate::test_day!(crate::day5::RUN, "day5", 0, 0);
