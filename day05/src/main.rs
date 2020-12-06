use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
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

fn solve(path: &Path) -> Result<(u32, u32), Box<dyn Error>> {
    // The seat numbers are just binary-encoded numbers using F/L as 0 and B/R as 1.
    let mut seat_ids: Vec<u32> = BufReader::new(File::open(path)?)
        .split(b'\n')
        .map(|line_or_err| line_or_err.map(|l| parse_seat_id(&l)))
        .collect::<Result<_, _>>()?;

    seat_ids.sort();
    // Find discontinuity
    let my_seat = seat_ids
        .iter()
        .zip(seat_ids.iter().skip(1))
        .find_map(|(cur, next)| if cur + 2 == *next {
            Some(cur + 1)
        } else {
            None
        })
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "no free seat found")
        })?;
    // Sanity check: highest seat ID
    let max_id = seat_ids.last().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "must have at least one seat number for determining maximum",
        )
    })?;

    Ok((*max_id, my_seat))
}

fn parse_seat_id(line: &[u8]) -> u32 {
    let mut id = 0;
    for x in line.iter() {
        id = id << 1;
        if *x == b'B' || *x == b'R' {
            id += 1;
        }
    }
    id
}

#[test]
fn test_seat_number_from_str() {
    assert_eq!(parse_seat_id(b"BFFFBBFRRR"), 567);
    assert_eq!(parse_seat_id(b"FFFBBBFRRR"), 119);
    assert_eq!(parse_seat_id(b"BBFFBBFRLL"), 820);
}
