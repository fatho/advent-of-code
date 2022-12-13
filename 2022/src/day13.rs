#![allow(unused)]

use anyhow::bail;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map, opt},
    multi::{fold_many0, many0, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (sum, _) = parsers::parse(
        fold_many0(
            terminated(parse_pair, opt(newline)),
            || (0, 1),
            |(sum, index), (v1, v2)| {
                (
                    sum + if matches!(v1.cmp(&v2), std::cmp::Ordering::Less) {
                        index
                    } else {
                        0
                    },
                    index + 1,
                )
            },
        ),
        input,
    )?;
    Ok(sum.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut packets = parsers::parse(
        many0(terminated(parse_val, take_while1(|ch| ch == b'\n'))),
        input,
    )?;
    let div1 = Val::List(vec![Val::List(vec![Val::Atom(2)])]);
    let div2 = Val::List(vec![Val::List(vec![Val::Atom(6)])]);
    packets.push(div1.clone());
    packets.push(div2.clone());
    packets.sort_unstable();

    let pos1 = packets
        .iter()
        .position(|v| v == &div1)
        .expect("dividers should still be there")
        + 1;
    let pos2 = packets
        .iter()
        .position(|v| v == &div2)
        .expect("dividers should still be there")
        + 1;

    Ok((pos1 * pos2).to_string())
}

fn parse_val(input: &[u8]) -> IResult<&[u8], Val> {
    alt((
        map(nom::character::complete::u32, Val::Atom),
        map(
            delimited(tag("["), separated_list0(tag(","), parse_val), tag("]")),
            Val::List,
        ),
    ))(input)
}

fn parse_pair(input: &[u8]) -> IResult<&[u8], (Val, Val)> {
    pair(
        terminated(parse_val, newline),
        terminated(parse_val, newline),
    )(input)
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum Val {
    Atom(u32),
    List(Vec<Val>),
}

impl Ord for Val {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn cmp_list(l: &[Val], r: &[Val]) -> std::cmp::Ordering {
            let mut ixs = l.iter();
            let mut iys = r.iter();
            loop {
                match (ixs.next(), iys.next()) {
                    (None, None) => return std::cmp::Ordering::Equal,
                    (None, Some(_)) => return std::cmp::Ordering::Less,
                    (Some(_), None) => return std::cmp::Ordering::Greater,
                    (Some(x), Some(y)) => match x.cmp(y) {
                        std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
                        std::cmp::Ordering::Equal => continue,
                        std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
                    },
                }
            }
        }

        match (self, other) {
            (Val::Atom(x), Val::Atom(y)) => x.cmp(y),
            (Val::Atom(x), Val::List(ys)) => cmp_list(&[Val::Atom(*x)], ys),
            (Val::List(xs), Val::Atom(y)) => cmp_list(xs, &[Val::Atom(*y)]),
            (Val::List(xs), Val::List(ys)) => cmp_list(xs, ys),
        }
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// TODO: try defining order predicate directly on input strings/on more efficient representation

crate::test_day!(RUN, "day13", "5252", "20592");
