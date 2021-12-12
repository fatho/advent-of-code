#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::{parsers, Day};
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map, map_res};
use nom::multi::fold_many0;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<i64> {
    let graph = parsers::parse(p_graph, input)?;

    let mut path = vec!["start"];
    let mut choices = vec![0];
    let mut visited: HashSet<&str> = HashSet::new();

    let mut num_paths = 0;

    while let Some(cur) = path.last() {
        let cur = *cur;
        if cur == "end" {
            //eprintln!("{:?}", path);
            num_paths += 1;

            visited.remove(cur);
            path.pop();
            choices.pop();
            continue;
        }

        let neighbours = graph.neighbours.get(cur).expect("node does not exist");
        let mut choice = choices
            .pop()
            .expect("must have one choice entry per path entry");

        let mut had_choice = false;
        while choice < neighbours.len() {
            let next = neighbours[choice];

            if next == "start" || (visited.contains(next) && !is_large_cave(next)) {
                // already was there
                choice += 1;
                continue;
            }

            // Update this choice
            choices.push(choice + 1);
            // Prepare next choice
            choices.push(0);

            visited.insert(next);
            path.push(next);
            had_choice = true;
            break;
        }
        if !had_choice {
            visited.remove(cur);
            // Choices exhausted, also pop current cave
            path.pop();
        }
    }

    Ok(num_paths)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let graph = parsers::parse(p_graph, input)?;

    todo!();
}

fn is_large_cave(name: &str) -> bool {
    !name.is_empty() && (b'A'..=b'Z').contains(&name.as_bytes()[0])
}

fn p_graph(input: &[u8]) -> IResult<&[u8], Graph> {
    fold_many0(
        terminated(separated_pair(p_node, tag("-"), p_node), parsers::newline),
        Graph::default,
        |mut graph, (node1, node2)| {
            graph.neighbours.entry(node1).or_default().push(node2);
            graph.neighbours.entry(node2).or_default().push(node1);
            graph
        },
    )(input)
}

fn p_node(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while(nom::character::is_alphabetic), |chrs| {
        std::str::from_utf8(chrs)
    })(input)
}

#[derive(Default)]
struct Graph<'a> {
    neighbours: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Graph<'a> {
    fn new() -> Self {
        Self::default()
    }
}

crate::test_day!(crate::day12::RUN, "day12", 0, 0);
