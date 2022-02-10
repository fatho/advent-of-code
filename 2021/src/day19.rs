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
use rustc_hash::FxHashSet;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let scanners = parsers::parse(p_input, input)?;

    let (absolute_points, _) = match_point_clouds(scanners)?;
    Ok(absolute_points.len().to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let scanners = parsers::parse(p_input, input)?;

    let (_, scanner_positions) = match_point_clouds(scanners)?;

    let mut largest = 0;
    for i in 0..scanner_positions.len() - 1 {
        for j in i..scanner_positions.len() {
            let delta = scanner_positions[i] - scanner_positions[j];
            let manhattan = delta.vs.iter().map(|v| v.abs()).sum();
            if manhattan > largest {
                largest = manhattan;
            }
        }
    }
    Ok(largest.to_string())
}

fn match_point_clouds(mut scanners: Vec<Scanner>) -> Result<(Vec<Vec3>, Vec<Vec3>), anyhow::Error> {
    let ref_scanner = scanners.pop().context("must have at leat one scanner")?;
    let mut scanner_positions = vec![Vec3::new(0, 0, 0)];

    let mut absolute_points = PointSet::default();
    ref_scanner.points.iter().copied().for_each(|p| {
        absolute_points.insert(p);
    });
    let mut reference_points = ref_scanner.points;
    let mut worklist = scanners
        .into_iter()
        .map(|scanner| (scanner, 0))
        .collect::<Vec<_>>();
    while !worklist.is_empty() {
        for i in (0..worklist.len()).rev() {
            let (ref scanner, ref mut ref_point_base) = worklist[i];
            if let Some((o, off)) = match_scanner(
                &absolute_points,
                &reference_points[*ref_point_base..],
                scanner,
                12,
            ) {
                for p in scanner.points.iter() {
                    let point = o.local_to_global(*p) + off;
                    if absolute_points.insert(point) {
                        reference_points.push(point);
                    }
                }
                worklist.swap_remove(i);
                scanner_positions.push(off);
            } else {
                *ref_point_base = reference_points.len();
            }
        }
    }
    Ok((reference_points, scanner_positions))
}

fn match_scanner(
    final_points: &PointSet,
    reference_points: &[Vec3],
    scanner: &Scanner,
    threshold: u32,
) -> Option<(Orientation, Vec3)> {
    // check each orientation
    for o in ORIENTATIONS {
        // take each of the original points as reference point
        for aref in reference_points {
            // assuming it corresponds to each point from the scanner
            for sref in scanner.points.iter() {
                // Now check if the remaining points are consistent
                let srot = o.local_to_global(*sref);
                // Assumed offset
                let offset = *aref - srot;

                let mut matches = 0;
                for spoint in scanner.points.iter() {
                    let spoint_as_abs = o.local_to_global(*spoint) + offset;
                    matches += final_points.contains(&spoint_as_abs) as u32;
                }
                if matches >= threshold {
                    return Some((o, offset));
                }
            }
        }
    }
    None
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
        |(x, _, y, _, z)| Vec3::new(x, y, z),
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vec3 {
    vs: [i32; 3],
}

impl Vec3 {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { vs: [x, y, z] }
    }

    pub const fn x(self) -> i32 {
        self.vs[0]
    }

    pub const fn y(self) -> i32 {
        self.vs[1]
    }

    pub const fn z(self) -> i32 {
        self.vs[2]
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

impl Mul<i32> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Vec3::new(self.x() * rhs, self.y() * rhs, self.z() * rhs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scanner {
    id: u32,
    points: Vec<Vec3>,
}

#[derive(Default)]
struct PointSet {
    points: FxHashSet<Vec3>,
    xyz: Vec<u64>,
}

impl PointSet {
    #[inline(always)]
    pub fn insert(&mut self, point: Vec3) -> bool {
        let xyz = zigzag(point.vs.into_iter().sum());
        let word = (xyz >> 6) as usize;
        let bit = xyz & 0b11_1111;

        if word >= self.xyz.len() {
            self.xyz.resize(word * 2, 0)
        }
        self.xyz[word] |= 1 << bit;

        self.points.insert(point)
    }

    #[inline(always)]
    pub fn contains(&self, point: &Vec3) -> bool {
        let xyz = zigzag(point.vs.into_iter().sum());
        let word = (xyz >> 6) as usize;
        let bit = xyz & 0b11_1111;

        self.xyz.get(word).map_or(false, move |w| {
            w & (1 << bit) != 0 && self.points.contains(point)
        })
    }
}

#[inline(always)]
fn zigzag(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

/// All 24 possible orientations of a scanner.
const ORIENTATIONS: [Orientation; 24] = [
    Orientation {
        perm: [0, 1, 2],
        sign: [1, 1, 1],
    },
    Orientation {
        perm: [0, 2, 1],
        sign: [1, 1, -1],
    },
    Orientation {
        perm: [0, 1, 2],
        sign: [1, -1, -1],
    },
    Orientation {
        perm: [0, 2, 1],
        sign: [1, -1, 1],
    },
    Orientation {
        perm: [0, 1, 2],
        sign: [-1, 1, -1],
    },
    Orientation {
        perm: [0, 2, 1],
        sign: [-1, 1, 1],
    },
    Orientation {
        perm: [0, 1, 2],
        sign: [-1, -1, 1],
    },
    Orientation {
        perm: [0, 2, 1],
        sign: [-1, -1, -1],
    },
    Orientation {
        perm: [1, 0, 2],
        sign: [1, 1, -1],
    },
    Orientation {
        perm: [1, 2, 0],
        sign: [1, 1, 1],
    },
    Orientation {
        perm: [1, 0, 2],
        sign: [1, -1, 1],
    },
    Orientation {
        perm: [1, 2, 0],
        sign: [1, -1, -1],
    },
    Orientation {
        perm: [1, 0, 2],
        sign: [-1, 1, 1],
    },
    Orientation {
        perm: [1, 2, 0],
        sign: [-1, 1, -1],
    },
    Orientation {
        perm: [1, 0, 2],
        sign: [-1, -1, -1],
    },
    Orientation {
        perm: [1, 2, 0],
        sign: [-1, -1, 1],
    },
    Orientation {
        perm: [2, 0, 1],
        sign: [1, 1, 1],
    },
    Orientation {
        perm: [2, 1, 0],
        sign: [1, 1, -1],
    },
    Orientation {
        perm: [2, 0, 1],
        sign: [1, -1, -1],
    },
    Orientation {
        perm: [2, 1, 0],
        sign: [1, -1, 1],
    },
    Orientation {
        perm: [2, 0, 1],
        sign: [-1, 1, -1],
    },
    Orientation {
        perm: [2, 1, 0],
        sign: [-1, 1, 1],
    },
    Orientation {
        perm: [2, 0, 1],
        sign: [-1, -1, 1],
    },
    Orientation {
        perm: [2, 1, 0],
        sign: [-1, -1, -1],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Orientation {
    /// Permutation of the vector components
    perm: [u8; 3],
    /// Rotation of the vector components
    sign: [i8; 3],
}

impl Orientation {
    #[inline(always)]
    pub fn local_to_global(&self, v: Vec3) -> Vec3 {
        Vec3 {
            vs: [
                self.sign[0] as i32 * v.vs[self.perm[0] as usize],
                self.sign[1] as i32 * v.vs[self.perm[1] as usize],
                self.sign[2] as i32 * v.vs[self.perm[2] as usize],
            ],
        }
    }
}

#[test]
fn test_p_relpnt() {
    assert_eq!(
        p_relpnt(b"1,-2,3").unwrap(),
        (b"".as_ref(), Vec3::new(1, -2, 3))
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
                points: vec![Vec3::new(602, 365, -604), Vec3::new(309, -819, -775)]
            }
        )
    );
}

#[test]
fn test_p_input() {
    let points = vec![Vec3::new(602, 365, -604), Vec3::new(309, -819, -775)];
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

crate::test_day!(crate::day19::RUN, "day19", "315", "13192");
