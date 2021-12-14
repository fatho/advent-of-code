#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::{parsers, Day};
use anyhow::Context;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{flat_map, map, map_res};
use nom::multi::fold_many0;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let graph = parsers::parse(p_graph, input)?;

    let mut num_paths = 0;
    let start = *graph
        .vertices
        .get("start")
        .context("must have start node")?;
    let end = *graph.vertices.get("end").context("must have end node")?;

    dfs(
        &graph,
        start,
        end,
        &mut SmallOnce::new(graph.vertices.len()),
        |_| num_paths += 1,
    );

    Ok(format!("{}", num_paths))
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let graph = parsers::parse(p_graph, input)?;

    let mut num_paths = 0;
    let start = *graph
        .vertices
        .get("start")
        .context("must have start node")?;
    let end = *graph.vertices.get("end").context("must have end node")?;

    dfs(
        &graph,
        start,
        end,
        &mut SmallOnceTwice::new(graph.vertices.len()),
        |_| num_paths += 1,
    );

    Ok(format!("{}", num_paths))
}

/// Brute force DFS solution for the problem. This might not be the most
/// efficient way to go about it.
///
/// TODO: explore other solution ideas, e.g. a BFS starting at end, or maybe
/// dynamic programming.
fn dfs<'a, V, F>(graph: &Graph<'a>, start: u32, end: u32, visited: &mut V, mut callback: F)
where
    V: Visitor,
    F: FnMut(&[u32]),
{
    let mut path = vec![start];
    let mut choices = vec![0];

    while let Some(cur) = path.last() {
        let cur = *cur;
        if cur == end {
            //eprintln!("{:?}", path);
            callback(&path);

            visited.unvisit(cur);
            path.pop();
            choices.pop();
            continue;
        }

        let neighbours = graph
            .neighbours
            .get(cur as usize)
            .expect("node does not exist");
        let mut choice = choices
            .pop()
            .expect("must have one choice entry per path entry");

        let mut had_choice = false;
        while choice < neighbours.len() {
            let next = neighbours[choice];

            if next != start && (graph.is_large_cave(next) || visited.visit(next)) {
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

trait Visitor {
    fn visit(&mut self, vertex: u32) -> bool;
    fn unvisit(&mut self, vertex: u32);
}

struct SmallOnce {
    visited: Vec<bool>,
}

impl SmallOnce {
    fn new(count: usize) -> Self {
        Self {
            visited: vec![false; count],
        }
    }
}

impl Visitor for SmallOnce {
    fn visit(&mut self, vertex: u32) -> bool {
        !std::mem::replace(&mut self.visited[vertex as usize], true)
    }

    fn unvisit(&mut self, vertex: u32) {
        self.visited[vertex as usize] = false
    }
}

#[derive(Default)]
struct SmallOnceTwice {
    visited: Vec<bool>,
    visited_twice: Option<u32>,
}

impl SmallOnceTwice {
    fn new(count: usize) -> Self {
        Self {
            visited: vec![false; count],
            visited_twice: None,
        }
    }
}

impl Visitor for SmallOnceTwice {
    fn visit(&mut self, vertex: u32) -> bool {
        let already_visited = std::mem::replace(&mut self.visited[vertex as usize], true);
        if already_visited && self.visited_twice.is_none() {
            self.visited_twice = Some(vertex);
            true
        } else {
            !already_visited
        }
    }

    fn unvisit(&mut self, vertex: u32) {
        if self.visited_twice == Some(vertex) {
            self.visited_twice = None;
        } else {
            self.visited[vertex as usize] = false;
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
            let id1 = graph.node(node1);
            let id2 = graph.node(node2);
            graph.add_edge(id1, id2);
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
    vertices: HashMap<&'a str, u32>,
    neighbours: Vec<Vec<u32>>,
    is_large: Vec<bool>,
}

impl<'a> Graph<'a> {
    fn node(&mut self, name: &'a str) -> u32 {
        let next_id = self.vertices.len() as u32;
        let id = *self.vertices.entry(name).or_insert(next_id);
        if id == next_id {
            self.neighbours.push(Vec::new());
            self.is_large.push(is_large_cave(name));
        }
        id
    }

    fn add_edge(&mut self, n1: u32, n2: u32) {
        self.neighbours[n1 as usize].push(n2);
        self.neighbours[n2 as usize].push(n1);
    }

    fn is_large_cave(&self, next: u32) -> bool {
        self.is_large[next as usize]
    }
}

crate::test_day!(crate::day12::RUN, "day12", "3802", "99448");
