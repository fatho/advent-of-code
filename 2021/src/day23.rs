#![allow(unused_imports)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::take_while;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::terminated;
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let mut board = parse(input);
    println!("{}", board);

    // for m in board.moves() {
    //     println!("{:?}", m);
    // }

    // panic!();

    // let mut least_cost = u32::MAX;
    // let mut moves = Vec::new();
    // let mut least_moves = Vec::new();
    // solve(&mut board, 0, &mut moves, &mut |moves, total_cost| {
    //     least_cost = total_cost.min(least_cost);
    //     least_moves = moves.to_owned();
    // });

    // for lm in least_moves {
    //     println!("{:?}", lm);
    // }

    // println!("{}", solve_iter(&mut board));

    let least_cost = solve_iter(&mut board);

    Ok(least_cost.to_string())
}

fn solve_iter(board: &mut Board) -> u32 {
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
                current_total -= mov.cost();
                board.undo_move(&mov);
            }
            choices.pop();
        } else {
            let next = allmoves.pop().expect("must have move");
            let cost = next.cost();
            if current_total + cost > best_so_far {
                continue;
            }
            // perform move
            current_total += cost;
            board.do_move(&next);
            moves.push(next);

            if board.is_done() {
                // found solution
                best_so_far = best_so_far.min(current_total);
                // undo move
                current_total -= cost;
                board.undo_move(&next);
                moves.pop();
            } else {
                // populate subsequent choices
                let top = allmoves.len();
                choices.push(top);
                board.compute_moves(&mut allmoves);
            }
        }
    }

    best_so_far
}

fn sort_moves(moves: &mut [Move]) {
    moves.sort_by_key(|m| Reverse(m.cost()))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let board = parse(input);
    println!("{:?}", board);

    todo!()
}

fn parse(input: &[u8]) -> Board {
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

#[derive(Debug, Clone, Copy)]
struct Move {
    /// Index of the moving amphipod
    amphipod: Color,
    from: (u32, u32),
    to: (u32, u32),
}

impl Move {
    pub fn cost(&self) -> u32 {
        let dist = manhattan(self.from, self.to);
        let multiplier = match self.amphipod {
            Color::Amber => 1,
            Color::Bronze => 10,
            Color::Copper => 100,
            Color::Desert => 1000,
        };
        dist * multiplier
    }
}

fn manhattan((x1, y1): (u32, u32), (x2, y2): (u32, u32)) -> u32 {
    fn absdiff(a: u32, b: u32) -> u32 {
        if a < b {
            b - a
        } else {
            a - b
        }
    }
    absdiff(x1, x2) + absdiff(y1, y2)
}

#[derive(Debug)]
struct Board {
    fields: Map<Option<Field>>,
    amphipods: Vec<(u32, u32)>,
}

impl Board {
    pub const TARGET_ZONES: [(u32, Color); 4] = [
        (3, Color::Amber),
        (5, Color::Bronze),
        (7, Color::Copper),
        (9, Color::Desert),
    ];

    pub fn new() -> Self {
        let mut map = Map::new(13, 5, None);
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
            if let Some(Field::Amphipod(color)) = self.fields[(x, y)] {
                // Find where it belongs
                let zone_x = match color {
                    Color::Amber => 3,
                    Color::Bronze => 5,
                    Color::Copper => 7,
                    Color::Desert => 9,
                };
                if y == 1 {
                    // In hallway
                    match (self.fields[(zone_x, 2)], self.fields[(zone_x, 3)]) {
                        // Full
                        (Some(Field::Amphipod(_)), Some(Field::Amphipod(_))) => {}
                        // Half full
                        (Some(Field::Hallway), Some(Field::Amphipod(other))) => {
                            if other == color && self.path_to_cave_free(x, zone_x) {
                                // other has same color, so we can move in
                                moves.push(Move {
                                    amphipod: color,
                                    from: (x, y),
                                    to: (zone_x, 2),
                                })
                            }
                        }
                        // Empty
                        (Some(Field::Hallway), Some(Field::Hallway)) => {
                            if self.path_to_cave_free(x, zone_x) {
                                moves.push(Move {
                                    amphipod: color,
                                    from: (x, y),
                                    to: (zone_x, 3),
                                })
                            }
                        }
                        // Invalid states
                        (t1, t2) => unreachable!(
                            "should never have target zone configuration {:?} {:?}",
                            t1, t2
                        ),
                    }
                } else {
                    // In a cave

                    // Sanity check: exit of a cave must always be free
                    debug_assert!(matches!(self.fields[(x, 1)], Some(Field::Hallway)));

                    // cannot leave if there is one before
                    if y == 3 && !matches!(self.fields[(x, 2)], Some(Field::Hallway)) {
                        continue;
                    }
                    // if already in the right room, don't leave unless necessary
                    if x == zone_x {
                        if y == 3 {
                            continue;
                        } else if let Some(Field::Amphipod(other)) = self.fields[(x, 3)] {
                            if other == color {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    // Check where we can go:
                    for tx in x + 1..=11 {
                        if matches!(self.fields[(tx, 1)], Some(Field::Hallway)) {
                            if !matches!(tx, 3 | 5 | 7 | 9) {
                                moves.push(Move {
                                    amphipod: color,
                                    from: (x, y),
                                    to: (tx, 1),
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
                                })
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn path_to_cave_free(&self, from_x: u32, cave_x: u32) -> bool {
        // source must be an amphipod
        debug_assert!(matches!(self.fields[(from_x, 1)], Some(Field::Amphipod(_))));

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
        debug_assert_eq!(self.fields[mov.from], Some(Field::Amphipod(mov.amphipod)));
        self.fields[mov.from] = Some(Field::Hallway);
        self.fields[mov.to] = Some(Field::Amphipod(mov.amphipod));
        self.amphipods.retain(|pos| *pos != mov.from);
        self.amphipods.push(mov.to);
    }

    pub fn undo_move(&mut self, mov: &Move) {
        debug_assert_eq!(self.fields[mov.to], Some(Field::Amphipod(mov.amphipod)));
        self.fields[mov.from] = Some(Field::Amphipod(mov.amphipod));
        self.fields[mov.to] = Some(Field::Hallway);
        self.amphipods.retain(|pos| *pos != mov.to);
        self.amphipods.push(mov.from);
    }

    pub fn is_done(&self) -> bool {
        for (zone_x, color) in Self::TARGET_ZONES {
            for y in 2..=3 {
                if self.fields[(zone_x, y)] != Some(Field::Amphipod(color)) {
                    return false;
                }
            }
        }
        true
    }
}

impl Display for Board {
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
            write!(f, "\n")?;
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

crate::test_day!(crate::day23::RUN, "day23", "14460", "not solved");
