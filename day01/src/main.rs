use std::{
    error::Error,
    path::Path,
    path::PathBuf,
};

use aoc2020_common::read_lines;

fn main() {
    if let Some((path, count)) = parse_args() {
        match solve(&path, count) {
            Ok(result) => {
                println!("{}: {}", path.display(), result);
            }
            Err(err) => {
                println!("{}: {}", path.display(), err);
            }
        }
    } else {
        eprintln!("Usage: aoc2020-day01 path/to/input <SIZE-OF-SUM>");
        std::process::exit(1);
    }
}

fn parse_args() -> Option<(PathBuf, usize)> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next()?.into();
    let count = args.next()?.to_str()?.parse().ok()?;
    if args.next().is_none() {
        Some((path, count))
    } else {
        None
    }
}

fn solve(path: &Path, count: usize) -> Result<u32, Box<dyn Error>> {
    let mut numbers: Vec<u32> = read_lines(path)?;

    numbers.sort();
    find_product(count, 2020, &numbers).ok_or(
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No numbers add up to 2020").into(),
    )
}

fn find_product(count: usize, sum: u32, sorted_nums: &[u32]) -> Option<u32> {
    if count == 0 {
        if sum == 0 {
            // With zero terms we can only form the empty sum `0`, and the empty product `1`.
            Some(1)
        } else {
            None
        }
    } else if count == 1 {
        // This case is not really needed, but allows for a small optimization:
        // We can directly check whether the `sum` is part of the set,
        // instead of checking whether there's an `x` in the set such that `sum - x == 0`.
        if sorted_nums.binary_search(&sum).is_ok() {
            // product of just this one number is itself
            Some(sum)
        } else {
            None
        }
    } else {
        // Assumption: There are no negative numbers.
        // This allows us to disregard any numbers greater than `sum`, as those
        // can never be part of a sum that adds up to exactly `sum`.
        let cutoff = match sorted_nums.binary_search(&sum) {
            Ok(pos) => pos + 1,
            Err(pos) => pos,
        };

        for i in 0..cutoff {
            let this_num = sorted_nums[i];
            // We know sum >= all numbers below the cutoff point.
            let remaining_sum = sum - this_num;
            if let Some(remaining_prod) = find_product(count - 1, remaining_sum, &sorted_nums[0..i])
            {
                return Some(this_num * remaining_prod);
            }
        }
        None
    }
}
