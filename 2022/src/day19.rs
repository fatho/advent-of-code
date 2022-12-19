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
            resources = build(resources, blueprint.cost[op.index()]).unwrap();
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
    seen: FxHashMap<State, u16>,
    best_so_far: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    time: u16,
    res: [u16; 4],
    bot: [u16; 4],
}

fn search_iter(
    blueprint: &Blueprint,
    mut remaining_time: u16,
    mut resources: [u16; 4],
    mut robots: [u16; 4],
) -> u16 {
    let mut todo = vec![State {
        time: remaining_time,
        res: resources,
        bot: robots,
    }];

    let mut best_so_far = 0;
    let mut seen = FxHashSet::default();

    while let Some(cur) = todo.pop() {
        if cur.time == 0 {
            best_so_far = best_so_far.max(cur.res[Res::Geode.index()]);
        } else {
            let heuristic = extrapolate(blueprint, cur.time, cur.res, cur.bot);

            if heuristic < best_so_far {
                continue;
            }
            if !seen.insert(cur) {
                continue;
            }

            // explore branches
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
            for new_bot in Res::ALL {
                if let Some(new_res) = build(cur.res, blueprint.cost[new_bot.index()]) {
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

fn search(
    memo: &mut MemoState,
    blueprint: &Blueprint,
    remaining_time: u16,
    resources: [u16; 4],
    robots: [u16; 4],
) -> u16 {
    let key = State {
        time: remaining_time,
        res: resources,
        bot: robots,
    };

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

        // each robot procuces its resource
        for (index, count) in robots.into_iter().enumerate() {
            new_resources[index] += count;
        }

        // just wait for accumulation
        let heuristic = extrapolate(blueprint, new_time, new_resources, robots);
        let mut best = if heuristic < memo.best_so_far {
            0
        } else {
            search(memo, blueprint, new_time, new_resources, robots)
        };
        if best > memo.best_so_far {
            memo.best_so_far = best;
        }

        // or choose next robot to build
        for res in Res::ALL {
            if let Some(mut built) = build(resources, blueprint.cost[res.index()]) {
                for (index, count) in robots.into_iter().enumerate() {
                    built[index] += count;
                }
                let mut new_robots = robots;
                new_robots[res.index()] += 1;

                let heuristic = extrapolate(blueprint, new_time, built, new_robots);
                if heuristic < memo.best_so_far {
                    continue;
                }
                let by_building = search(memo, blueprint, new_time, built, new_robots);

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

//type State = (u16, [u16; 4], [u16; 4]); // (Time, Resources, Robots)

fn parse_blueprint(input: &[u8]) -> IResult<&[u8], Blueprint> {
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
    cost: [[u16; 4]; 4],
}

// Super expensive test, leave commented out until optimized
// crate::test_day!(RUN, "day19", "1487", "13440");
