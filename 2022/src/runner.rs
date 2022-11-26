use std::{
    fmt,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};

use anyhow::Context;
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
    #[structopt(short, long, required_unless_one(&["all"]))]
    day: Option<i32>,

    #[structopt(short, long, default_value("1"))]
    part: Part,

    #[structopt(short, long, conflicts_with_all(&["day", "part"]))]
    all: bool,

    /// Repeat the computation many times for easier flamegraphing.
    #[structopt(short, long, default_value("1"))]
    repeat: usize,

    /// Override input file
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// Run with example input
    #[structopt(short, long, conflicts_with_all(&["input"]))]
    example: bool,
}

#[derive(Clone, Copy)]
pub struct Day {
    pub part1: fn(&[u8]) -> anyhow::Result<String>,
    pub part2: fn(&[u8]) -> anyhow::Result<String>,
}

impl Day {
    pub fn unsolved() -> Self {
        fn no_solution(_: &[u8]) -> anyhow::Result<String> {
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
    let input_type = if opt.example { "example" } else { "input" };

    if opt.all {
        let base_dir = opt.input.unwrap_or_else(|| PathBuf::from("inputs"));

        let before = Instant::now();
        for (index, day) in days.iter().enumerate() {
            let inpath = base_dir.join(format!("day{}/{}.txt", index + 1, input_type));
            let contents =
                read_bytes(&inpath).with_context(|| format!("reading {}", inpath.display()))?;

            for _ in 0..opt.repeat {
                let before = Instant::now();
                let out1 = (day.part1)(&contents)
                    .with_context(|| format!("day{}.1: {}", index + 1, inpath.display()))?;
                let after_part1 = Instant::now();
                let out2 = (day.part2)(&contents)
                    .with_context(|| format!("day{}.2: {}", index + 1, inpath.display()))?;
                let after_part2 = Instant::now();

                println!(
                    "{}\t{:?}\t{:?}\t{:.3}\t{:.3}",
                    index + 1,
                    out1,
                    out2,
                    after_part1.duration_since(before).as_secs_f64() * 1000.0,
                    after_part2.duration_since(after_part1).as_secs_f64() * 1000.0,
                );
            }
        }
        let duration = before.elapsed();
        eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
    } else if let Some(day) = days.get((opt.day.unwrap() - 1) as usize) {
        let runner = match opt.part {
            Part::One => day.part1,
            Part::Two => day.part2,
        };
        let before = Instant::now();
        let infile = opt.input.unwrap_or_else(|| {
            PathBuf::from(format!("inputs/day{}/{}.txt", opt.day.unwrap(), input_type))
        });
        let contents = read_bytes(&infile)?;
        let mut output = None;
        for _ in 0..opt.repeat {
            output = Some(runner(&contents)?);
        }
        let duration = before.elapsed();
        if let Some(output) = output {
            println!("{}", output);
        }
        eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
    } else {
        anyhow::bail!("no such day")
    }
    Ok(())
}

fn read_bytes(inpath: &Path) -> Result<Vec<u8>, std::io::Error> {
    let mut file = std::fs::File::open(inpath)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
