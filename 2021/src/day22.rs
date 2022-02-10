#![allow(unused_imports)]

use std::ops::RangeInclusive;

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map, value};
use nom::multi::{fold_many0, many0};
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let input = parsers::parse(p_init_seq, input)?;

    const TARGET: RangeInclusive<i32> = -50..=50;
    let target_cube = Cuboid {
        x: TARGET.into(),
        y: TARGET.into(),
        z: TARGET.into(),
    };

    let mut on_ranges: Vec<Cuboid> = Vec::new();
    let mut new = Vec::new();
    let mut tmp = Vec::new();
    // Invariant: on_ranges do not overlap
    for cmd in input {
        let clamped_cuboid = target_cube.intersect(&Cuboid {
            x: cmd.x.clone().into(),
            y: cmd.y.clone().into(),
            z: cmd.z.clone().into(),
        });
        if clamped_cuboid.is_empty() {
            continue;
        }
        if cmd.on {
            new.push(clamped_cuboid);
            for on in on_ranges.iter() {
                for n in new.iter() {
                    n.subtract(on, &mut tmp);
                }
                std::mem::swap(&mut new, &mut tmp);
                tmp.clear();
            }
            on_ranges.append(&mut new);
        } else {
            for on in on_ranges.drain(..) {
                on.subtract(
                    &Cuboid {
                        x: cmd.x.clone().into(),
                        y: cmd.y.clone().into(),
                        z: cmd.z.clone().into(),
                    },
                    &mut tmp,
                );
            }
            std::mem::swap(&mut on_ranges, &mut tmp);
        }
    }

    // Count on
    let num_on = on_ranges.iter().map(|c| c.volume()).sum::<usize>();

    Ok(num_on.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let input = parsers::parse(p_init_seq, input)?;

    let mut on_ranges: Vec<Cuboid> = Vec::new();
    let mut new = Vec::new();
    let mut tmp = Vec::new();
    // Invariant: on_ranges do not overlap
    for cmd in input {
        if cmd.on {
            new.push(Cuboid {
                x: cmd.x.clone().into(),
                y: cmd.y.clone().into(),
                z: cmd.z.clone().into(),
            });
            for on in on_ranges.iter() {
                for n in new.iter() {
                    n.subtract(on, &mut tmp);
                }
                std::mem::swap(&mut new, &mut tmp);
                tmp.clear();
            }
            on_ranges.append(&mut new);
        } else {
            for on in on_ranges.drain(..) {
                on.subtract(
                    &Cuboid {
                        x: cmd.x.clone().into(),
                        y: cmd.y.clone().into(),
                        z: cmd.z.clone().into(),
                    },
                    &mut tmp,
                );
            }
            std::mem::swap(&mut on_ranges, &mut tmp);
        }
    }

    // Count on
    let num_on = on_ranges.iter().map(|c| c.volume()).sum::<usize>();

    Ok(num_on.to_string())
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
            p_range,
        )),
        |(on, _, x, _, y, _, z)| Cmd { on, x, y, z },
    )(input)
}

fn p_range(input: &[u8]) -> IResult<&[u8], RangeInclusive<i32>> {
    map(
        separated_pair(numbers::i32, tag(".."), numbers::i32),
        |(from, to)| from..=to,
    )(input)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct CoordRange {
    start: i32,
    end: i32,
}

impl From<RangeInclusive<i32>> for CoordRange {
    fn from(r: RangeInclusive<i32>) -> Self {
        Self {
            start: *r.start(),
            end: *r.end(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Cuboid {
    x: CoordRange,
    y: CoordRange,
    z: CoordRange,
}

impl Cuboid {
    pub fn is_empty(&self) -> bool {
        self.x.end < self.x.start || self.y.end < self.y.start || self.z.end < self.z.start
    }

    pub fn intersect(&self, other: &Cuboid) -> Cuboid {
        Cuboid {
            x: (self.x.start.max(other.x.start)..=self.x.end.min(other.x.end)).into(),
            y: (self.y.start.max(other.y.start)..=self.y.end.min(other.y.end)).into(),
            z: (self.z.start.max(other.z.start)..=self.z.end.min(other.z.end)).into(),
        }
    }

    pub fn volume(&self) -> usize {
        (self.x.end - self.x.start + 1) as usize
            * (self.y.end - self.y.start + 1) as usize
            * (self.z.end - self.z.start + 1) as usize
    }

    pub fn xmin(&self) -> i32 {
        self.x.start
    }
    pub fn xmax(&self) -> i32 {
        self.x.end
    }
    pub fn ymin(&self) -> i32 {
        self.y.start
    }
    pub fn ymax(&self) -> i32 {
        self.y.end
    }
    pub fn zmin(&self) -> i32 {
        self.z.start
    }
    pub fn zmax(&self) -> i32 {
        self.z.end
    }

    pub fn subtract(&self, other: &Cuboid, output: &mut Vec<Cuboid>) {
        let chunk = self.intersect(other);

        if chunk.is_empty() {
            // No overlap, return whole
            output.push(*self);
            return;
        }

        // Z-top
        if self.zmin() < chunk.zmin() {
            output.push(Cuboid {
                x: self.x,
                y: self.y,
                z: (self.zmin()..=chunk.zmin() - 1).into(),
            });
        }
        // Z-bottom
        if chunk.zmax() < self.zmax() {
            output.push(Cuboid {
                x: self.x,
                y: self.y,
                z: (chunk.zmax() + 1..=self.zmax()).into(),
            });
        }
        // X-left
        if self.xmin() < chunk.xmin() {
            output.push(Cuboid {
                x: (self.xmin()..=chunk.xmin() - 1).into(),
                y: self.y,
                z: chunk.z,
            });
        }
        // X-right
        if chunk.xmax() < self.xmax() {
            output.push(Cuboid {
                x: (chunk.xmax() + 1..=self.xmax()).into(),
                y: self.y,
                z: chunk.z,
            });
        }
        // Y-left
        if self.ymin() < chunk.ymin() {
            output.push(Cuboid {
                x: chunk.x,
                y: (self.ymin()..=chunk.ymin() - 1).into(),
                z: chunk.z,
            });
        }
        // Y-right
        if chunk.ymax() < self.ymax() {
            output.push(Cuboid {
                x: chunk.x,
                y: (chunk.ymax() + 1..=self.ymax()).into(),
                z: chunk.z,
            });
        }
    }
}

struct Cmd {
    on: bool,
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
}

crate::test_day!(crate::day22::RUN, "day22", "570915", "1268313839428137");
