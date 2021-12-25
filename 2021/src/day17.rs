#![allow(unused_imports)]

use std::ops::RangeInclusive;

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map, opt};
use nom::multi::fold_many0;
use nom::sequence::{pair, preceded, separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let target = parsers::parse(terminated(p_target, opt(parsers::newline)), input)?;

    // TODO: awful performance, pls don't do this at home
    let mut highest = 0;
    for vx in 0..=*target.x.end() {
        for vy in 0..10000 {
            if let Some(top) = simulate((0, 0), (vx, vy), &target) {
                if top > highest {
                    highest = top;
                }
            }
        }
    }

    Ok(highest.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let target = parsers::parse(terminated(p_target, opt(parsers::newline)), input)?;

    // TODO: awful performance, pls don't do this at home
    let mut count = 0;
    for vx in 0..=*target.x.end() {
        for vy in -200..10000 {
            if let Some(_top) = simulate((0, 0), (vx, vy), &target) {
                count += 1;
            }
        }
    }

    Ok(count.to_string())
}

fn simulate(start: (i32, i32), vel: (i32, i32), target: &Target) -> Option<i32> {
    let (mut x, mut y) = start;
    let (mut vx, mut vy) = vel;
    let mut highest = y;
    loop {
        x += vx;
        y += vy;
        vx -= vx.signum();
        vy -= 1;

        if y > highest {
            highest = y;
        }

        if target.x.contains(&x) && target.y.contains(&y) {
            return Some(highest);
        }
        if y < *target.y.start() {
            return None;
        }
    }
}

fn p_target(input: &[u8]) -> IResult<&[u8], Target> {
    map(
        preceded(
            tag("target area: "),
            separated_pair(
                preceded(
                    tag("x="),
                    separated_pair(numbers::i32, tag(".."), numbers::i32),
                ),
                tag(", "),
                preceded(
                    tag("y="),
                    separated_pair(numbers::i32, tag(".."), numbers::i32),
                ),
            ),
        ),
        |((x1, x2), (y1, y2))| Target {
            x: x1..=x2,
            y: y1..=y2,
        },
    )(input)
}

#[derive(Debug)]
struct Target {
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
}

crate::test_day!(crate::day17::RUN, "day17", "7750", "4120");
