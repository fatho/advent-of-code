#![allow(unused)]

use anyhow::{bail, Context};
use nom::{
    bytes::complete::tag,
    character::complete::i32 as parse_i32,
    combinator::map,
    multi::many0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input)?;

    let result = count_row(&sensors, 2000000)?;
    Ok(result.to_string())
}

fn count_row(sensors: &[Sensor], row: i32) -> anyhow::Result<usize> {
    // TODO: use trick from part 2 to speed up the counting

    // Find min and max pos to evaluate
    let (min, max) = sensors
        .iter()
        .filter_map(|s| {
            let range = s.beacon_distance();
            let dy = row.abs_diff(s.position.y);
            if dy > range {
                None
            } else {
                let dx = range - dy;
                Some((s.position.x - dx as i32, s.position.x + dx as i32))
            }
        })
        .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
        .context("no sensors in range for this row")?;

    let count = (min..=max)
        .filter(|x| {
            sensors
                .iter()
                .any(|s| !s.maybe_beacon(Pos { x: *x, y: row }))
        })
        .count();
    Ok(count)
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input)?;
    find_beacon(sensors, 4000000).map(|r| r.to_string())
}

fn find_beacon(mut sensors: Vec<Sensor>, max_coord: i32) -> anyhow::Result<u64> {
    sensors.sort_unstable_by_key(|s| s.position.x);

    let mut target = None;
    let ranges = sensors
        .iter()
        .map(|s| s.beacon_distance())
        .collect::<Vec<_>>();

    for y in 0..=max_coord {
        let mut x = 0;
        for (s, range) in sensors.iter().zip(ranges.iter().copied()) {
            let dy = s.position.y.abs_diff(y);
            if dy <= range {
                let dx = s.position.x.abs_diff(x);
                let rx = range - dy;
                if dx <= rx {
                    x = s.position.x + rx as i32 + 1;
                }
            }
        }
        if x <= max_coord {
            target = Some(Pos { x, y });
            break;
        }
    }

    if let Some(target) = target {
        let tuning = target.x as u64 * 4000000 + target.y as u64;
        Ok(tuning)
    } else {
        bail!("beacon not found");
    }
}

fn parse_sensor(input: &[u8]) -> IResult<&[u8], Sensor> {
    map(
        separated_pair(
            preceded(tag("Sensor at "), parse_pos),
            tag(": "),
            preceded(tag("closest beacon is at "), parse_pos),
        ),
        |(position, beacon)| Sensor { position, beacon },
    )(input)
}

fn parse_pos(input: &[u8]) -> IResult<&[u8], Pos> {
    map(
        separated_pair(
            preceded(tag("x="), parse_i32),
            tag(", "),
            preceded(tag("y="), parse_i32),
        ),
        |(x, y)| Pos { x, y },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn manhattan(self, other: Pos) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Sensor {
    position: Pos,
    beacon: Pos,
}

impl Sensor {
    fn beacon_distance(&self) -> u32 {
        self.beacon.manhattan(self.position)
    }

    fn maybe_beacon(&self, pos: Pos) -> bool {
        pos == self.beacon || pos.manhattan(self.position) > self.beacon_distance()
    }
}

#[test]
fn test_example() {
    let input = include_bytes!("../inputs/day15/example.txt");
    let sensors = parsers::parse(many0(terminated(parse_sensor, newline)), input).unwrap();

    let result = count_row(&sensors, 10).unwrap();

    assert_eq!(result, 26);

    let result2 = find_beacon(sensors, 20).unwrap();

    assert_eq!(result2, 56000011)
}

// crate::test_day!(RUN, "day15", "5607466", "<solution part2>");
