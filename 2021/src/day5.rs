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

    let mut map = HashMap::<Point, u32>::new();

    for line in lines {
        if line.is_axis_aligned() {
            for p in line.points() {
                let entry = map.entry(p).or_default();
                *entry += 1;
            }
        }
    }

    Ok(map.values().filter(|lines| **lines >= 2).count() as i64)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let lines = parsers::parse(many0(terminated(Line::parse, parsers::newline)), input)?;

    let mut map = HashMap::<Point, u32>::new();

    for line in lines {
        for p in line.points() {
            let entry = map.entry(p).or_default();
            *entry += 1;
        }
    }

    Ok(map.values().filter(|lines| **lines >= 2).count() as i64)
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

    pub fn is_axis_aligned(&self) -> bool {
        self.p1.x == self.p2.x || self.p1.y == self.p2.y
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        let dx = self.p2.x - self.p1.x;
        let dy = self.p2.y - self.p1.y;
        assert!(
            dx == 0 || dy == 0 || dx.abs() == dy.abs(),
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
        .take(steps as usize + 1)
    }
}

crate::test_day!(crate::day5::RUN, "day5", 7644, 18627);
