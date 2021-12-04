use std::{fmt, path::PathBuf, str::FromStr, time::Instant};

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
#[structopt(
    name = "advent-of-code",
    about = "Solutions for Advent of Code puzzles."
)]
pub struct AocOpt {
    #[structopt(short, long)]
    day: i32,

    #[structopt(short, long, default_value("1"))]
    part: Part,

    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Copy)]
pub struct Day {
    pub part1: fn(&mut dyn std::io::Read) -> anyhow::Result<()>,
    pub part2: fn(&mut dyn std::io::Read) -> anyhow::Result<()>,
}

impl Day {
    pub fn unsolved() -> Self {
        fn no_solution(_: &mut dyn std::io::Read) -> anyhow::Result<()> {
            anyhow::bail!("no solution for this day");
        }
        Self {
            part1: no_solution,
            part2: no_solution,
        }
    }
}

pub fn aoc_main(days: &[Day]) -> anyhow::Result<()> {
    let opt = AocOpt::from_args();
    if let Some(day) = days.get((opt.day - 1) as usize) {
        let runner = match opt.part {
            Part::One => day.part1,
            Part::Two => day.part2,
        };
        let mut input = std::fs::File::open(opt.input)?;
        let before = Instant::now();
        runner(&mut input)?;
        let duration = before.elapsed();
        eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
        Ok(())
    } else {
        anyhow::bail!("no such day")
    }
}
