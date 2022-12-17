use std::{cmp::Reverse, fmt::Write, str::Utf8Error};

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::map_res,
    multi::{many1, separated_list1},
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};
use rustc_hash::FxHashMap;

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

    // let best = search_dp_stack(&valves, start, 30);
    Ok(best.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let (valves, start) = compile_network(input)?;

    // let (start_dp, functioning) = search_dp(&valves, start, 26);

    // // Split is symmetric, so we can skip half of them
    // let mut best = 0;
    // for split in 0..(1 << (functioning - 1)) {
    //     let other = (1 << functioning) - 1 - split;

    //     let best_me = start_dp[split];
    //     let best_ele = start_dp[other];

    //     let sum = best_me + best_ele;

    //     if sum > best {
    //         best = sum;
    //     }
    // }

    let (dp, functioning) = simple_dp(&valves, 26);

    // Split is symmetric, so we can skip half of them
    let mut best = 0;
    for split in 0..(1 << (functioning - 1)) {
        let other = (1 << functioning) - 1 - split;

        let best_me = dp[(start, split)];
        let best_ele = dp[(start, other)];

        let sum = best_me + best_ele;

        if sum > best {
            best = sum;
        }
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

fn simple_dp(valves: &[Valve], max_time: usize) -> (ndarray::Array2<u32>, usize) {
    let functioning_valves = valves.iter().take_while(|v| v.flow > 0).count();

    // dp[(time])[set of valves to operate][start]
    //  = maximum relief achievable with given time starting at start
    //  = max { relief by moving, relief by opening }

    // NOTE: The time dimension is implicit by performing DP on windows of size 2 through time.

    // This order of dimensions is almost 30% faster than the reverse layout
    let mut dp = ndarray::Array2::<u32>::zeros((valves.len(), 1 << functioning_valves));
    let mut prev = dp.clone();

    for remaining_time in 1..max_time {
        for pos in 0..valves.len() {
            let bit = 1 << pos;
            for valve_set in 0..(1 << functioning_valves) {
                // opening valve: position remains the same, set of valves shrinks
                let by_opening = if valve_set & bit != 0 {
                    prev[(pos, valve_set & !bit)] + valves[pos].flow * remaining_time as u32
                } else {
                    0
                };

                // moving: sets of valves remains, position changes
                dp[(pos, valve_set)] = valves[pos]
                    .neighbors
                    .iter()
                    .map(|idx| prev[(*idx, valve_set)])
                    .fold(by_opening, Ord::max);
            }
        }

        std::mem::swap(&mut prev, &mut dp);
    }

    (prev, functioning_valves)
}

fn search_permutations(
    perm: &mut [usize],
    valves: &[Valve],
    dist: &ndarray::Array2<u32>,
    start: usize,
    max_time: u32,
) -> u32 {
    let total_flow = perm.iter().map(|index| valves[*index].flow).sum::<u32>();
    // Sorting provides a minor speedup by exploring more reasonable paths first - but doesn't
    // influence correctness
    perm.sort_unstable_by_key(|p| dist[(start, *p)]);

    let mut taken = 0;
    let mut flow = 0;
    let mut relief = 0;

    let mut todo = vec![State {
        pos: start as u32,
        time: 0,
        choice: 0,
    }];

    let mut best_relief = 0;

    while let Some(mut cur) = todo.pop() {
        if taken + cur.choice < perm.len() {
            // remember choice
            let next = cur.choice;
            cur.choice += 1;
            todo.push(cur);

            // Go there and turn valve on
            perm.swap(taken, taken + next);

            let next_pos = perm[taken];
            let steps = dist[(cur.pos as usize, next_pos)];

            // assuming we'd open all remaining valves instantaneously, could we still improve the
            // solution?
            let next_time = cur.time + steps + 1;
            let hypothetical_relief =
                relief + (next_time - cur.time) * flow + (max_time - next_time) * total_flow;

            if cur.time + steps + 1 > max_time || hypothetical_relief <= best_relief {
                // unreachable
                let final_relief = relief + (max_time - cur.time) * flow;

                if final_relief > best_relief {
                    best_relief = final_relief;
                }

                // undo
                perm.swap(taken, taken + next);
            } else {
                relief += flow * (steps + 1);
                flow += valves[next_pos].flow;
                todo.push(State {
                    pos: next_pos as u32,
                    time: cur.time + steps + 1,
                    choice: 0,
                });
                taken += 1;
            }
        } else {
            // done exploring this branch
            let final_relief = relief + (max_time - cur.time) * flow;

            if final_relief > best_relief {
                best_relief = final_relief;
            }

            if let Some(last) = todo.last() {
                // unless popping off last element:
                taken -= 1;
                let unswap = last.choice - 1;
                perm.swap(taken, taken + unswap);

                let pos_flow = valves[cur.pos as usize].flow;
                flow -= pos_flow;
                let rev_steps = cur.time - last.time;
                relief -= flow * rev_steps;
            }
        }
    }

    best_relief
}

#[derive(Clone, Copy)]
struct State {
    pos: u32,
    time: u32,
    choice: usize,
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

#[allow(unused)]
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

#[derive(Debug)]
struct Valve {
    id: usize,
    flow: u32,
    neighbors: Vec<usize>,
}

crate::test_day!(RUN, "day16", "2330", "2675");
