use std::{error::Error, path::Path, path::PathBuf};

fn main() {
    if let Some((path, validate_values)) = parse_args() {
        match solve(&path, validate_values) {
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

fn parse_args() -> Option<(PathBuf, bool)> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next()?.into();
    let validate_values = args.next() == Some("--validate-values".into());
    if args.next().is_none() {
        Some((path, validate_values))
    } else {
        None
    }
}

fn solve(path: &Path, validate_values: bool) -> Result<usize, Box<dyn Error>> {
    let batch_data = std::fs::read_to_string(path)?;
    let valid_count = batch_data
        .split("\n\n")
        .filter(|pp| validate_passport(pp, validate_values))
        .count();
    Ok(valid_count)
}

fn validate_passport(passport_data: &str, validate_values: bool) -> bool {
    let mut byr = false; // (Birth Year) - four digits; at least 1920 and at most 2002.
    let mut iyr = false; // (Issue Year) - four digits; at least 2010 and at most 2020.
    let mut eyr = false; // (Expiration Year) - four digits; at least 2020 and at most 2030.
    let mut hgt = false; // (Height) - a number followed by either cm or in:
                         //     If cm, the number must be at least 150 and at most 193.
                         //     If in, the number must be at least 59 and at most 76.
    let mut hcl = false; // (Hair Color) - a # followed by exactly six characters 0-9 or a-f.
    let mut ecl = false; // (Eye Color) - exactly one of: amb blu brn gry grn hzl oth.
    let mut pid = false; // (Passport ID) - a nine-digit number, including leading zeroes.
    passport_data
        .split_ascii_whitespace()
        .filter_map(|field| {
            let sep = field.find(':')?;
            Some((&field[..sep], &field[sep + 1..]))
        })
        .for_each(|(name, value)| match name {
            "byr" => byr = !validate_values || is_year_in_range(value, 1920, 2002),
            "iyr" => iyr = !validate_values || is_year_in_range(value, 2010, 2020),
            "eyr" => eyr = !validate_values || is_year_in_range(value, 2020, 2030),
            "hgt" => hgt = !validate_values || is_height_valid(value),
            "hcl" => hcl = !validate_values || is_hex_color(value),
            "ecl" => ecl = !validate_values || EYE_COLORS.contains(&value),
            "pid" => pid = !validate_values || is_passport_id(value),
            _ => (), // unexpected, but not explicitly invalid
        });

    byr && iyr && eyr && hgt && hcl && ecl && pid
}

fn is_year_in_range(input: &str, min: u32, max: u32) -> bool {
    if let Ok(year) = input.parse::<u32>() {
        year >= min && year <= max
    } else {
        false
    }
}

fn is_hex_color(input: &str) -> bool {
    input.len() == 7
        && input.starts_with('#')
        && input.chars().skip(1).all(|ch| ch.is_ascii_hexdigit())
}

fn is_passport_id(input: &str) -> bool {
    input.len() == 9 && input.chars().all(|ch| ch.is_ascii_digit())
}

fn is_height_valid(input: &str) -> bool {
    if input.ends_with("cm") {
        input[..input.len() - 2]
            .parse::<u32>()
            .map_or(false, |num| num >= 150 && num <= 193)
    } else if input.ends_with("in") {
        input[..input.len() - 2]
            .parse::<u32>()
            .map_or(false, |num| num >= 59 && num <= 76)
    } else {
        false
    }
}

static EYE_COLORS: &[&str] = &["amb", "blu", "brn", "gry", "grn", "hzl", "oth"];
