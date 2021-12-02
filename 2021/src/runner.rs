use std::{str::FromStr, path::PathBuf, fmt, time::Instant};

use structopt::StructOpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    One,
    Two,
}

#[derive(Debug, Clone)]
pub struct NoPart;

impl fmt::Display for NoPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "only part 1 and 2 are valid")
    }
}

impl FromStr for Part {
    type Err = NoPart;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Part::One),
            "2" => Ok(Part::Two),
            _ => Err(NoPart),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "advent-of-code", about = "Solutions for Advent of Code puzzles.")]
pub struct AocOpt {
    #[structopt(short, long)]
    day: i32,

    #[structopt(short, long, default_value("1"))]
    part: Part,

    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}


pub struct Day {
    pub first: fn(&mut dyn std::io::Read) -> std::io::Result<()>,
    pub second: fn(&mut dyn std::io::Read) -> std::io::Result<()>,
}

impl Day {
    pub fn unsolved() -> Self {
        fn no_solution(_: &mut dyn std::io::Read) -> std::io::Result<()> {
            Err(std::io::ErrorKind::Unsupported.into())
        }
        Self {
            first: no_solution,
            second: no_solution,
        }
    }
}

pub fn aoc_main(days: &[Day]) -> Result<(), std::io::Error> {
    let opt = AocOpt::from_args();
    if let Some(day) = days.get((opt.day - 1) as usize) {
        let runner = match opt.part {
            Part::One => day.first,
            Part::Two => day.second,
        };
        let mut input = std::fs::File::open(opt.input)?;
        let before = Instant::now();
        runner(&mut input)?;
        let duration = before.elapsed();
        eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
        Ok(())
    } else {
        Err(std::io::ErrorKind::NotFound.into())
    }
}
