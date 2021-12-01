use std::{
    error::Error,
    fmt::{Debug, Display},
    path::Path,
    path::PathBuf,
    str::FromStr,
};

use aoc2020_common::read_lines;

fn main() {
    if let Some((path, slopes)) = parse_args() {
        match solve(&path, &slopes) {
            Ok(result) => {
                println!("{}: {:?}", path.display(), result);
            }
            Err(err) => {
                println!("{}: {}", path.display(), err);
            }
        }
    } else {
        eprintln!("Usage: aoc2020-day03 path/to/input [<x1> <y1> [<x2> <y2> [ ... ]]]");
        std::process::exit(1);
    }
}

fn parse_args() -> Option<(PathBuf, Vec<(usize, usize)>)> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next()?.into();
    let mut slopes = Vec::new();
    while let Some(slope_x) = args.next() {
        let slope_x = slope_x.to_str()?.parse::<usize>().ok()?;
        let slope_y = args.next()?.to_str()?.parse::<usize>().ok()?;
        slopes.push((slope_x, slope_y));
    }
    if args.next().is_none() {
        Some((path, slopes))
    } else {
        None
    }
}

/// Solve the riddle. Part one is a special case of part two where exactly one slope is provided.
fn solve(path: &Path, slopes: &[(usize, usize)]) -> Result<usize, Box<dyn Error>> {
    let rows: Vec<Row> = read_lines(path)?;

    let mut tree_count_product = 1;

    for (slope_x, slope_y) in slopes {
        let path = std::iter::successors(Some((0, 0)), |(x, y)| {
            if y + slope_y < rows.len() {
                Some((x + slope_x, y + slope_y))
            } else {
                None
            }
        });

        let num_trees: usize = path
            .map(|(x, y)| {
                let row = &rows[y].0;
                match row[x % row.len()] {
                    Cell::Open => 0,
                    Cell::Tree => 1,
                }
            })
            .sum();

        tree_count_product *= num_trees;
    }

    Ok(tree_count_product)
}

/// A single cell of the map
enum Cell {
    Open,
    Tree,
}

/// One row of the map
struct Row(Vec<Cell>);

#[derive(Debug)]
pub struct MalformedCell(char);

impl Error for MalformedCell {}
impl Display for MalformedCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cells must only contain `.` or `#`, but found `{}`",
            self.0
        )
    }
}

impl FromStr for Row {
    type Err = MalformedCell;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Row(s
            .chars()
            .map(|ch| match ch {
                '.' => Ok(Cell::Open),
                '#' => Ok(Cell::Tree),
                _ => Err(MalformedCell(ch)),
            })
            .collect::<Result<Vec<_>, _>>()?))
    }
}
