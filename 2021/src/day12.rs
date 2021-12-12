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

    let mut num_paths = 0;

    dfs(&graph, &mut SmallOnce::default(), |_| num_paths += 1);

    Ok(num_paths)
}

pub fn part2(input: &[u8]) -> anyhow::Result<i64> {
    let graph = parsers::parse(p_graph, input)?;

    let mut num_paths = 0;

    dfs(&graph, &mut SmallOnceTwice::default(), |_| num_paths += 1);

    Ok(num_paths)
}

fn dfs<'a, V, F>(graph: &Graph<'a>, visited: &mut V, mut callback: F)
where
    V: Visitor<'a>,
    F: FnMut(&[&str]),
{
    let mut path = vec!["start"];
    let mut choices = vec![0];

    while let Some(cur) = path.last() {
        let cur = *cur;
        if cur == "end" {
            //eprintln!("{:?}", path);
            callback(&path);

            visited.unvisit(cur);
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

            if visited.visit(next) {
                // Update this choice
                choices.push(choice + 1);
                // Prepare next choice
                choices.push(0);

                path.push(next);
                had_choice = true;
                break;
            } else {
                // already was there
                choice += 1;
                continue;
            }
        }
        if !had_choice {
            visited.unvisit(cur);
            // Choices exhausted, also pop current cave
            path.pop();
        }
    }
}

trait Visitor<'a> {
    fn visit(&mut self, vertex: &'a str) -> bool;
    fn unvisit(&mut self, vertex: &'a str);
}

#[derive(Default)]
struct SmallOnce<'a> {
    visited: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for SmallOnce<'a> {
    fn visit(&mut self, vertex: &'a str) -> bool {
        if vertex == "start" {
            // already visited
            false
        } else if is_large_cave(vertex) {
            true
        } else {
            self.visited.insert(vertex)
        }
    }

    fn unvisit(&mut self, vertex: &str) {
        self.visited.remove(vertex);
    }
}

#[derive(Default)]
struct SmallOnceTwice<'a> {
    visited: HashSet<&'a str>,
    visited_twice: Option<&'a str>,
}

impl<'a> Visitor<'a> for SmallOnceTwice<'a> {
    fn visit(&mut self, vertex: &'a str) -> bool {
        if vertex == "start" {
            // already visited
            false
        } else if is_large_cave(vertex) {
            true
        } else {
            let visited_first_time = self.visited.insert(vertex);
            if visited_first_time {
                true
            } else if self.visited_twice.is_none() {
                self.visited_twice = Some(vertex);
                true
            } else {
                false
            }
        }
    }

    fn unvisit(&mut self, vertex: &str) {
        if self.visited_twice == Some(vertex) {
            self.visited_twice = None;
        } else {
            self.visited.remove(vertex);
        }
    }
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
    // TODO: make faster by using numeric vertex IDs
    neighbours: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Graph<'a> {
    fn new() -> Self {
        Self::default()
    }
}

crate::test_day!(crate::day12::RUN, "day12", 3802, 99448);
