use std::fmt::{Display, Write};

use rustc_hash::FxHashMap;

use crate::Day;

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let jet_input: Vec<_> = input
        .iter()
        .copied()
        .map(<Jet as TryFrom<u8>>::try_from)
        .filter_map(Result::ok)
        .collect();

    let mut sim = Simulator::new(&SHAPES, &jet_input);

    for _ in 0..2022 {
        sim.rock_fall();
    }

    Ok(sim.cave.rock_height.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let jet_input: Vec<_> = input
        .iter()
        .copied()
        .map(<Jet as TryFrom<u8>>::try_from)
        .filter_map(Result::ok)
        .collect();

    let mut sim = Simulator::new(&SHAPES, &jet_input);

    let mut num_rocks = 0;

    let mut states: FxHashMap<(usize, usize, Vec<u8>), (usize, usize)> = FxHashMap::default();
    let mut heights = Vec::new();

    let output = loop {
        // Detect cycles by finding a recurring state, identified by:
        // 1. The index of the shape to spawn next
        // 2. The index of the next jet
        // 3. The effective portion of the stacked rocks
        let state = (
            sim.current_shape,
            sim.current_jet,
            sim.cave.relevant_top().to_owned(),
        );

        match states.entry(state) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let (prev_height, prev_count) = e.get();

                // We decompose the number of simulated cycles into:
                // 1. An initialization part (start of the simulation until start of the cycle)
                // 2. A certain number of repeated cycles
                // 3. A remainder (which consists of the intial part of a cycle)
                //
                // |iiiii(ccccccc)*ccc
                //    ^      ^       ^
                //    |      |cycle  |partial cycle
                //    |initialization

                let cycle_length = num_rocks - prev_count;
                let num_cycles = (1000000000000 - prev_count) / cycle_length;
                let remainder = 1000000000000 - prev_count - num_cycles * cycle_length;

                // The final height of the tower consists of the total height gain due to the cycles
                let height_per_cycle = sim.cave.rock_height - prev_height;
                // Plus the height gain due to the initialiation and the partial cycle
                let final_height = num_cycles * height_per_cycle + heights[prev_count + remainder];

                break final_height;
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert((sim.cave.rock_height, num_rocks));
            }
        }

        num_rocks += 1;
        heights.push(sim.cave.rock_height);
        sim.rock_fall();
    };

    Ok(output.to_string())
}

struct Simulator<'a> {
    shapes: &'a [Shape],
    jets: &'a [Jet],
    current_shape: usize,
    current_jet: usize,
    cave: Cave,
}

impl<'a> Simulator<'a> {
    fn new(shapes: &'a [Shape], jets: &'a [Jet]) -> Self {
        Self {
            shapes,
            jets,
            current_shape: 0,
            current_jet: 0,
            cave: Cave::new(),
        }
    }

    fn rock_fall(&mut self) {
        let shape = &self.shapes[self.current_shape];
        self.current_shape += 1;
        if self.current_shape == self.shapes.len() {
            self.current_shape = 0;
        }

        let spawnx = 2;
        let spawny = self.cave.rock_height + 3;
        self.cave.ensure_height(spawny + shape.height);
        let mut x = spawnx;
        let mut y = spawny;
        loop {
            // Jet pushing
            let jet = self.jets[self.current_jet];
            self.current_jet += 1;
            if self.current_jet == self.jets.len() {
                self.current_jet = 0;
            }
            match jet {
                Jet::Left => {
                    if x > 0 && !self.cave.collides(shape, x - 1, y) {
                        x -= 1;
                    }
                }
                Jet::Right => {
                    if x + shape.width < Cave::WIDTH && !self.cave.collides(shape, x + 1, y) {
                        x += 1;
                    }
                }
            }
            // Falling down
            if y == 0 || self.cave.collides(shape, x, y - 1) {
                self.cave.draw(shape, x, y);
                break;
            } else {
                y -= 1;
            }
        }
    }
}

struct Cave {
    rows: Vec<u8>,
    rock_height: usize,
}

impl Cave {
    const WIDTH: usize = 7;

    fn new() -> Self {
        Cave {
            rows: vec![],
            rock_height: 0,
        }
    }

    fn collides(&self, shape: &Shape, x: usize, y: usize) -> bool {
        for (dy, row) in shape.rows.iter().enumerate() {
            let pat = row << x;
            if self.rows[y + dy] & pat != 0 {
                return true;
            }
        }
        false
    }

    fn draw(&mut self, shape: &Shape, x: usize, y: usize) {
        for (dy, row) in shape.rows.iter().enumerate() {
            let pat = row << x;
            self.rows[y + dy] |= pat;
        }
        self.rock_height = self.rock_height.max(y + shape.height);
    }

    fn ensure_height(&mut self, new_height: usize) {
        if new_height > self.rows.len() {
            self.rows.resize(new_height, 0);
        }
    }

    fn relevant_top(&self) -> &[u8] {
        // skip empty rows
        let mut top = self.rows.len();
        while top > 0 && self.rows[top - 1] == 0 {
            top -= 1;
        }
        // find effective floor
        let mut pat = 0;
        let mut bottom = top;
        while bottom > 0 && pat != 0b1111111 {
            bottom -= 1;
            pat |= self.rows[bottom];
        }

        &self.rows[bottom..top]
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (y, row) in (self.rows.iter().enumerate()).rev() {
            write!(
                f,
                "{}{:4} ",
                if y == self.rock_height { '^' } else { ' ' },
                y,
            )?;
            for x in 0..Cave::WIDTH {
                let ch = if row & (1 << x) != 0 { '#' } else { '.' };
                f.write_char(ch)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Shape {
    width: usize,
    height: usize,
    rows: &'static [u8],
}

static SHAPES: [Shape; 5] = [
    Shape {
        width: 4,
        height: 1,
        rows: &[0b1111],
    },
    Shape {
        width: 3,
        height: 3,
        rows: &[0b010, 0b111, 0b010],
    },
    Shape {
        width: 3,
        height: 3,
        rows: &[0b111, 0b100, 0b100],
    },
    Shape {
        width: 1,
        height: 4,
        rows: &[0b1, 0b1, 0b1, 0b1],
    },
    Shape {
        width: 2,
        height: 2,
        rows: &[0b11, 0b11],
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

#[test]
fn test_example() {
    let input = include_bytes!("../inputs/day17/example.txt");
    assert_eq!(part1(input).unwrap().as_str(), "3068");
}

crate::test_day!(RUN, "day17", "3179", "1567723342929");
