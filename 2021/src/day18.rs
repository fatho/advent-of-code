#![allow(unused_imports)]

use std::fmt::Display;

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let nums = parsers::parse(many0(terminated(p_num, parsers::newline)), input)?;
    let mut result = nums[0].clone();
    for n in &nums[1..] {
        result = add(result, n.clone());
        result.reduce();
    }
    Ok(result.magnitude().to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let nums = parsers::parse(many0(terminated(p_num, parsers::newline)), input)?;

    let result = nums
        .iter()
        .flat_map(|l| nums.iter().map(move |r| (l, r)))
        .map(|(l, r)| add(l.clone(), r.clone()))
        .map(|mut x| {
            x.reduce();
            x.magnitude()
        })
        .max()
        .unwrap();

    Ok(result.to_string())
}

fn add(a: Num, b: Num) -> Num {
    Num::Pair(Box::new(a), Box::new(b))
}

fn p_num(input: &[u8]) -> IResult<&[u8], Num> {
    alt((
        map(numbers::i64, Num::Reg),
        map(
            delimited(tag("["), separated_pair(p_num, tag(","), p_num), tag("]")),
            |(l, r)| Num::Pair(Box::new(l), Box::new(r)),
        ),
    ))(input)
}

// TODO: use better representation for faster operations
#[derive(Debug, Clone, PartialEq, Eq)]
enum Num {
    Reg(i64),
    Pair(Box<Num>, Box<Num>),
}

impl Num {
    fn magnitude(&self) -> i64 {
        match self {
            Num::Reg(n) => *n,
            Num::Pair(l, r) => l.magnitude() * 3 + r.magnitude() * 2,
        }
    }

    fn reduce(&mut self) {
        loop {
            if self.explode(0).is_some() {
                continue;
            }
            if self.split() {
                continue;
            }
            break;
        }
    }

    fn explode(&mut self, depth: u32) -> Option<(Option<i64>, Option<i64>)> {
        match self {
            Num::Reg(n) => None,
            Num::Pair(l, r) => {
                if depth == 4 {
                    // Exploding pairs always consist of regular numbers
                    let ln = match l.as_ref() {
                        Num::Reg(n) => *n,
                        Num::Pair(_, _) => unreachable!(),
                    };
                    let rn = match r.as_ref() {
                        Num::Reg(n) => *n,
                        Num::Pair(_, _) => unreachable!(),
                    };
                    // explode this
                    *self = Num::Reg(0);
                    Some((Some(ln), Some(rn)))
                } else if let Some((ln, rn)) = l.explode(depth + 1) {
                    if let Some(rn) = rn {
                        r.add_to_leftmost(rn);
                    }
                    Some((ln, None))
                } else if let Some((ln, rn)) = r.explode(depth + 1) {
                    if let Some(ln) = ln {
                        l.add_to_rightmost(ln);
                    }
                    Some((None, rn))
                } else {
                    None
                }
            }
        }
    }

    fn add_to_leftmost(&mut self, x: i64) {
        match self {
            Num::Reg(n) => *n += x,
            Num::Pair(l, _) => l.add_to_leftmost(x),
        }
    }

    fn add_to_rightmost(&mut self, x: i64) {
        match self {
            Num::Reg(n) => *n += x,
            Num::Pair(_, r) => r.add_to_rightmost(x),
        }
    }

    fn split(&mut self) -> bool {
        match self {
            Num::Reg(n) => {
                let n = *n;
                if n >= 10 {
                    let l = Num::Reg(n / 2);
                    let r = Num::Reg((n + 1) / 2);
                    *self = Num::Pair(Box::new(l), Box::new(r));
                    true
                } else {
                    false
                }
            }
            Num::Pair(l, r) => l.split() || r.split(),
        }
    }
}

impl Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num::Reg(n) => write!(f, "{}", n),
            Num::Pair(l, r) => write!(f, "[{},{}]", l, r),
        }
    }
}

crate::test_day!(crate::day18::RUN, "day18", "4289", "4807");
