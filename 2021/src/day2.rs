use crate::{BufReadExt};
use std::io::{self, Read};
use std::str::FromStr;

pub fn part1(input: &mut dyn Read) -> std::io::Result<()> {
    let mut reader = io::BufReader::new(input);
    let mut line = String::new();
    
    let mut depth = 0;
    let mut x = 0;
    
    while let Some(cmd) = reader.read_parse_or_eof::<CtrlCmd>(&mut line)? {
        match cmd.dir {
            CtrlDir::Up => depth -= cmd.amount,
            CtrlDir::Down => depth += cmd.amount,
            CtrlDir::Forward => x += cmd.amount,
        }
    }
    println!("{}", depth * x);
    Ok(())
}

pub fn part2(input: &mut dyn Read) -> std::io::Result<()> {
    let mut reader = io::BufReader::new(input);
    let mut line = String::new();
    
    let mut depth = 0;
    let mut aim = 0;
    let mut x = 0;
    
    while let Some(cmd) = reader.read_parse_or_eof::<CtrlCmd>(&mut line)? {
        match cmd.dir {
            CtrlDir::Up => aim -= cmd.amount,
            CtrlDir::Down => aim += cmd.amount,
            CtrlDir::Forward => {
                x += cmd.amount;
                depth += aim * cmd.amount;
            },
        }
    }
    println!("{}", depth * x);
    Ok(())
}enum CtrlDir {
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
            _ => Err(())
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
