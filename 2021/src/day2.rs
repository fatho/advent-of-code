use crate::{Day, FileParser};
use std::io::Read;
use std::str::FromStr;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);

    let mut depth = 0;
    let mut x = 0;

    for cmd in parser.iter_parse::<CtrlCmd>() {
        match cmd.dir {
            CtrlDir::Up => depth -= cmd.amount,
            CtrlDir::Down => depth += cmd.amount,
            CtrlDir::Forward => x += cmd.amount,
        }
    }

    parser.finish()?;

    Ok((depth * x) as i64)
}

pub fn part2(input: &mut dyn Read) -> anyhow::Result<i64> {
    let mut parser = FileParser::new(input);

    let mut depth = 0;
    let mut aim = 0;
    let mut x = 0;

    for cmd in parser.iter_parse::<CtrlCmd>() {
        match cmd.dir {
            CtrlDir::Up => aim -= cmd.amount,
            CtrlDir::Down => aim += cmd.amount,
            CtrlDir::Forward => {
                x += cmd.amount;
                depth += aim * cmd.amount;
            }
        }
    }
    parser.finish()?;
    Ok((depth * x) as i64)
}

enum CtrlDir {
    Up,
    Down,
    Forward,
}

struct CtrlCmd {
    dir: CtrlDir,
    amount: i64,
}

impl FromStr for CtrlDir {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(CtrlDir::Up),
            "down" => Ok(CtrlDir::Down),
            "forward" => Ok(CtrlDir::Forward),
            _ => Err(()),
        }
    }
}

impl FromStr for CtrlCmd {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (cmd, amount) = s.split_once(' ').ok_or(())?;
        Ok(CtrlCmd {
            dir: cmd.parse().map_err(|_| ())?,
            amount: amount.parse().map_err(|_| ())?,
        })
    }
}

crate::test_day!(RUN, "day2", 1507611, 1880593125);
