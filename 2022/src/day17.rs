#![allow(unused)]

use std::fmt::Display;

use anyhow::bail;

use crate::{parsers, Day};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let jet_input: Vec<_> = input
        .iter()
        .copied()
        .map(<Jet as TryFrom<u8>>::try_from)
        .filter_map(Result::ok)
        .collect();

    let mut jet_stream = Cycle::new(&jet_input).copied();
    let rocks = Cycle::new(&SHAPES);

    let mut cave = Cave::new();

    for shape in rocks.take(2022) {
        let spawnx = 2;
        let spawny = cave.rock_height + 3;
        cave.ensure_height(spawny + shape.height);
        let sh = std::str::from_utf8(shape.data).unwrap();
        assert!(
            !cave.collides(shape, spawnx, spawny),
            "{spawnx} {spawny} {sh}\n{cave}"
        );

        let mut x = spawnx;
        let mut y = spawny;
        loop {
            // Jet pushing
            match jet_stream.next().expect("infinite stream") {
                Jet::Left => {
                    if x > 0 && !cave.collides(shape, x - 1, y) {
                        x -= 1;
                    }
                }
                Jet::Right => {
                    if x + shape.width < Cave::WIDTH && !cave.collides(shape, x + 1, y) {
                        x += 1;
                    }
                }
            }
            // Falling down
            if y == 0 || cave.collides(shape, x, y - 1) {
                cave.draw(shape, x, y);
                break;
            } else {
                y -= 1;
            }
        }
    }

    Ok(cave.rock_height.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    bail!("not implemented")
}

struct Cave {
    data: Vec<u8>,
    height: usize,
    rock_height: usize,
}

impl Cave {
    const WIDTH: usize = 7;

    fn new() -> Self {
        Cave {
            data: vec![],
            height: 0,
            rock_height: 0,
        }
    }

    fn collides(&self, shape: &Shape, x: usize, y: usize) -> bool {
        for dy in 0..shape.height {
            for dx in 0..shape.width {
                let si = dy * shape.width + dx;
                let ci = (y + dy) * Cave::WIDTH + (x + dx);

                if self.data[ci] == b'#' && shape.data[si] == b'#' {
                    return true;
                }
            }
        }
        false
    }

    fn draw(&mut self, shape: &Shape, x: usize, y: usize) {
        for dy in 0..shape.height {
            for dx in 0..shape.width {
                let si = dy * shape.width + dx;
                let ci = (y + dy) * Cave::WIDTH + (x + dx);

                if shape.data[si] == b'#' {
                    self.data[ci] = b'#';
                }
            }
        }
        self.rock_height = self.rock_height.max(y + shape.height);
    }

    fn ensure_height(&mut self, new_height: usize) {
        if new_height > self.height {
            self.data.resize(new_height * Self::WIDTH, b'.');
            self.height = new_height;
        }
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..self.height).rev() {
            writeln!(
                f,
                "{}{:4} {}",
                if y == self.rock_height { '^' } else { ' ' },
                y,
                std::str::from_utf8(&self.data[y * Self::WIDTH..(y + 1) * Self::WIDTH]).unwrap()
            )?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Shape {
    width: usize,
    height: usize,
    data: &'static [u8],
}

static SHAPES: [Shape; 5] = [
    Shape {
        width: 4,
        height: 1,
        data: b"####",
    },
    Shape {
        width: 3,
        height: 3,
        data: b".#.###.#.",
    },
    Shape {
        width: 3,
        height: 3,
        data: b"###..#..#",
    },
    Shape {
        width: 1,
        height: 4,
        data: b"####",
    },
    Shape {
        width: 2,
        height: 2,
        data: b"####",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Jet {
    Left,
    Right,
}

impl TryFrom<u8> for Jet {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'<' => Ok(Jet::Left),
            b'>' => Ok(Jet::Right),
            _ => Err(()),
        }
    }
}

struct Cycle<'a, T> {
    index: usize,
    data: &'a [T],
}

impl<'a, T> Cycle<'a, T> {
    fn new(data: &'a [T]) -> Self {
        Self { data, index: 0 }
    }
}

impl<'a, T> Iterator for Cycle<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = &self.data[self.index];
        self.index += 1;
        if self.index == self.data.len() {
            self.index = 0;
        }
        Some(result)
    }
}

#[test]
fn test_example() {
    let input = include_bytes!("../inputs/day17/example.txt");
    assert_eq!(part1(input).unwrap().as_str(), "3068");
}

// crate::test_day!(RUN, "day17", "<solution part1>", "<solution part2>");
