#![allow(unused)]

use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{fail, map},
    multi::{fold_many0, fold_many1, many0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    parsers::{self, asciichar},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (mut stacks, moves) = parsers::parse(parse_input, input)?;

    for mov in moves {
        for _ in 0..mov.count {
            let crat = stacks[mov.from as usize - 1].pop().unwrap();
            stacks[mov.to as usize - 1].push(crat);
        }
    }

    stacks
        .iter()
        .map(|stack| {
            (stack
                .last()
                .map(|ch| *ch as char)
                .ok_or_else(|| anyhow::anyhow!("invalid outcome")))
        })
        .collect::<anyhow::Result<String>>()
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (mut stacks, moves) = parsers::parse(parse_input, input)?;

    for mov in moves {
        let from_count = stacks[mov.from as usize - 1].len();
        for i in 0..mov.count {
            let crat = stacks[mov.from as usize - 1][from_count - mov.count as usize + i as usize];
            stacks[mov.to as usize - 1].push(crat);
        }
        stacks[mov.from as usize - 1].truncate(from_count - mov.count as usize);
    }

    stacks
        .iter()
        .map(|stack| {
            (stack
                .last()
                .map(|ch| *ch as char)
                .ok_or_else(|| anyhow::anyhow!("invalid outcome")))
        })
        .collect::<anyhow::Result<String>>()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Move {
    count: u32,
    from: u32,
    to: u32,
}

fn parse_input(input: &[u8]) -> IResult<&[u8], (Vec<Vec<u8>>, Vec<Move>)> {
    separated_pair(parse_stacks, parsers::newline, many0(parse_move))(input)
}

fn parse_stacks(input: &[u8]) -> IResult<&[u8], Vec<Vec<u8>>> {
    let (rest, rows) = fold_many1(
        terminated(take_while1(|ch| ch != b'\n'), parsers::newline),
        Vec::new,
        |mut rows, row| {
            if row
                .iter()
                .all(|ch| (b'A'..=b'Z').contains(ch) || b" []".contains(ch))
            {
                rows.push(row)
            }
            rows
        },
    )(input)?;

    let mut stacks: Vec<Vec<u8>> = vec![];

    for row in rows {
        let num_stacks = (row.len() + 1) / 4;
        if num_stacks > stacks.len() {
            stacks.extend(std::iter::repeat_with(Vec::new).take(num_stacks - stacks.len()));
        }

        for (index, chunk) in row.chunks(4).enumerate() {
            if (chunk.len() == 4 && chunk[3] != b' ') || chunk.len() < 3 {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Verify,
                )));
            }
            let crat = match chunk {
                [b'[', crat, b']', b' '] => Some(crat),
                [b'[', crat, b']'] => Some(crat),
                b"    " => None,
                b"   " => None,
                _ => {
                    return Err(nom::Err::Failure(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Verify,
                    )))
                }
            };
            if let Some(crat) = crat {
                stacks[index].push(*crat);
            }
        }
    }

    stacks.iter_mut().for_each(|v| v.reverse());

    Ok((rest, stacks))
}

fn parse_move(input: &[u8]) -> IResult<&[u8], Move> {
    terminated(
        map(
            tuple((
                preceded(tag(b"move "), parsers::u32),
                preceded(tag(b" from "), parsers::u32),
                preceded(tag(b" to "), parsers::u32),
            )),
            |(count, from, to)| Move { count, from, to },
        ),
        parsers::newline,
    )(input)
}

#[test]
fn test_parse_move() {
    assert_eq!(
        parse_move(b"move 23 from 1 to 5\n"),
        Ok((
            b"".as_slice(),
            Move {
                count: 23,
                from: 1,
                to: 5
            }
        ))
    );
}

#[test]
fn test_parse_stack() {
    assert_eq!(
        parse_stacks(
            b"    [D]
[N] [C]
[Z] [M] [P]
 1   2   3
"
        ),
        Ok((
            b"".as_slice(),
            vec![vec![b'Z', b'N'], vec![b'M', b'C', b'D'], vec![b'P']]
        ))
    )
}

crate::test_day!(RUN, "day5", "RFFFWBPNS", "CQQBBJFCS");
