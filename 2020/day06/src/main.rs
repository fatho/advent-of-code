use std::{
    collections::HashSet,
    error::Error,
    path::Path,
    path::PathBuf,
};

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
        eprintln!("Usage: aoc2020-day04 path/to/input [--validate-values]");
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

fn solve(path: &Path) -> Result<(usize, usize), Box<dyn Error>> {
    let data = std::fs::read_to_string(path)?;
    Ok(data
        .split("\n\n")
        .fold((0, 0), |(any_count, all_count), group| {
            (
                any_count + count_any_yes(group),
                all_count + count_all_yes(group),
            )
        }))
}

fn count_any_yes(group: &str) -> usize {
    gather_answers(group).len()
}

fn gather_answers(yesses: &str) -> HashSet<char> {
    yesses
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect::<HashSet<_>>()
}

fn count_all_yes(group: &str) -> usize {
    let mut answers_per_person = group.lines().map(gather_answers);
    let mut acc = answers_per_person.next().unwrap_or(HashSet::new());
    for other in answers_per_person {
        acc.retain(|ch| other.contains(ch));
    }
    acc.len()
}
