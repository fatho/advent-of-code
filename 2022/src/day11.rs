use std::cmp::Reverse;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::u64 as parse_u64,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let monkeys = parsers::parse(separated_list0(tag("\n"), parse_monkey), input)?;
    monkey_business(&monkeys, 20, |worry| worry / 3)
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let monkeys = parsers::parse(separated_list0(tag("\n"), parse_monkey), input)?;

    let common_mod = monkeys
        .iter()
        .map(|m| m.test.divisible_by)
        .fold(1, num::integer::lcm);

    monkey_business(&monkeys, 10000, |worry| worry % common_mod)
}

fn monkey_business(
    monkeys: &[Monkey],
    rounds: usize,
    anxiety_meds: impl Fn(u64) -> u64,
) -> anyhow::Result<String> {
    let mut items: Vec<_> = monkeys.iter().map(|m| m.starting_items.clone()).collect();
    let mut inspections: Vec<usize> = vec![0; items.len()];
    let mut inspecting = Vec::new();

    for _ in 0..rounds {
        for (index, m) in monkeys.iter().enumerate() {
            std::mem::swap(&mut items[index], &mut inspecting);
            inspections[index] += inspecting.len();

            for item_worry in inspecting.drain(..) {
                let new_worry = anxiety_meds(m.op.eval(item_worry));
                let target = m.test.eval(new_worry) as usize;
                items[target].push(new_worry);
            }
        }
    }

    let (top, second, _rest) = inspections.select_nth_unstable_by_key(1, |count| Reverse(*count));

    Ok((top[0] * *second).to_string())
}

fn parse_monkey(input: &[u8]) -> IResult<&[u8], Monkey> {
    map(
        tuple((
            delimited(tag("Monkey "), parse_u64, tag(":\n")),
            delimited(
                tag("  Starting items: "),
                separated_list0(tag(", "), parse_u64),
                tag("\n"),
            ),
            delimited(tag("  Operation: new = "), parse_operation, tag("\n")),
            delimited(tag("  Test: divisible by "), parse_u64, tag("\n")),
            delimited(tag("    If true: throw to monkey "), parse_u64, tag("\n")),
            delimited(tag("    If false: throw to monkey "), parse_u64, tag("\n")),
        )),
        |(_index, items, op, div_by, throw_true, throw_false)| Monkey {
            starting_items: items,
            op,
            test: Test {
                divisible_by: div_by,
                true_monkey: throw_true as usize,
                false_monkey: throw_false as usize,
            },
        },
    )(input)
}

fn parse_operation(input: &[u8]) -> IResult<&[u8], Operation> {
    map(
        tuple((
            terminated(parse_operand, tag(" ")),
            terminated(parse_operator, tag(" ")),
            parse_operand,
        )),
        |(lhs, op, rhs)| Operation { lhs, op, rhs },
    )(input)
}

fn parse_operand(input: &[u8]) -> IResult<&[u8], Operand> {
    alt((
        map(tag("old"), |_| Operand::Old),
        map(parse_u64, Operand::Const),
    ))(input)
}

fn parse_operator(input: &[u8]) -> IResult<&[u8], Operator> {
    alt((
        map(tag("+"), |_| Operator::Add),
        map(tag("*"), |_| Operator::Mul),
    ))(input)
}

#[derive(Clone, Debug)]
struct Monkey {
    starting_items: Vec<u64>,
    op: Operation,
    test: Test,
}

#[derive(Clone, Debug)]
struct Test {
    divisible_by: u64,
    true_monkey: usize,
    false_monkey: usize,
}

impl Test {
    fn eval(&self, value: u64) -> usize {
        if value % self.divisible_by == 0 {
            self.true_monkey
        } else {
            self.false_monkey
        }
    }
}

#[derive(Clone, Debug)]
struct Operation {
    lhs: Operand,
    op: Operator,
    rhs: Operand,
}

impl Operation {
    fn eval(&self, old: u64) -> u64 {
        let lhs = self.lhs.eval(old);
        let rhs = self.rhs.eval(old);
        match self.op {
            Operator::Add => lhs + rhs,
            Operator::Mul => lhs * rhs,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Add,
    Mul,
}

#[derive(Clone, Copy, Debug)]
enum Operand {
    Old,
    Const(u64),
}

impl Operand {
    fn eval(&self, old: u64) -> u64 {
        match self {
            Operand::Old => old,
            Operand::Const(c) => *c,
        }
    }
}

crate::test_day!(RUN, "day11", "54054", "14314925001");
