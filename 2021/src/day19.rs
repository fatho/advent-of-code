#![allow(unused_imports)]

use std::collections::HashSet;
use std::ops::{Add, Mul, Sub};

use crate::{parsers, Day};
use anyhow::Context;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0, separated_list0};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

// TODO: definitely needs a performance upgrade

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut scanners = parsers::parse(p_input, input)?;
    //println!("{:?}", scanners);

    let ref_scanner = scanners.pop().context("must have at leat one scanner")?;
    let mut absolute_points = ref_scanner.points.iter().copied().collect::<HashSet<_>>();

    while !scanners.is_empty() {
        for i in (0..scanners.len()).rev() {
            if let Some((o, off)) = match_scanner(&absolute_points, &scanners[i], 12) {
                for p in scanners[i].points.iter() {
                    absolute_points.insert(o.local_to_global(*p) + off);
                }
                scanners.swap_remove(i);
            }
        }
        println!("{} remaining", scanners.len());
    }
    Ok(absolute_points.len().to_string())
}

fn match_scanner(
    abs: &HashSet<Vec3>,
    scanner: &Scanner,
    threshold: u32,
) -> Option<(Orientation, Vec3)> {
    // check each orientation
    for o in ORIENTATIONS {
        // take each of the original points as reference point
        for aref in abs {
            // assuming it corresponds to each point from the scanner
            for sref in scanner.points.iter() {
                // Now check if the remaining points are consistent
                let srot = o.local_to_global(*sref);
                // Assumed offset
                let offset = *aref - srot;

                let mut matches = 0;
                for spoint in scanner.points.iter() {
                    let spoint_as_abs = o.local_to_global(*spoint) + offset;
                    // TODO: faster lookup
                    if abs.contains(&spoint_as_abs) {
                        matches += 1;
                    }
                }
                if matches >= threshold {
                    return Some((o, offset));
                }
            }
        }
    }
    None
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut scanners = parsers::parse(p_input, input)?;
    //println!("{:?}", scanners);

    let mut scanner_positions = Vec::new();
    let ref_scanner = scanners.pop().context("must have at leat one scanner")?;
    scanner_positions.push(Vec3::new(0, 0, 0));
    let mut absolute_points = ref_scanner.points.iter().copied().collect::<HashSet<_>>();

    while !scanners.is_empty() {
        for i in (0..scanners.len()).rev() {
            if let Some((o, off)) = match_scanner(&absolute_points, &scanners[i], 12) {
                for p in scanners[i].points.iter() {
                    absolute_points.insert(o.local_to_global(*p) + off);
                }
                scanners.swap_remove(i);
                scanner_positions.push(off);
            }
        }
        println!("{} remaining", scanners.len());
    }

    let mut largest = 0;
    for i in 0..scanner_positions.len() - 1 {
        for j in i..scanner_positions.len() {
            let delta = scanner_positions[i] - scanner_positions[j];
            let manhattan = delta.x.abs() + delta.y.abs() + delta.z.abs();
            if manhattan > largest {
                largest = manhattan;
            }
        }
    }
    Ok(largest.to_string())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
