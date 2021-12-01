use std::{
    collections::HashMap, collections::HashSet, error::Error, fmt::Display, path::Path,
    path::PathBuf, str::FromStr,
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
    let rules: Vec<Rule> = read_lines(path)?;

    // build graph where each bag points to those bags that can directly contain it
    let mut outer_map: HashMap<&str, Vec<&str>> = HashMap::new();
    // and another one where each bag points to the rule describing which bags it can contain
    let mut inner_map: HashMap<&str, &Rule> = HashMap::new();
    for rule in rules.iter() {
        for (_, contained) in rule.contains.iter() {
            let outer = outer_map.entry(contained).or_default();
            outer.push(&rule.color);
        }
        inner_map.insert(rule.color.as_ref(), rule);
    }

    // count number of nodes that eventually contain `shiny gold`.
    let mut visited: HashSet<&str> = HashSet::new();

    let mut work_stack = outer_map.get("shiny gold").cloned().unwrap_or_else(|| Vec::new());

    while let Some(node) = work_stack.pop() {
        if visited.insert(node) {
            if let Some(outer) = outer_map.get(node) {
                work_stack.extend(outer.iter())
            }
        }
    }

    // count number of bags inside a `shiny gold` bag
    let mut total_count: usize = 0;
    // The bag colors that still need to be processed, and how many we need of those
    let mut bag_stack = vec![(1, "shiny gold")];

    while let Some((multiplier, bag)) = bag_stack.pop() {
        if let Some(rule) = inner_map.get(bag) {
            for (count, color) in rule.contains.iter() {
                let this_count = multiplier * count;
                total_count += this_count;
                bag_stack.push((this_count, color));
            }
        }
    }


    Ok((visited.len(), total_count))
}

#[derive(Debug)]
struct Rule {
    color: String,
    contains: Vec<(usize, String)>,
}

#[derive(Debug)]
struct InvalidRuleError;

impl Display for InvalidRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule does not follow the expected format")
    }
}

impl Error for InvalidRuleError {}

impl FromStr for Rule {
    type Err = InvalidRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_rule(s).ok_or(InvalidRuleError)
    }
}

fn parse_rule(rule: &str) -> Option<Rule> {
    let mut words = rule.split_ascii_whitespace();

    fn expect<'a, I: Iterator<Item = &'a str>>(iter: &mut I, words: &[&str]) -> Option<()> {
        let next = iter.next()?;
        if words.contains(&next) {
            Some(())
        } else {
            None
        }
    }

    fn bag_color<'a, I: Iterator<Item = &'a str>>(iter: &mut I) -> Option<String> {
        let mut color = iter.next()?.to_owned();
        loop {
            let color_part = iter.next()?;
            if color_part.starts_with("bag") {
                return Some(color);
            } else {
                color.push(' ');
                color.push_str(color_part);
            }
        }
    }

    let head_color = bag_color(&mut words)?;

    expect(&mut words, &["contain"])?;

    let mut tail: Vec<(usize, String)> = Vec::new();

    while let Some(word) = words.next() {
        if word == "no" {
            expect(&mut words, &["other"])?;
            expect(&mut words, &["bags."])?;
            break;
        }
        let count = word.parse::<usize>().ok()?;
        let color = bag_color(&mut words)?;
        tail.push((count, color));
    }
    Some(Rule {
        color: head_color.to_owned(),
        contains: tail,
    })
}
