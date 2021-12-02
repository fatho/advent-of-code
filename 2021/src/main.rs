use advent_of_code_2021::BufReadExt;
use std::collections::VecDeque;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long)]
    day: i32,

    #[structopt(short, long, default_value("1"))]
    part: i32,

    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let before = Instant::now();
    match opt.day {
        1 => day1(&opt.input, if opt.part == 1 { 1 } else { 3 })?,
        2 => day2(&opt.input, opt.part != 1)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "This day is not implemented yet",
            )
            .into());
        }
    }
    let duration = before.elapsed();
    eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
    Ok(())
}

fn day1(input: &Path, window_size: usize) -> Result<(), Box<dyn Error>> {
    let mut reader = io::BufReader::new(std::fs::File::open(input)?);
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

fn day2(input: &Path, use_aim: bool) -> Result<(), Box<dyn Error>> {
    let mut reader = io::BufReader::new(std::fs::File::open(input)?);
    let mut line = String::new();
    
    let mut depth = 0;
    let mut aim = 0;
    let mut x = 0;
    
    while let Some(cmd) = reader.read_parse_or_eof::<CtrlCmd>(&mut line)? {
        if use_aim {
            match cmd.dir {
                CtrlDir::Up => aim -= cmd.amount,
                CtrlDir::Down => aim += cmd.amount,
                CtrlDir::Forward => {
                    x += cmd.amount;
                    depth += aim * cmd.amount;
                },
            }
        } else {
            match cmd.dir {
                CtrlDir::Up => depth -= cmd.amount,
                CtrlDir::Down => depth += cmd.amount,
                CtrlDir::Forward => x += cmd.amount,
            }
        }
    }
    println!("{}", depth * x);
    Ok(())
}