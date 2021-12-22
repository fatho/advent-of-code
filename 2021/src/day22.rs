#![allow(unused_imports)]

use std::ops::RangeInclusive;

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{take_while, tag};
use nom::combinator::{flat_map, map, value};
use nom::multi::{fold_many0, many0};
use nom::character::complete as numbers;
use nom::sequence::{terminated, separated_pair, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let input = parsers::parse(p_init_seq, input)?;

    let mut cube = [[[false; 101]; 101]; 101];

    const TARGET: RangeInclusive<i32> = -50..=50;

    for cmd in input.iter() {
        for x in clamp_range(TARGET, &cmd.x) {
            for y in clamp_range(TARGET, &cmd.y) {
                for z in clamp_range(TARGET, &cmd.z) {
                    cube[(x + 50) as usize][(y + 50) as usize][(z + 50) as usize] = cmd.on;
                }
            }
        }
    }

    let mut num_on = 0;
    for ex_x in cube {
        for ex_xy in ex_x {
            for ex_xyz in ex_xy {
                if ex_xyz {
                    num_on += 1;
                }
            }
        }
    }

    Ok(num_on.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let input = parsers::parse(p_init_seq, input)?;

    let extent = input.iter().fold(((0,0), (0,0), (0,0)), |((xmin, xmax),(ymin, ymax), (zmin, zmax)), cmd| {
        (
            (xmin.min(*cmd.x.start()), xmax.min(*cmd.x.end())),
            (ymin.min(*cmd.y.start()), ymax.min(*cmd.y.end())),
            (zmin.min(*cmd.z.start()), zmax.min(*cmd.z.end())),
        )
    });

    println!("{:?}", extent);

    todo!()
}

fn clamp_range(target: RangeInclusive<i32>, actual: &RangeInclusive<i32>) -> RangeInclusive<i32> {
    *target.start().max(actual.start())..=*target.end().min(actual.end())
}


fn p_init_seq(input: &[u8]) -> IResult<&[u8], Vec<Cmd>> {
    many0(terminated(p_cmd, parsers::newline))(input)
}

fn p_cmd(input: &[u8]) -> IResult<&[u8], Cmd> {
    // on x=-20..26,y=-36..17,z=-47..7
    map(
        tuple((
            alt((value(false, tag("off")), value(true, tag("on")))),
            tag(" x="),
            p_range,
            tag(",y="),
            p_range,
            tag(",z="),
            p_range
        )),
        |(on, _, x, _, y, _, z)| Cmd {
            on, x, y, z
        }
    )(input)
}

fn p_range(input: &[u8]) -> IResult<&[u8], RangeInclusive<i32>> {
    map(
        separated_pair(numbers::i32, tag(".."), numbers::i32),
        |(from, to)| from..=to
    )(input)
}




#[derive(Debug, Clone, PartialEq, Eq)]
struct Cube {
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
}


impl Cube {
    pub fn is_empty(&self) -> bool {
        [&self.x, &self.y, &self.z].into_iter().any(|coords| coords.end() < coords.start())
    }

    pub fn intersect(&self, other: &Cube) -> Cube {
        Cube {
            x: *self.x.start().max(other.x.start())..=*self.x.end().min(other.x.end()),
            y: *self.y.start().max(other.y.start())..=*self.y.end().min(other.y.end()),
            z: *self.z.start().max(other.z.start())..=*self.z.end().min(other.z.end()),
        }
    }

    pub fn volume(&self) -> usize {
        (self.x.end() - self.x.start() + 1) as usize
        * (self.y.end() - self.y.start() + 1) as usize
        * (self.z.end() - self.z.start() + 1) as usize
    }
}

struct Cmd {
    on: bool,
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
}

crate::test_day!(crate::day22::RUN, "day22", "570915", "not solved");
