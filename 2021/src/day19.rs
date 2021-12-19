#![allow(unused_imports)]

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

fn p_relpnt(input: &[u8]) -> IResult<&[u8], RelPnt> {
    map(
        tuple((numbers::i32, tag(","), numbers::i32, tag(","), numbers::i32)),
        |(x, _, y, _, z)| RelPnt { x, y, z },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RelPnt {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scanner {
    id: u32,
    points: Vec<RelPnt>,
}

#[test]
fn test_p_relpnt() {
    assert_eq!(
        p_relpnt(b"1,-2,3").unwrap(),
        (b"".as_ref(), RelPnt { x: 1, y: -2, z: 3 })
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
                    RelPnt {
                        x: 602,
                        y: 365,
                        z: -604
                    },
                    RelPnt {
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
        RelPnt {
            x: 602,
            y: 365,
            z: -604,
        },
        RelPnt {
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
