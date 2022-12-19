#![allow(unused)]

use anyhow::bail;
use ndarray::array;
use nom::{
    bytes::complete::tag,
    character::complete::u16 as parse_u16,
    character::complete::u32 as parse_u32,
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};
use rustc_hash::FxHashMap;

use crate::{
    parsers::{self, newline},
    Day,
};

pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let blueprints = parsers::parse(many0(terminated(parse_blueprint, newline)), input)?;

    println!("{blueprints:?}");

    let mut result = 0;

    for blueprint in blueprints {
        println!("evaluating blueprint {}", blueprint.id);
        let mut memo = MemoState {
            seen: FxHashMap::default(),
            best_so_far: 0,
        };
        let mut hist = Vec::new();
        let res = search(&mut memo, &blueprint, 24, [0; 4], [1, 0, 0, 0], &mut hist);
        println!("eval({}) = {}", blueprint.id, res);
        result += (blueprint.id as u64) * (res as u64);
    }

    // Guessed: 1510, too high

    Ok(result.to_string())
}

struct MemoState {
    seen: FxHashMap<State, u16>,
    best_so_far: u16,
}

fn search(
    memo: &mut MemoState,
    blueprint: &Blueprint,
    remaining_time: u16,
    resources: [u16; 4],
    robots: [u16; 4],
    hist: &mut Vec<Option<Res>>,
) -> u16 {
    let key = (remaining_time, resources, robots);

    if let Some(cached) = memo.seen.get(&key) {
        return *cached;
    }

    if remaining_time == 0 {
        // No time left - count the geodes
        resources[Res::Geode.index()]
    } else {
        // Time left, try stuff
        let new_time = remaining_time - 1;
        let mut new_resources = resources;
        let mut new_robots = robots;

        // each robot procuces its resource
        for (index, count) in robots.into_iter().enumerate() {
            new_resources[index] += count;
        }

        // just wait for accumulation
        hist.push(None);
        let heuristic = extrapolate(blueprint, new_time, new_resources, new_robots);
        let mut best = if heuristic < memo.best_so_far {
            0
        } else {
            search(memo, blueprint, new_time, new_resources, new_robots, hist)
        };
        hist.pop();
        if best > memo.best_so_far {
            memo.best_so_far = best;
        }

        // or choose next robot to build
        for res in Res::ALL {
            if let Some(mut built) = build(resources, blueprint.cost[res.index()]) {
                for (index, count) in robots.into_iter().enumerate() {
                    built[index] += count;
                }
                new_robots[res.index()] += 1;

                let heuristic = extrapolate(blueprint, new_time, built, new_robots);
                if heuristic < memo.best_so_far {
                    continue;
                }
                hist.push(Some(res));
                let by_building = search(memo, blueprint, new_time, built, new_robots, hist);
                hist.pop();
                new_robots[res.index()] -= 1;

                if by_building > best {
                    best = by_building;
                    if best > memo.best_so_far {
                        memo.best_so_far = best;
                    }
                }
            }
        }

        memo.seen.insert(key, best);
        best
    }
}

fn build(mut resources: [u16; 4], cost: [u16; 4]) -> Option<[u16; 4]> {
    for i in 0..4 {
        if resources[i] >= cost[i] {
            resources[i] -= cost[i];
        } else {
            return None;
        }
    }
    Some(resources)
}

fn extrapolate(
    blueprint: &Blueprint,
    mut remaining_time: u16,
    resources: [u16; 4],
    robots: [u16; 4],
) -> u16 {
    // how many geodes can we crack in the best case
    let mut geodes = resources[Res::Geode.index()];
    let mut geode_bots = robots[Res::Geode.index()];
    while remaining_time > 0 {
        geodes += geode_bots;
        geode_bots += 1;
        remaining_time -= 1;
    }
    geodes
}

type State = (u16, [u16; 4], [u16; 4]); // (Time, Resources, Robots)

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    bail!("not implemented")
}

fn parse_blueprint(input: &[u8]) -> IResult<&[u8], Blueprint> {
    // Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 4 ore. Each obsidian robot costs 4 ore and 18 clay. Each geode robot costs 4 ore and 9 obsidian.
    // Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
    map(
        separated_pair(
            preceded(tag("Blueprint "), parse_u32),
            tag(": "),
            tuple((
                delimited(tag("Each ore robot costs "), parse_u16, tag(" ore. ")),
                delimited(tag("Each clay robot costs "), parse_u16, tag(" ore. ")),
                map(
                    tuple((
                        tag("Each obsidian robot costs "),
                        parse_u16,
                        tag(" ore and "),
                        parse_u16,
                        tag(" clay. "),
                    )),
                    |(_, ore, _, clay, _)| (ore, clay),
                ),
                map(
                    tuple((
                        tag("Each geode robot costs "),
                        parse_u16,
                        tag(" ore and "),
                        parse_u16,
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

#[derive(Debug, Clone, Copy)]
enum Res {
    Ore = 0,
    Clay = 1,
    Obsidian = 2,
    Geode = 3,
}

impl Res {
    const ALL: [Res; 4] = [Res::Ore, Res::Clay, Res::Obsidian, Res::Geode];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone)]
struct Blueprint {
    id: usize,
    cost: [[u16; 4]; 4],
}

// crate::test_day!(RUN, "day19", "<solution part1>", "<solution part2>");
