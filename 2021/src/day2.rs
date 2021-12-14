use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    multi::fold_many0,
    sequence::{separated_pair, terminated},
    IResult,
};

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    parsers::parse(
        map(
            fold_many0(
                terminated(CtrlCmd::parse, parsers::newline),
                || (0, 0),
                |(depth, x), cmd| match cmd.dir {
                    CtrlDir::Up => (depth - cmd.amount, x),
                    CtrlDir::Down => (depth + cmd.amount, x),
                    CtrlDir::Forward => (depth, x + cmd.amount),
                },
            ),
            |(depth, x)| format!("{}", depth * x),
        ),
        input,
    )
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    parsers::parse(
        map(
            fold_many0(
                terminated(CtrlCmd::parse, parsers::newline),
                || (0, 0, 0),
                |(aim, depth, x), cmd| match cmd.dir {
                    CtrlDir::Up => (aim - cmd.amount, depth, x),
                    CtrlDir::Down => (aim + cmd.amount, depth, x),
                    CtrlDir::Forward => (aim, depth + aim * cmd.amount, x + cmd.amount),
                },
            ),
            |(_, depth, x)| format!("{}", depth * x),
        ),
        input,
    )
}

#[derive(Debug, Clone, Copy)]
enum CtrlDir {
    Up,
    Down,
    Forward,
}

impl CtrlDir {
    fn parse(input: &[u8]) -> IResult<&[u8], CtrlDir> {
        alt((
            value(CtrlDir::Up, tag(b"up")),
            value(CtrlDir::Down, tag(b"down")),
            value(CtrlDir::Forward, tag(b"forward")),
        ))(input)
    }
}

struct CtrlCmd {
    dir: CtrlDir,
    amount: i64,
}

impl CtrlCmd {
    fn parse(input: &[u8]) -> IResult<&[u8], CtrlCmd> {
        map(
            separated_pair(CtrlDir::parse, tag(b" "), parsers::i64),
            |(dir, amount)| CtrlCmd { dir, amount },
        )(input)
    }
}

crate::test_day!(crate::day2::RUN, "day2", "1507611", "1880593125");
