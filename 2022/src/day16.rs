use std::{cmp::Reverse, fmt::Write, str::Utf8Error};

use anyhow::{bail, Context};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{map, map_res, opt},
    multi::{many1, separated_list1},
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let (valves, _, id_to_index) = compile_network(input)?;

    let best = run_permute(&valves, id_to_index["AA"], 30);
    Ok(best.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (valves, _, id_to_index) = compile_network(input)?;

    let dp = run_dp(&valves, id_to_index["AA"], 26);
    let max_valves = dp.shape()[1];

    let mut global_best = None;
    for me_open in 0..max_valves {
        for elephant_open in 0..max_valves {
            // sets of valves must be disjoint
            if me_open & elephant_open != 0 {
                continue;
            }
            for me_pos in 0..valves.len() {
                for elephant_pos in 0..valves.len() {
                    global_best =
                        global_best.max(dp[(26, me_open, me_pos)].and_then(|me| {
                            dp[(26, elephant_open, elephant_pos)].map(|ele| ele + me)
                        }))
                }
            }
        }
    }

    Ok(global_best.context("no solution")?.to_string())
}

fn compile_network(
    input: &[u8],
) -> anyhow::Result<(Vec<Valve>, Vec<&str>, FxHashMap<&str, usize>)> {
    let mut src_valves = parsers::parse(many1(terminated(parse_valve, newline)), input)?;

    // Prepare network by putting functioning valves first
    src_valves.sort_unstable_by_key(|v| Reverse(v.flow));

    let mut index_to_id = Vec::new();
    let mut id_to_index = FxHashMap::default();

    for (index, v) in src_valves.iter().enumerate() {
        index_to_id.push(v.id);
        id_to_index.insert(v.id, index);
    }

    let valves: Vec<_> = src_valves
        .iter()
        .map(|v| Valve {
            id: id_to_index[v.id],
            flow: v.flow,
            neighbors: v.neighbors.iter().map(|n| id_to_index[n]).collect(),
        })
        .collect();

    Ok((valves, index_to_id, id_to_index))
}

fn run_dp(valves: &[Valve], start: usize, max_time: usize) -> ndarray::Array3<Option<u32>> {
    let functioning_valves = valves.iter().take_while(|v| v.flow > 0).count();

    // DP

    // [time][valves][position]

    let max_valves = 1 << functioning_valves;
    let mut dp =
        ndarray::Array3::<Option<u32>>::from_elem((max_time + 1, max_valves, valves.len()), None);

    // Init
    dp[(0, 0, start)] = Some(0);

    for time in 1..=max_time {
        for open in 0..max_valves {
            let current_flow = (0..functioning_valves)
                .map(|vi| {
                    if open & (1 << vi) != 0 {
                        valves[vi].flow
                    } else {
                        0
                    }
                })
                .sum::<u32>();

            for pos in 0..valves.len() {
                let pos_bit = 1 << pos;

                let mut best = None;

                // could've opened
                if open & pos_bit != 0 {
                    best = dp[(time - 1, open & !pos_bit, pos)]
                        .map(|prev| prev + current_flow - valves[pos].flow)
                };

                // could've stayed
                let by_staying = dp[(time - 1, open, pos)].map(|prev| prev + current_flow);
                best = best.max(by_staying);

                // could've moved
                for from in valves[pos].neighbors.iter() {
                    let by_moving = dp[(time - 1, open, *from)].map(|prev| prev + current_flow);
                    best = best.max(by_moving);
                }

                dp[(time, open, pos)] = best;
            }
        }
    }

    dp
}

fn run_permute(valves: &[Valve], start: usize, max_time: u32) -> u32 {
    let dist = floyd_warshall(valves);
    let functioning_valves = valves.iter().take_while(|v| v.flow > 0).count();

    let mut perm: Vec<_> = (0..functioning_valves).collect();
    let mut taken = 0;

    let mut todo = vec![State {
        pos: start,
        flow: 0,
        relief: 0,
        time: 0,
        choice: 0,
        unswap: 0,
    }];

    let mut best_relief = 0;

    while let Some(mut cur) = todo.pop() {
        assert_eq!(taken, todo.len());
        if taken + cur.choice < perm.len() {
            // remember choice
            let next = cur.choice;
            cur.choice += 1;
            todo.push(cur);

            // Go there and turn valve on
            perm.swap(taken, taken + next);

            let next_pos = perm[taken];
            let steps = dist[(cur.pos, next_pos)];

            if cur.time + steps + 1 > max_time {
                // unreachable
                let final_relief = cur.relief + (max_time - cur.time) * cur.flow;

                if final_relief > best_relief {
                    best_relief = final_relief;
                }

                // undo
                perm.swap(taken, taken + next);
            } else {
                todo.push(State {
                    pos: next_pos,
                    flow: cur.flow + valves[next_pos].flow,
                    relief: cur.relief + cur.flow * (steps + 1),
                    time: cur.time + steps + 1,
                    choice: 0,
                    unswap: taken + next,
                });
                taken += 1;
            }
        } else {
            // done exploring this branch
            let final_relief = cur.relief + (max_time - cur.time) * cur.flow;

            if final_relief > best_relief {
                best_relief = final_relief;
            }

            if taken > 0 {
                // unless popping off last element:
                taken -= 1;
                perm.swap(taken, cur.unswap);
            }
        }
    }

    best_relief
}

#[derive(Clone, Copy)]
struct State {
    pos: usize,
    flow: u32,
    relief: u32,
    time: u32,
    choice: usize,
    unswap: usize,
}

/// Computes distances between each pair of vertices
/// Curtesy of https://en.wikipedia.org/wiki/Floyd–Warshall_algorithm
fn floyd_warshall(valves: &[Valve]) -> ndarray::Array2<u32> {
    let mut dist = ndarray::Array2::<u32>::from_elem((valves.len(), valves.len()), u32::MAX);
    for v in valves {
        for n in v.neighbors.iter().copied() {
            assert_ne!(v.id, n);
            dist[(v.id, n)] = 1;
        }
        dist[(v.id, v.id)] = 0;
    }

    for k in 0..valves.len() {
        for i in 0..valves.len() {
            for j in 0..valves.len() {
                let indirect = dist[(i, k)].saturating_add(dist[(k, j)]);
                if dist[(i, j)] > indirect {
                    dist[(i, j)] = indirect;
                }
            }
        }
    }

    dist
}

fn parse_valve(input: &[u8]) -> IResult<&[u8], SrcValve> {
    map_res(
        separated_pair(
            pair(
                preceded(tag("Valve "), take(2usize)),
                preceded(tag(" has flow rate="), nom::character::complete::u32),
            ),
            tag("; "),
            preceded(
                alt((
                    tag("tunnel leads to valve "),
                    tag("tunnels lead to valves "),
                )),
                separated_list1(tag(", "), take(2usize)),
            ),
        ),
        |((id, flow), neighbors)| {
            Ok::<_, Utf8Error>(SrcValve {
                id: std::str::from_utf8(id)?,
                flow,
                neighbors: neighbors
                    .into_iter()
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        },
    )(input)
}

fn to_dot(valves: &[SrcValve]) -> String {
    let mut out = String::new();
    writeln!(&mut out, "graph G {{").unwrap();
    for v in valves {
        writeln!(&mut out, "{} [label=\"{} : {}\"];", v.id, v.id, v.flow).unwrap();
        for n in &v.neighbors {
            if v.id < n {
                writeln!(&mut out, "{} -- {};", v.id, n).unwrap();
            }
        }
    }
    writeln!(&mut out, "}}").unwrap();
    out
}

#[derive(Debug, Clone)]
struct SrcValve<'a> {
    id: &'a str,
    flow: u32,
    neighbors: Vec<&'a str>,
}

struct Valve {
    id: usize,
    flow: u32,
    neighbors: Vec<usize>,
}

// crate::test_day!(RUN, "day16", "2330", "<solution part2>");
