use std::{
    error::Error,
    fmt::{Debug, Display},
    path::Path,
    path::PathBuf,
    str::FromStr,
};

use aoc2020_common::read_lines;

fn main() {
    if let Some(path) = parse_args() {
        match solve(&path) {
            Ok(result) => {
                println!("{}: {:?}", path.display(), result);
            }
            Err(err) => {
                println!("{}: {}", path.display(), err);
            }
        }
    } else {
        eprintln!("Usage: aoc2020-day02 path/to/input");
        std::process::exit(1);
    }
}

fn parse_args() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next()?.into();
    if args.next().is_none() {
        Some(path)
    } else {
        None
    }
}

/// Solve both parts of the riddle.
/// The nth component of the tuple is the solution to the nth part of the riddle.
fn solve(path: &Path) -> Result<(usize, usize), Box<dyn Error>> {
    let password_entries: Vec<Entry> = read_lines(path)?;
    let range_valid = password_entries
        .iter()
        .filter(|e| e.is_valid_range())
        .count();
    let exact_valid = password_entries
        .iter()
        .filter(|e| e.is_valid_exact())
        .count();
    Ok((range_valid, exact_valid))
}

#[derive(Debug)]
struct Entry {
    policy: Policy,
    password: String,
}

impl Entry {
    fn is_valid_exact(&self) -> bool {
        self.policy.validate_exact(&self.password)
    }

    fn is_valid_range(&self) -> bool {
        self.policy.validate_range(&self.password)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MalformedEntry;

impl Error for MalformedEntry {}

impl Display for MalformedEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entry has not the correct format")
    }
}

impl FromStr for Entry {
    type Err = MalformedEntry;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        (|| {
            let sep = s.find(':')?;
            let policy = s[..sep].trim();
            let password = s[sep + 1..].trim();

            let sep = policy.find(' ')?;
            let range = policy[0..sep].trim();
            let ch = policy[sep + 1..].chars().next()?;
            let sep = range.find('-')?;

            let min = range[0..sep].parse().ok()?;
            let max = range[sep + 1..].parse().ok()?;

            Some(Entry {
                policy: Policy { min, max, ch },
                password: password.to_owned(),
            })
        })()
        .ok_or(MalformedEntry)
    }
}

#[derive(Debug)]
struct Policy {
    min: usize,
    max: usize,
    ch: char,
}

impl Policy {
    fn validate_exact(&self, password: &str) -> bool {
        // It's unclear whether Toboggan Rental Shop passwords may be arbitrary unicode or not,
        // hence we do an O(n) scan instead of indexing directly.
        1 == password
            .chars()
            .enumerate()
            .filter(|(pos, ch)| (pos + 1 == self.min || pos + 1 == self.max) && *ch == self.ch)
            .count()
    }

    fn validate_range(&self, password: &str) -> bool {
        let count = password.chars().filter(|ch| *ch == self.ch).count();
        count >= self.min && count <= self.max
    }
}
