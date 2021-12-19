#![allow(unused_imports)]

use std::ops::{Add, Mul};

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0, separated_list0};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let scanners = parsers::parse(p_input, input)?;
    println!("{:?}", scanners);

    todo!()
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let scanners = parsers::parse(p_input, input)?;
    println!("{:?}", scanners);

    todo!()
}

fn p_input(input: &[u8]) -> IResult<&[u8], Vec<Scanner>> {
    separated_list0(tag("\n"), p_scanner)(input)
}

fn p_scanner(input: &[u8]) -> IResult<&[u8], Scanner> {
    map(
        pair(
            // Header
            delimited(tag("--- scanner "), numbers::u32, tag(" ---\n")),
            // Points
            many0(terminated(p_relpnt, parsers::newline)),
        ),
        |(id, points)| Scanner { id, points },
    )(input)
}

fn p_relpnt(input: &[u8]) -> IResult<&[u8], Vec3> {
    map(
        tuple((numbers::i32, tag(","), numbers::i32, tag(","), numbers::i32)),
        |(x, _, y, _, z)| Vec3 { x, y, z },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn cross(self, other: Vec3) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub const fn dot(self, other: Vec3) -> i32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<i32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scanner {
    id: u32,
    points: Vec<Vec3>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Orientation {
    fwd: Vec3,
    up: Vec3,
    right: Vec3, // TODO: or is it left?
}

impl Orientation {
    pub const fn new(fwd: Vec3, up: Vec3) -> Self {
        Self {
            fwd,
            up,
            right: fwd.cross(up),
        }
    }

    pub fn local_to_global(&self, v: Vec3) -> Vec3 {
        self.fwd * v.x + self.up * v.y + self.right * v.z
    }

    pub fn global_to_local(&self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.fwd.dot(v),
            y: self.up.dot(v),
            z: self.right.dot(v),
        }
    }
}

const ORIENTATIONS: [Orientation; 24] = [
    // +X
    Orientation::new(Vec3::new(1, 0, 0), Vec3::new(0, 1, 0)),
    Orientation::new(Vec3::new(1, 0, 0), Vec3::new(0, 0, 1)),
    Orientation::new(Vec3::new(1, 0, 0), Vec3::new(0, -1, 0)),
    Orientation::new(Vec3::new(1, 0, 0), Vec3::new(0, 0, -1)),
    // -X
    Orientation::new(Vec3::new(-1, 0, 0), Vec3::new(0, 1, 0)),
    Orientation::new(Vec3::new(-1, 0, 0), Vec3::new(0, 0, 1)),
    Orientation::new(Vec3::new(-1, 0, 0), Vec3::new(0, -1, 0)),
    Orientation::new(Vec3::new(-1, 0, 0), Vec3::new(0, 0, -1)),
    // +Y
    Orientation::new(Vec3::new(0, 1, 0), Vec3::new(1, 0, 0)),
    Orientation::new(Vec3::new(0, 1, 0), Vec3::new(0, 0, 1)),
    Orientation::new(Vec3::new(0, 1, 0), Vec3::new(-1, 0, 0)),
    Orientation::new(Vec3::new(0, 1, 0), Vec3::new(0, 0, -1)),
    // -Y
    Orientation::new(Vec3::new(0, -1, 0), Vec3::new(1, 0, 0)),
    Orientation::new(Vec3::new(0, -1, 0), Vec3::new(0, 0, 1)),
    Orientation::new(Vec3::new(0, -1, 0), Vec3::new(-1, 0, 0)),
    Orientation::new(Vec3::new(0, -1, 0), Vec3::new(0, 0, -1)),
    // +Z
    Orientation::new(Vec3::new(0, 0, 1), Vec3::new(1, 0, 0)),
    Orientation::new(Vec3::new(0, 0, 1), Vec3::new(0, 1, 0)),
    Orientation::new(Vec3::new(0, 0, 1), Vec3::new(-1, 0, 0)),
    Orientation::new(Vec3::new(0, 0, 1), Vec3::new(0, -1, 0)),
    // -Z
    Orientation::new(Vec3::new(0, 0, -1), Vec3::new(1, 0, 0)),
    Orientation::new(Vec3::new(0, 0, -1), Vec3::new(0, 1, 0)),
    Orientation::new(Vec3::new(0, 0, -1), Vec3::new(-1, 0, 0)),
    Orientation::new(Vec3::new(0, 0, -1), Vec3::new(0, -1, 0)),
];

#[test]
fn test_p_relpnt() {
    assert_eq!(
        p_relpnt(b"1,-2,3").unwrap(),
        (b"".as_ref(), Vec3 { x: 1, y: -2, z: 3 })
    );
}

#[test]
fn test_p_scanner() {
    assert_eq!(
        p_scanner(
            b"--- scanner 0 ---
602,365,-604
309,-819,-775
"
        )
        .unwrap(),
        (
            b"".as_ref(),
            Scanner {
                id: 0,
                points: vec![
                    Vec3 {
                        x: 602,
                        y: 365,
                        z: -604
                    },
                    Vec3 {
                        x: 309,
                        y: -819,
                        z: -775
                    }
                ]
            }
        )
    );
}

#[test]
fn test_p_input() {
    let points = vec![
        Vec3 {
            x: 602,
            y: 365,
            z: -604,
        },
        Vec3 {
            x: 309,
            y: -819,
            z: -775,
        },
    ];
    assert_eq!(
        p_input(
            b"--- scanner 0 ---
602,365,-604
309,-819,-775

--- scanner 1 ---
602,365,-604
309,-819,-775
"
        )
        .unwrap(),
        (
            b"".as_ref(),
            vec![
                Scanner {
                    id: 0,
                    points: points.clone()
                },
                Scanner { id: 1, points }
            ]
        )
    );
}

//crate::test_day!(crate::day19::RUN, "day19", "not solved", "not solved");
