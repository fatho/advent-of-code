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
    let (valves, start) = compile_network(input)?;

    let dist = floyd_warshall(&valves);
    let functioning_valves = valves.iter().take_while(|v| v.flow > 0).count();

    let mut perm: Vec<_> = (0..functioning_valves).collect();

    let best = search_permutations(&mut perm, &valves, &dist, start, 30);
    Ok(best.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (valves, start) = compile_network(input)?;

    let dist = floyd_warshall(&valves);
    let functioning_valves = valves.iter().take_while(|v| v.flow > 0).count();

    let mut me_perm = Vec::new();
    let mut ele_perm = Vec::new();
    let mut best = 0;
    // Split is symmetric, so we can skip half of them
    for split in 0..(1 << (functioning_valves - 1)) {
        for v in 0..functioning_valves {
            if split & (1 << v) != 0 {
                me_perm.push(v);
            } else {
                ele_perm.push(v);
            }
        }

        let best_me = search_permutations(&mut me_perm, &valves, &dist, start, 26);
        let best_ele = search_permutations(&mut ele_perm, &valves, &dist, start, 26);

        let sum = best_me + best_ele;

        if sum > best {
            best = sum;
        }

        me_perm.clear();
        ele_perm.clear();
    }

    Ok(best.to_string())
}

fn compile_network(input: &[u8]) -> anyhow::Result<(Vec<Valve>, usize)> {
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

    Ok((valves, id_to_index["AA"]))
}

fn search_permutations(
    perm: &mut [usize],
    valves: &[Valve],
    dist: &ndarray::Array2<u32>,
    start: usize,
    max_time: u32,
) -> u32 {
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

// crate::test_day!(RUN, "day16", "2330", "2675");
