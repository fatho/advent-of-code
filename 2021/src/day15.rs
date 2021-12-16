#![allow(unused_imports)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map, map_opt};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(p_map, input)?;
    let shortest = dijkstra(&map, (0, 0), (map.width - 1, map.height - 1));
    Ok(format!("{}", shortest))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let map = parsers::parse(p_map, input)?;

    let mut extended = Map::new(map.width * 5, map.height * 5, 0);
    for yi in 0..5 {
        for xi in 0..5 {
            for y in 0..map.height {
                for x in 0..map.width {
                    const WRAP: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8];
                    let output = WRAP[(map[(x, y)] as u32 + xi + yi) as usize];
                    extended[(x + xi * map.width, y + yi * map.height)] = output;
                }
            }
        }
    }

    let shortest = dijkstra(&extended, (0, 0), (extended.width - 1, extended.height - 1));
    Ok(format!("{}", shortest))
}

// TODO: use A* with manhattan distance for extra performance here
fn dijkstra(map: &Map<u8>, start: (u32, u32), end: (u32, u32)) -> u32 {
    let mut queue = BinaryHeap::new();
    let mut visited = Map::new(map.width, map.height, false);
    queue.push((Reverse(0), start));

    while let Some((Reverse(risk_so_far), point)) = queue.pop() {
        if visited[point] {
            continue;
        }
        visited[point] = true;
        if point == end {
            return risk_so_far;
        }
        for (neighbour, risk) in map.neighbours_with_index(point.0, point.1) {
            if visited[neighbour] {
                // we came here before via a different (and necessarily shorter) path
                continue;
            }
            let new_total_risk = risk_so_far + risk as u32;
            queue.push((Reverse(new_total_risk), neighbour));
        }
    }
    u32::MAX
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
    // Presumably no need to bother with Z-order curve here since whole data
    // (100 bytes) fits into two cache lines already (typically 128 bytes).
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T> Map<T>
where
    T: Copy,
{
    pub fn new(width: u32, height: u32, value: T) -> Self {
        Self {
            data: vec![value; width as usize * height as usize],
            width,
            height,
        }
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
            .map(|point| (point, self[point]))
    }
}

impl<T> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1 * self.width) as usize]
    }
}

impl<T> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1 * self.width) as usize]
    }
}

crate::test_day!(crate::day15::RUN, "day15", "592", "2897");
