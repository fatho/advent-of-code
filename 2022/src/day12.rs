#![allow(unused)]

use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fmt::{Display, Write},
    ops::{Index, IndexMut},
};

use anyhow::bail;

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

// TODO: factor out common bits

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let input = parse_map(input)?;
    path1(input).map(|len| len.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let input = parse_map(input)?;
    path2(input).map(|len| len.to_string())
}

// TODO: consider A*
fn path1(input: Input) -> anyhow::Result<u32> {
    let mut queue = BinaryHeap::new();
    let mut visited = Map::new(input.map.width, input.map.height, false);
    queue.push(Node {
        dist: Reverse(0),
        pos: input.start,
    });

    fn can_pass(map: &Map<u8>, from: (u32, u32), to: (u32, u32)) -> bool {
        let fromh = map[from];
        let toh = map[to];
        toh <= fromh + 1
    }

    while let Some(node) = queue.pop() {
        if visited[node.pos] {
            continue;
        }
        visited[node.pos] = true;

        if node.pos == input.end {
            return Ok(node.dist.0);
        }

        // queue neighbours
        let (x, y) = node.pos;
        let new_dist = Reverse(node.dist.0 + 1);
        if x > 0 && can_pass(&input.map, (x, y), (x - 1, y)) && !visited[(x - 1, y)] {
            queue.push(Node {
                dist: new_dist,
                pos: (x - 1, y),
            })
        }
        if y > 0 && can_pass(&input.map, (x, y), (x, y - 1)) && !visited[(x, y - 1)] {
            queue.push(Node {
                dist: new_dist,
                pos: (x, y - 1),
            })
        }
        if x < input.map.width - 1
            && can_pass(&input.map, (x, y), (x + 1, y))
            && !visited[(x + 1, y)]
        {
            queue.push(Node {
                dist: new_dist,
                pos: (x + 1, y),
            })
        }
        if y < input.map.height - 1
            && can_pass(&input.map, (x, y), (x, y + 1))
            && !visited[(x, y + 1)]
        {
            queue.push(Node {
                dist: new_dist,
                pos: (x, y + 1),
            })
        }
    }

    bail!("No path")
}

// TODO: consider A*
fn path2(input: Input) -> anyhow::Result<u32> {
    let mut queue = BinaryHeap::new();
    let mut visited = Map::new(input.map.width, input.map.height, false);
    queue.push(Node {
        dist: Reverse(0),
        pos: input.end,
    });

    fn can_pass(map: &Map<u8>, from: (u32, u32), to: (u32, u32)) -> bool {
        let fromh = map[from];
        let toh = map[to];
        toh + 1 >= fromh
    }

    while let Some(node) = queue.pop() {
        if visited[node.pos] {
            continue;
        }
        visited[node.pos] = true;

        if input.map[node.pos] == 0 {
            return Ok(node.dist.0);
        }

        // queue neighbours
        let (x, y) = node.pos;
        let new_dist = Reverse(node.dist.0 + 1);
        if x > 0 && can_pass(&input.map, (x, y), (x - 1, y)) && !visited[(x - 1, y)] {
            queue.push(Node {
                dist: new_dist,
                pos: (x - 1, y),
            })
        }
        if y > 0 && can_pass(&input.map, (x, y), (x, y - 1)) && !visited[(x, y - 1)] {
            queue.push(Node {
                dist: new_dist,
                pos: (x, y - 1),
            })
        }
        if x < input.map.width - 1
            && can_pass(&input.map, (x, y), (x + 1, y))
            && !visited[(x + 1, y)]
        {
            queue.push(Node {
                dist: new_dist,
                pos: (x + 1, y),
            })
        }
        if y < input.map.height - 1
            && can_pass(&input.map, (x, y), (x, y + 1))
            && !visited[(x, y + 1)]
        {
            queue.push(Node {
                dist: new_dist,
                pos: (x, y + 1),
            })
        }
    }

    bail!("No path")
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Node {
    dist: Reverse<u32>,
    pos: (u32, u32),
}

fn parse_map(input: &[u8]) -> anyhow::Result<Input> {
    let mut width = 0;
    let mut x = 0;
    let mut y = 0;

    let mut start = (0, 0);
    let mut end = (0, 0);

    let trees = input
        .iter()
        .copied()
        .filter_map(|ch| match ch {
            b'a'..=b'z' => {
                x += 1;
                Some(Ok(ch - b'a'))
            }
            b'S' => {
                start = (x, y);
                x += 1;
                Some(Ok(0))
            }
            b'E' => {
                end = (x, y);
                x += 1;
                Some(Ok(25))
            }
            b'\n' => {
                y += 1;
                if width == 0 {
                    width = x;
                }

                if width == x {
                    x = 0;
                    None
                } else {
                    Some(Err(anyhow::anyhow!("Row width mismatch")))
                }
            }
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()?;

    // handle optional last \n
    if x > 0 {
        y += 1
    }

    Ok(Input {
        map: Map {
            data: trees,
            width,
            height: y,
        },
        start,
        end,
    })
}

struct Input {
    map: Map<u8>,
    start: (u32, u32),
    end: (u32, u32),
}

struct Map<T> {
    data: Vec<T>,
    width: u32,
    height: u32,
}

impl<T: Clone> Map<T> {
    fn new(width: u32, height: u32, init: T) -> Self {
        Map {
            data: vec![init; width as usize * height as usize],
            width,
            height,
        }
    }

    fn offset(&self, x: u32, y: u32) -> usize {
        (x as usize) + (self.width as usize) * (y as usize)
    }
}

impl<T: Clone> Index<(u32, u32)> for Map<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[self.offset(index.0, index.1)]
    }
}

impl<T: Clone> IndexMut<(u32, u32)> for Map<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        let offset = self.offset(index.0, index.1);
        &mut self.data[offset]
    }
}

impl Display for Map<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.chunks(self.width as usize) {
            for col in row {
                f.write_char((col + b'a') as char)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

crate::test_day!(RUN, "day12", "370", "363");
