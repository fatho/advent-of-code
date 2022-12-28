#![allow(unused)]

use anyhow::bail;
use ndarray::array;
use nom::{
    bytes::complete::tag,
    character::complete::u32 as parse_u32,
    character::complete::u8 as parse_u8,
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let blueprints = parsers::parse(many0(terminated(parse_blueprint, newline)), input)?;

    let mut result = 0;

    for blueprint in blueprints {
        let res = search_iter(&blueprint, 24, [0; 4], [1, 0, 0, 0]);
        result += (blueprint.id as u64) * (res as u64);
    }

    Ok(result.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let mut blueprints = parsers::parse(many0(terminated(parse_blueprint, newline)), input)?;
    blueprints.truncate(3);

    let mut result = 1;

    for blueprint in blueprints {
        let res = search_iter(&blueprint, 32, [0; 4], [1, 0, 0, 0]);
        result *= (res as u64);
    }

    Ok(result.to_string())
}

fn print_trace(blueprint: &Blueprint, hist: &[Option<Res>]) {
    let mut robots = [1, 0, 0, 0];
    let mut resources = [0, 0, 0, 0];
    for (time, build_op) in hist.iter().enumerate() {
        println!("== Minute {} ==", time + 1);
        let mut new_robots = robots;
        if let Some(op) = build_op {
            println!("Building {} robot", op.name());
            resources = try_build_robot(resources, blueprint.cost[op.index()]).unwrap();
            new_robots[op.index()] += 1;
        }
        for (index, count) in robots.into_iter().enumerate() {
            resources[index] += count;
        }
        println!("Resources: {:?} ", resources);

        robots = new_robots;
        println!("Robots: {:?} ", robots);
    }
}

struct MemoState {
    seen: FxHashMap<State, u8>,
    best_so_far: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    // Fields are ordered from most to least distinctive - this speeds up
    // comparisons, and thus the hash table operations.
    res: [u8; 4],
    bot: [u8; 4],
    time: u8,
}

fn search_iter(
    blueprint: &Blueprint,
    mut remaining_time: u8,
    mut resources: [u8; 4],
    mut robots: [u8; 4],
) -> u8 {
    let mut todo = vec![State {
        time: remaining_time,
        res: resources,
        bot: robots,
    }];

    let max_cost = blueprint.cost.into_iter().fold([0u8; 4], |mut max, cur| {
        max.iter_mut()
            .zip(cur)
            .for_each(|(max, cur)| *max = (*max).max(cur));
        max
    });

    let mut best_so_far = 0;
    let mut seen = FxHashSet::default();

    while let Some(cur) = todo.pop() {
        if cur.time == 1 {
            // For the last step, it doesn't make sense to build anything, since it would only start
            // producing resources when the time is already exhausted.
            best_so_far = best_so_far.max(cur.res[Res::Geode.index()] + cur.bot[Res::Geode.index()])
        } else {
            let heuristic = extrapolate(cur.time, cur.res, cur.bot);

            if heuristic <= best_so_far {
                continue;
            }
            if !seen.insert(cur) {
                continue;
            }

            todo.push(State {
                time: cur.time - 1,
                res: [
                    cur.res[0] + cur.bot[0],
                    cur.res[1] + cur.bot[1],
                    cur.res[2] + cur.bot[2],
                    cur.res[3] + cur.bot[3],
                ],
                ..cur
            });
            // Explore higher-value builds first
            for new_bot in Res::ALL.into_iter().rev() {
                // If we already produce as much of a resource per minute as we can ever
                // consume in the same amount of time, it doesn't make sense to
                // produce even more of it
                if new_bot != Res::Geode && cur.bot[new_bot.index()] >= max_cost[new_bot.index()] {
                    continue;
                }
                if let Some(new_res) = try_build_robot(cur.res, blueprint.cost[new_bot.index()]) {
                    let mut new_bots = cur.bot;
                    new_bots[new_bot.index()] += 1;
                    todo.push(State {
                        time: cur.time - 1,
                        res: [
                            new_res[0] + cur.bot[0],
                            new_res[1] + cur.bot[1],
                            new_res[2] + cur.bot[2],
                            new_res[3] + cur.bot[3],
                        ],
                        bot: new_bots,
                    });
                }
            }
        }
    }

    best_so_far
}

fn try_build_robot(mut resources: [u8; 4], cost: [u8; 4]) -> Option<[u8; 4]> {
    for i in 0..4 {
        if resources[i] >= cost[i] {
            resources[i] -= cost[i];
        } else {
            return None;
        }
    }
    Some(resources)
}

fn extrapolate(remaining_time: u8, resources: [u8; 4], robots: [u8; 4]) -> u8 {
    // how many geodes can we still crack in the best case
    let mut geodes = resources[Res::Geode.index()];
    let mut geode_bots = robots[Res::Geode.index()];
    for _ in 0..remaining_time {
        geodes += geode_bots;
        geode_bots += 1;
    }
    geodes
}

//type State = (u8, [u8; 4], [u8; 4]); // (Time, Resources, Robots)

fn parse_blueprint(input: &[u8]) -> IResult<&[u8], Blueprint> {
    map(
        separated_pair(
            preceded(tag("Blueprint "), parse_u32),
            tag(": "),
            tuple((
                delimited(tag("Each ore robot costs "), parse_u8, tag(" ore. ")),
                delimited(tag("Each clay robot costs "), parse_u8, tag(" ore. ")),
                map(
                    tuple((
                        tag("Each obsidian robot costs "),
                        parse_u8,
                        tag(" ore and "),
                        parse_u8,
                        tag(" clay. "),
                    )),
                    |(_, ore, _, clay, _)| (ore, clay),
                ),
                map(
                    tuple((
                        tag("Each geode robot costs "),
                        parse_u8,
                        tag(" ore and "),
                        parse_u8,
                        tag(" obsidian."),
                    )),
                    |(_, ore, _, obsidian, _)| (ore, obsidian),
                ),
            )),
        ),
        |(id, (ore_ore, clay_ore, (obs_ore, obs_clay), (geo_ore, geo_obs)))| Blueprint {
            id: id as usize,
            cost: [
                [ore_ore, 0, 0, 0],
                [clay_ore, 0, 0, 0],
                [obs_ore, obs_clay, 0, 0],
                [geo_ore, 0, geo_obs, 0],
            ],
        },
    )(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Res {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}

impl Res {
    const ALL: [Res; 4] = [Res::Ore, Res::Clay, Res::Obsidian, Res::Geode];

    const fn index(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        match self {
            Res::Ore => "ore",
            Res::Clay => "clay",
            Res::Obsidian => "obsidian",
            Res::Geode => "geode",
        }
    }
}

#[derive(Debug, Clone)]
struct Blueprint {
    id: usize,
    cost: [[u8; 4]; 4],
}

crate::test_day!(RUN, "day19", "1487", "13440");
