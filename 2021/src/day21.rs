#![allow(unused_imports)]

use std::ops::{Index, IndexMut};

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::combinator::{flat_map, map};
use nom::multi::fold_many0;
use nom::sequence::{pair, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (pos1, pos2) = parsers::parse(p_input, input)?;

    let mut positions = [pos1, pos2];
    let mut scores = [0, 0];
    let mut die = DeterministicDie::default();

    let mut player_turn = 0;

    while scores.iter().all(|s| *s < 1000) {
        let roll = die.roll() + die.roll() + die.roll();
        let player_index = player_turn % positions.len();

        positions[player_index] = (positions[player_index] - 1 + roll) % 10 + 1;
        scores[player_index] += positions[player_index];

        player_turn += 1;
    }

    let result = die.rolls * scores.iter().copied().min().unwrap_or(0);

    Ok(result.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (pos1, pos2) = parsers::parse(p_input, input)?;

    let mut state_space = DiracState::new();
    let mut new_state_space = DiracState::new();

    state_space[StateIndex {
        p1_pos: pos1 - 1,
        p2_pos: pos2 - 1,
        p1_score: 0,
        p2_score: 0,
    }] = 1;

    let mut states_left = true;
    let mut player1 = true;

    let mut p1_wins = 0;
    let mut p2_wins = 0;

    while states_left {
        println!("{}", state_space.states.iter().copied().sum::<u64>());
        states_left = false;
        // Advance one step
        for p1_pos in 0..10 {
            for p2_pos in 0..10 {
                for p1_score in 0..=20 {
                    for p2_score in 0..=20 {
                        let cur_index = StateIndex {
                            p1_pos,
                            p2_pos,
                            p1_score,
                            p2_score,
                        };
                        let num_states = state_space[cur_index];
                        if num_states > 0 {
                            states_left = true;
                            for roll1 in 0..3 {
                                for roll2 in 0..3 {
                                    for roll3 in 0..3 {
                                        let total_roll = 3 + roll1 + roll2 + roll3;
                                        if player1 {
                                            let p1_new_pos = (p1_pos + total_roll) % 10;
                                            let p1_new_score = p1_score + p1_new_pos + 1;
                                            if p1_new_score >= 21 {
                                                p1_wins += num_states;
                                            } else {
                                                let next = StateIndex {
                                                    p1_pos: p1_new_pos,
                                                    p2_pos,
                                                    p1_score: p1_new_score,
                                                    p2_score,
                                                };
                                                new_state_space[next] += num_states;
                                            }
                                        } else {
                                            let p2_new_pos = (p2_pos + total_roll) % 10;
                                            let p2_new_score = p2_score + p2_new_pos + 1;
                                            if p2_new_score >= 21 {
                                                p2_wins += num_states;
                                            } else {
                                                let next = StateIndex {
                                                    p1_pos,
                                                    p2_pos: p2_new_pos,
                                                    p1_score,
                                                    p2_score: p2_new_score,
                                                };
                                                new_state_space[next] += num_states;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        std::mem::swap(&mut state_space, &mut new_state_space);
        new_state_space.reset();
        player1 = !player1;
    }

    // for p1_pos in 0..10 {
    //     for p2_pos in 0..10 {
    //         for p1_score in 21..=30 {
    //             for p2_score in 0..20 {
    //                 let cur_index = StateIndex {
    //                     p1_pos,
    //                     p2_pos,
    //                     p1_score,
    //                     p2_score,
    //                 };
    //                 p1_wins += state_space[cur_index];
    //             }
    //         }
    //         for p2_score in 21..=30 {
    //             for p1_score in 0..20 {
    //                 let cur_index = StateIndex {
    //                     p1_pos,
    //                     p2_pos,
    //                     p1_score,
    //                     p2_score,
    //                 };
    //                 p2_wins += state_space[cur_index];
    //             }
    //         }
    //     }
    // }
    println!("{} {}", p1_wins, p2_wins);

    Ok(p1_wins.max(p2_wins).to_string())
}

fn p_input(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    map(
        pair(
            terminated(p_line, parsers::newline),
            terminated(p_line, parsers::newline),
        ),
        |((id1, pos1), (_id2, pos2))| if id1 == 1 { (pos1, pos2) } else { (pos2, pos1) },
    )(input)
}

fn p_line(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
    map(
        tuple((
            tag("Player "),
            numbers::u32,
            tag(" starting position: "),
            numbers::u32,
        )),
        |(_, id, _, pos)| (id, pos),
    )(input)
}

#[derive(Debug, Default)]
struct DeterministicDie {
    rolls: u32,
}

impl DeterministicDie {
    pub fn roll(&mut self) -> u32 {
        let outcome = (self.rolls % 100) + 1;
        self.rolls += 1;
        outcome
    }
}

struct DiracState {
    states: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StateIndex {
    p1_pos: u32,
    p2_pos: u32,
    p1_score: u32,
    p2_score: u32,
}

impl StateIndex {
    pub fn offset(&self) -> usize {
        let offset =
            self.p1_pos + self.p2_pos * 10 + self.p1_score * 10 * 10 + self.p2_score * 10 * 10 * 21;
        offset as usize
    }
}

impl DiracState {
    pub fn new() -> Self {
        Self {
            states: vec![0; 10 * 10 * 21 * 21],
        }
    }

    pub fn reset(&mut self) {
        self.states.iter_mut().for_each(|c| *c = 0);
    }
}

impl Index<StateIndex> for DiracState {
    type Output = u64;

    fn index(&self, index: StateIndex) -> &Self::Output {
        &self.states[index.offset()]
    }
}

impl IndexMut<StateIndex> for DiracState {
    fn index_mut(&mut self, index: StateIndex) -> &mut Self::Output {
        &mut self.states[index.offset()]
    }
}

crate::test_day!(crate::day21::RUN, "day21", "897798", "48868319769358");
