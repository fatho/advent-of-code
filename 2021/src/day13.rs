#![allow(unused_imports)]

use std::collections::HashSet;

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0};
use nom::sequence::{preceded, separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (mut points, folds) = parsers::parse(p_instructions, input)?;

    fold(folds[0], &mut points);

    let distinct = points.into_iter().collect::<HashSet<_>>();

    Ok(format!("{}", distinct.len()))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (mut points, folds) = parsers::parse(p_instructions, input)?;

    for f in folds.iter() {
        // might be able to make things a bit faster by pruning points along the
        // way
        fold(*f, &mut points);
    }

    Ok(render(&points))
}

fn render(points: &[Point]) -> String {
    let lookup = points.iter().copied().collect::<HashSet<_>>();
    let (max_x, max_y) = points
        .iter()
        .fold((0, 0), |(mx, my), p| (mx.max(p.x), my.max(p.y)));
    let mut result = String::new();
    for y in 0..=max_y {
        for x in 0..=max_x {
            if lookup.contains(&Point { x, y }) {
                result.push('#');
            } else {
                result.push('.');
            }
        }
        result.push('\n');
    }
    result
}

fn fold(fold: Fold, points: &mut [Point]) {
    match fold {
        Fold::X(xfold) => {
            for p in points.iter_mut() {
                let fold = (p.x > xfold) as i32;
                p.x -= (fold * (2 * (p.x as i32 - xfold as i32))) as u32;
            }
        }
        Fold::Y(yfold) => {
            for p in points.iter_mut() {
                let fold = (p.y > yfold) as i32;
                p.y -= (fold * (2 * (p.y as i32 - yfold as i32))) as u32;
            }
        }
    }
}

fn p_instructions(input: &[u8]) -> IResult<&[u8], (Vec<Point>, Vec<Fold>)> {
    separated_pair(p_points, parsers::newline, p_folds)(input)
}

fn p_point(input: &[u8]) -> IResult<&[u8], Point> {
    map(
        separated_pair(parsers::u32, tag(","), parsers::u32),
        |(x, y)| Point { x, y },
    )(input)
}

fn p_points(input: &[u8]) -> IResult<&[u8], Vec<Point>> {
    many0(terminated(p_point, parsers::newline))(input)
}

fn p_fold(input: &[u8]) -> IResult<&[u8], Fold> {
    preceded(
        tag("fold along "),
        alt((
            preceded(tag("x="), map(parsers::u32, Fold::X)),
            preceded(tag("y="), map(parsers::u32, Fold::Y)),
        )),
    )(input)
}

fn p_folds(input: &[u8]) -> IResult<&[u8], Vec<Fold>> {
    many0(terminated(p_fold, parsers::newline))(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Fold {
    X(u32),
    Y(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Point {
    x: u32,
    y: u32,
}

crate::test_day!(
    crate::day13::RUN,
    "day13",
    "708",
    r"####.###..#....#..#.###..###..####.#..#
#....#..#.#....#..#.#..#.#..#.#....#..#
###..###..#....#..#.###..#..#.###..####
#....#..#.#....#..#.#..#.###..#....#..#
#....#..#.#....#..#.#..#.#.#..#....#..#
####.###..####..##..###..#..#.#....#..#
"
);
