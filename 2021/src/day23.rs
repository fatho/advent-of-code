#![allow(unused_imports)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::io::BufRead;
use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use anyhow::Context;
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut board = parse::<2>(input);
    let least_cost = solve_iter(&mut board);

    Ok(least_cost.to_string())
}

fn solve_iter<const CAVE_HEIGHT: u32>(board: &mut Board<CAVE_HEIGHT>) -> u32 {
    let mut moves: Vec<Move> = Vec::new();
    let mut allmoves = Vec::new();
    board.compute_moves(&mut allmoves);
    let mut choices = vec![0];

    let mut current_total = 0;
    let mut best_so_far = u32::MAX;

    while let Some(last_move) = choices.last().copied() {
        if allmoves.len() == last_move {
            // nothing to do here anymore, undo move
            if let Some(mov) = moves.pop() {
                current_total -= mov.cost;
                board.undo_move(&mov);
            }
            choices.pop();
        } else {
            let next = allmoves.pop().expect("must have move");
            let cost = next.cost;
            // perform move
            current_total += cost;
            board.do_move(&next);

            let estimated_total = current_total + board.estimate_remaining_cost();
            if estimated_total > best_so_far {
                // undo move
                current_total -= cost;
                board.undo_move(&next);
            } else if board.is_done() {
                // found solution
                best_so_far = best_so_far.min(current_total);
                // undo move
                current_total -= cost;
                board.undo_move(&next);
            } else {
                moves.push(next);
                // populate subsequent choices
                let top = allmoves.len();
                choices.push(top);
                board.compute_moves(&mut allmoves);
            }
        }
    }

    best_so_far
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (pos, _) = input
        .iter()
        .copied()
        .enumerate()
        .filter(|(_pos, ch)| *ch == b'\n')
        .nth(2)
        .context("not enough lines")?;
    let mut modified_input = input[0..pos + 1].to_owned();
    let insertion = b"  #D#C#B#A#\n  #D#B#A#C#\n";
    modified_input.extend_from_slice(insertion);
    modified_input.extend_from_slice(&input[pos + 1..]);
    let mut board = parse::<4>(&modified_input);
    let least_cost = solve_iter(&mut board);

    Ok(least_cost.to_string())
}

fn parse<const CAVE_HEIGHT: u32>(input: &[u8]) -> Board<CAVE_HEIGHT> {
    let mut board = Board::new();
    input.iter().fold((0, 0), |(x, y), ch| {
        if *ch == b'\n' {
            (0, y + 1)
        } else {
            if let Some(color) = Color::from_ascii(*ch) {
                board.fields[(x, y)] = Some(Field::Amphipod(color));
                board.amphipods.push((x, y));
            }

            (x + 1, y)
        }
    });
    board
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl Color {
    pub fn from_ascii(ch: u8) -> Option<Color> {
        match ch {
            b'A' => Some(Color::Amber),
            b'B' => Some(Color::Bronze),
            b'C' => Some(Color::Copper),
            b'D' => Some(Color::Desert),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Color::Amber => 'A',
            Color::Bronze => 'B',
            Color::Copper => 'C',
            Color::Desert => 'D',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Amphipod(Color),
    Hallway,
}

impl Field {
    pub fn as_amphipod(self) -> Option<Color> {
        match self {
            Field::Amphipod(c) => Some(c),
            Field::Hallway => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Move {
    /// Index of the moving amphipod
    amphipod: Color,
    from: (u32, u32),
    to: (u32, u32),
    cost: u32,
}

fn manhattan((x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> u32 {
    absdiff(x1, x2) + absdiff(y1, y2)
}

fn absdiff(a: u32, b: u32) -> u32 {
    if a < b {
        b - a
    } else {
        a - b
    }
}

fn cost(color: Color, dist: u32) -> u32 {
    let multiplier = match color {
        Color::Amber => 1,
        Color::Bronze => 10,
        Color::Copper => 100,
        Color::Desert => 1000,
    };
    dist * multiplier
}

#[derive(Debug)]
struct Board<const CAVE_HEIGHT: u32> {
    fields: Map<Option<Field>>,
    amphipods: Vec<(u32, u32)>,
}

impl<const CAVE_HEIGHT: u32> Board<CAVE_HEIGHT> {
    pub const TARGET_ZONES: [(u32, Color); 4] = [
        (3, Color::Amber),
        (5, Color::Bronze),
        (7, Color::Copper),
        (9, Color::Desert),
    ];

    pub fn new() -> Self {
        let mut map = Map::new(13, 3 + CAVE_HEIGHT, None);
        for i in 1..=11 {
            map[(i, 1)] = Some(Field::Hallway);
        }
        Self {
            fields: map,
            amphipods: vec![],
        }
    }

    pub fn compute_moves(&self, moves: &mut Vec<Move>) {
        for (x, y) in self.amphipods.iter().copied() {
            let color = self.fields[(x, y)].unwrap().as_amphipod().unwrap();
            // Find where it belongs
            let zone_x = match color {
                Color::Amber => 3,
                Color::Bronze => 5,
                Color::Copper => 7,
                Color::Desert => 9,
            };
            // Can we move to the target cave?
            let target_pos_free = if self.path_to_cave_free(x, zone_x) {
                // Check if we can enter the target cave
                let target_y = (2..2 + CAVE_HEIGHT)
                    .rev()
                    .find(|cy| matches!(self.fields[(zone_x, *cy)], Some(Field::Hallway)));
                target_y.and_then(|target_y| {
                    // Check if all others have the same color
                    let same_color = (target_y + 1..2 + CAVE_HEIGHT)
                        .all(|cy| self.fields[(zone_x, cy)] == Some(Field::Amphipod(color)));
                    if same_color {
                        // we can move in
                        Some((zone_x, target_y))
                    } else {
                        None
                    }
                })
            } else {
                None
            };
            if y == 1 {
                if let Some(target) = target_pos_free {
                    // when in the hallway, always make the move
                    moves.push(Move {
                        from: (x, y),
                        to: target,
                        amphipod: color,
                        cost: cost(color, manhattan((x, y), target)),
                    });
                }
            } else {
                // In a cave

                // Sanity check: exit of a cave must always be free
                assert!(matches!(self.fields[(x, 1)], Some(Field::Hallway)));

                let can_leave =
                    (2..y).all(|cy| matches!(self.fields[(x, cy)], Some(Field::Hallway)));
                if !can_leave {
                    continue;
                }
                // if already in the right room, don't leave unless necessary
                if x == zone_x {
                    let must_make_room = (y + 1..2 + CAVE_HEIGHT).any(|cy| {
                        if let Some(Field::Amphipod(other)) = self.fields[(x, cy)] {
                            other != color
                        } else {
                            unreachable!("caves must be filled from the bottom up")
                        }
                    });
                    if !must_make_room {
                        continue;
                    }
                }

                // if direct path to target is available, always use that
                if let Some(target) = target_pos_free {
                    moves.push(Move {
                        from: (x, y),
                        to: target,
                        amphipod: color,
                        cost: cost(color, absdiff(target.0, x) + (y - 1) + (target.1 - 1)),
                    });
                    continue;
                }

                // Check where we can go:
                for tx in x + 1..=11 {
                    if matches!(self.fields[(tx, 1)], Some(Field::Hallway)) {
                        if !matches!(tx, 3 | 5 | 7 | 9) {
                            moves.push(Move {
                                amphipod: color,
                                from: (x, y),
                                to: (tx, 1),
                                cost: cost(color, manhattan((x, y), (tx, 1))),
                            })
                        }
                    } else {
                        break;
                    }
                }
                for tx in (1..=x - 1).rev() {
                    if matches!(self.fields[(tx, 1)], Some(Field::Hallway)) {
                        if !matches!(tx, 3 | 5 | 7 | 9) {
                            moves.push(Move {
                                amphipod: color,
                                from: (x, y),
                                to: (tx, 1),
                                cost: cost(color, manhattan((x, y), (tx, 1))),
                            })
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn path_to_cave_free(&self, from_x: u32, cave_x: u32) -> bool {
        if from_x > cave_x {
            for x in (cave_x..from_x).rev() {
                if !matches!(self.fields[(x, 1)], Some(Field::Hallway)) {
                    return false;
                }
            }
        } else {
            for x in from_x + 1..=cave_x {
                if !matches!(self.fields[(x, 1)], Some(Field::Hallway)) {
                    return false;
                }
            }
        }

        true
    }

    pub fn do_move(&mut self, mov: &Move) {
        assert_eq!(self.fields[mov.from], Some(Field::Amphipod(mov.amphipod)));
        self.fields[mov.from] = Some(Field::Hallway);
        self.fields[mov.to] = Some(Field::Amphipod(mov.amphipod));
        self.amphipods.retain(|pos| *pos != mov.from);
        self.amphipods.push(mov.to);
    }

    pub fn undo_move(&mut self, mov: &Move) {
        assert_eq!(self.fields[mov.to], Some(Field::Amphipod(mov.amphipod)));
        self.fields[mov.from] = Some(Field::Amphipod(mov.amphipod));
        self.fields[mov.to] = Some(Field::Hallway);
        self.amphipods.retain(|pos| *pos != mov.to);
        self.amphipods.push(mov.from);
    }

    pub fn is_done(&self) -> bool {
        for (zone_x, color) in Self::TARGET_ZONES {
            for y in 2..2 + CAVE_HEIGHT {
                if self.fields[(zone_x, y)] != Some(Field::Amphipod(color)) {
                    return false;
                }
            }
        }
        true
    }

    pub fn estimate_remaining_cost(&self) -> u32 {
        // If we disregard collisions, how many steps do we require at least for
        // reaching the target configuration?
        let mut estimate = 0;

        for (x, y) in self.amphipods.iter().copied() {
            let color = self.fields[(x, y)].unwrap().as_amphipod().unwrap();
            // Find where it belongs
            let zone_x = match color {
                Color::Amber => 3,
                Color::Bronze => 5,
                Color::Copper => 7,
                Color::Desert => 9,
            };
            if zone_x != x {
                estimate += cost(color, absdiff(zone_x, x) + (y - 1) + 1);
            }
        }
        estimate
    }
}

impl<const CAVE_HEIGHT: u32> Display for Board<CAVE_HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.fields.height {
            for x in 0..self.fields.width {
                write!(
                    f,
                    "{}",
                    match self.fields[(x, y)] {
                        Some(Field::Amphipod(color)) => color.to_char(),
                        Some(Field::Hallway) => '.',
                        None => '#',
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
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
}

impl<T> Index<(u32, u32)> for Map<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1 * self.width) as usize]
    }
}

impl<T> IndexMut<(u32, u32)> for Map<T> {
    #[inline]
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1 * self.width) as usize]
    }
}

crate::test_day!(crate::day23::RUN, "day23", "14460", "41366");
