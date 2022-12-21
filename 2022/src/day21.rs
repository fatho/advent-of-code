#![allow(unused)]

use std::fmt::Display;

use anyhow::{bail, Context};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while1},
    combinator::{map, map_opt, map_res},
    multi::{many0, many1},
    sequence::{delimited, separated_pair, terminated, tuple},
    IResult,
};
use rustc_hash::FxHashMap;

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let monkeys = parsers::parse(many1(terminated(parse_monkey, newline)), input)?;

    let mut monkey_lookup: FxHashMap<MonkeyId, _> =
        monkeys.into_iter().map(|m| (m.id, m.expr)).collect();

    eval(&monkey_lookup, MonkeyId::ROOT).map(|ret| ret.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let monkeys = parsers::parse(many1(terminated(parse_monkey, newline)), input)?;
    let mut monkey_lookup: FxHashMap<MonkeyId, _> =
        monkeys.into_iter().map(|m| (m.id, m.expr)).collect();

    let mut contains_human =
        FxHashMap::with_capacity_and_hasher(monkey_lookup.len(), Default::default());
    contains_human.insert(MonkeyId::HUMAN, true);

    fn find_human<'a>(
        monkeys: &'_ FxHashMap<MonkeyId<'a>, Expr<'a>>,
        human: &'_ mut FxHashMap<MonkeyId<'a>, bool>,
        node: MonkeyId<'a>,
    ) -> anyhow::Result<bool> {
        if node == MonkeyId::HUMAN {
            Ok(true)
        } else {
            match monkeys.get(&node) {
                None => bail!("invalid monkey reference: {}", node),
                Some(expr) => {
                    let has_human = match expr {
                        Expr::Const(ret) => false,
                        Expr::BinOp(lhs, op, rhs) => {
                            let human_left = find_human(monkeys, human, *lhs)?;
                            let human_right = find_human(monkeys, human, *rhs)?;

                            human_left || human_right
                        }
                    };

                    human.insert(node, has_human);
                    Ok(has_human)
                }
            }
        }
    }

    let _ = find_human(&monkey_lookup, &mut contains_human, MonkeyId::ROOT);

    let (root_left, root_right) = match monkey_lookup
        .get(&MonkeyId::ROOT)
        .context("root not found")?
    {
        Expr::Const(_) => bail!("root is const"),
        Expr::BinOp(left, _, right) => (*left, *right),
    };

    let (mut human, mut no_human) = if contains_human[&root_left] {
        (root_left, eval(&monkey_lookup, root_right)?)
    } else {
        (root_right, eval(&monkey_lookup, root_left)?)
    };

    while human != MonkeyId::HUMAN {
        match monkey_lookup[&human] {
            Expr::Const(_) => unreachable!("impossible, missed human, this is a bug"),
            Expr::BinOp(lhs, op, rhs) => {
                // lhs `op` rhs = no_human

                if contains_human[&lhs] {
                    let inv = match op {
                        Op::Add => Op::Sub,
                        Op::Sub => Op::Add,
                        Op::Mul => Op::Div,
                        Op::Div => Op::Mul,
                    };
                    // lhs + rhs = no_human => lhs = no_human - rhs
                    // lhs - rhs = no_human => lhs = no_human + rhs
                    // lhs * rhs = no_human => lhs = no_human / rhs
                    // lhs / rhs = no_human => lhs = no_human * rhs

                    no_human = eval_op(inv, no_human, eval(&monkey_lookup, rhs)?);
                    human = lhs;
                } else {
                    // lhs `op` rhs = no_human

                    // lhs + rhs = no_human => no_human - lhs = rhs
                    // lhs - rhs = no_human => lhs - no_human = rhs
                    // lhs * rhs = no_human => no_human / lhs = rhs
                    // lhs / rhs = no_human => lhs / no_human = rhs

                    let lhs = eval(&monkey_lookup, lhs)?;
                    no_human = match op {
                        Op::Add => no_human - lhs,
                        Op::Sub => lhs - no_human,
                        Op::Mul => no_human / lhs,
                        Op::Div => lhs / no_human,
                    };

                    human = rhs;
                }
            }
        }
    }

    Ok(no_human.to_string())
}

fn eval<'a>(
    monkeys: &'_ FxHashMap<MonkeyId<'a>, Expr<'a>>,
    node: MonkeyId<'a>,
) -> anyhow::Result<i64> {
    match monkeys.get(&node) {
        None => bail!("invalid monkey reference: {}", node),
        Some(expr) => match expr {
            Expr::Const(ret) => Ok(*ret),
            Expr::BinOp(lhs, op, rhs) => {
                let lhs = eval(monkeys, *lhs)?;
                let rhs = eval(monkeys, *rhs)?;
                Ok(eval_op(*op, lhs, rhs))
            }
        },
    }
}

fn eval_op(op: Op, lhs: i64, rhs: i64) -> i64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Mul => lhs * rhs,
        Op::Div => lhs / rhs,
    }
}

fn parse_monkey(input: &[u8]) -> IResult<&[u8], Monkey> {
    map(
        separated_pair(parse_id, tag(": "), parse_expr),
        |(id, expr)| Monkey { id, expr },
    )(input)
}

fn parse_id(input: &[u8]) -> IResult<&[u8], MonkeyId> {
    map(take_while1(|ch| (b'a'..=b'z').contains(&ch)), MonkeyId)(input)
}

fn parse_expr(input: &[u8]) -> IResult<&[u8], Expr> {
    alt((
        map(nom::character::complete::u32, |x| Expr::Const(x as i64)),
        map(
            tuple((parse_id, delimited(tag(" "), parse_op, tag(" ")), parse_id)),
            |(lhs, op, rhs)| Expr::BinOp(lhs, op, rhs),
        ),
    ))(input)
}

fn parse_op(input: &[u8]) -> IResult<&[u8], Op> {
    map_opt(take(1usize), |op: &[u8]| match op {
        b"+" => Some(Op::Add),
        b"-" => Some(Op::Sub),
        b"*" => Some(Op::Mul),
        b"/" => Some(Op::Div),
        _ => None,
    })(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Monkey<'a> {
    id: MonkeyId<'a>,
    expr: Expr<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MonkeyId<'a>(&'a [u8]);

impl<'a> Display for MonkeyId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.0))
    }
}

impl MonkeyId<'static> {
    const ROOT: Self = MonkeyId(b"root");
    const HUMAN: Self = MonkeyId(b"humn");
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr<'a> {
    Const(i64),
    BinOp(MonkeyId<'a>, Op, MonkeyId<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

crate::test_day!(RUN, "day21", "83056452926300", "3469704905529");
