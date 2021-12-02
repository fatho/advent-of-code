use advent_of_code_2021::{BufReadExt, aoc_main, Day};
use std::collections::VecDeque;
use std::io::{self, Read};
use std::str::FromStr;


fn main() -> std::io::Result<()> {
    aoc_main(&[
        Day { first: day1_1, second: day1_2 },
        Day { first: day2_1, second: day2_2 },
    ])
}

fn day1_1(input: &mut dyn Read) -> std::io::Result<()> {
    day1_impl(input, 1)
}

fn day1_2(input: &mut dyn Read) -> std::io::Result<()> {
    day1_impl(input, 3)
}

fn day1_impl(input: &mut dyn Read, window_size: usize) -> std::io::Result<()> {
    let mut reader = io::BufReader::new(input);
    let mut line = String::new();
    let mut increases = 0;
    let mut window_sum: i32 = 0;
    let mut buffer = VecDeque::new();

    for _ in 0..window_size {
        let reading = reader.read_parse::<i32>(&mut line)?;
        window_sum += reading;
        buffer.push_back(reading);
    }

    while let Some(new_reading) = reader.read_parse_or_eof(&mut line)? {
        let old_reading = buffer.pop_front().expect("window_size must be > 0");
        buffer.push_back(new_reading);
        let new_window = window_sum + new_reading - old_reading;

        if new_window > window_sum {
            increases += 1;
        }

        window_sum = new_window;
        line.clear();
    }
    println!("{}", increases);
    Ok(())
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

fn day2_1(input: &mut dyn Read) -> std::io::Result<()> {
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

fn day2_2(input: &mut dyn Read) -> std::io::Result<()> {
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
}