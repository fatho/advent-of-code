#![allow(unused_imports)]

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(p_map, input)?;

    let mut total_risk = 0;
    for y in 0..map.height {
        for x in 0..map.width {
            let height = map.get(x, y);
            if map.neighbours(x, y).all(|h| h > height) {
                total_risk += 1 + height as u32;
            }
        }
    }

    Ok(format!("{}", total_risk))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(p_map, input)?;

    // find low points as starting locations
    let mut low_points = Vec::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let height = map.get(x, y);
            if map.neighbours(x, y).all(|h| h > height) {
                low_points.push((x, y));
            }
        }
    }

    // flood-fill each low point
    let mut flooded = Map::new(map.width, map.height, false);
    let mut basin_sizes = Vec::new();
    for (lx, ly) in low_points.iter().copied() {
        let mut flood_queue = vec![(lx, ly)];
        let mut basin_size = 1;
        flooded.set(lx, ly, true);
        while let Some((x, y)) = flood_queue.pop() {
            for ((nx, ny), _) in map
                .neighbours_with_index(x, y)
                .filter(|(_, height)| *height < 9)
            {
                if !flooded.get(nx, ny) {
                    flooded.set(nx, ny, true);
                    basin_size += 1;
                    flood_queue.push((nx, ny))
                }
            }
        }
        basin_sizes.push(basin_size);
    }

    let (top3, _, _) = basin_sizes.select_nth_unstable_by(3, |b1, b2| b2.cmp(b1));
    let result: u32 = top3.iter().product();

    Ok(format!("{}", result))
}

fn p_map(input: &[u8]) -> IResult<&[u8], Map<u8>> {
    flat_map(
        terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
        |first_line| {
            let width = first_line.len();
            fold_many0(
                map_opt(
                    terminated(take_while(|c| matches!(c, b'0'..=b'9')), parsers::newline),
                    move |line| {
                        if line.len() == width {
                            Some(line)
                        } else {
                            None
                        }
                    },
                ),
                move || Map {
                    data: first_line.iter().map(|c| c - b'0').collect(),
                    width: width as u32,
                    height: 1,
                },
                |mut map, line| {
                    map.height += 1;
                    map.data.extend(line.iter().map(|c| c - b'0'));
                    map
                },
            )
        },
    )(input)
}

#[derive(Debug)]
struct Map<T> {
    // TODO: try z-order curve layout rather than row major for better cache
    // locality
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Map<T>
where
    T: Copy,
{
    pub fn new(width: u32, height: u32, default: T) -> Self {
        Self {
            data: vec![default; width as usize * height as usize],
            width,
            height,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> T {
        self.data[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        self.data[(y * self.width + x) as usize] = value;
    }

    pub fn neighbours(&self, x: u32, y: u32) -> impl Iterator<Item = T> + '_ {
        self.neighbours_with_index(x, y).map(|(_, val)| val)
    }

    pub fn neighbours_with_index(
        &self,
        x: u32,
        y: u32,
    ) -> impl Iterator<Item = ((u32, u32), T)> + '_ {
        // TODO: find something nicer than relying on wrapping?
        let positions = [
            (x, y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y + 1),
        ];
        positions
            .into_iter()
            .filter(|(nx, ny)| (0..self.width).contains(nx) && (0..self.height).contains(ny))
            .map(|(nx, ny)| ((nx, ny), self.get(nx, ny)))
    }
}

crate::test_day!(crate::day9::RUN, "day9", "500", "970200");
