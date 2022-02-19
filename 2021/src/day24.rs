#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::{parsers, Day};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete as numbers;
use nom::character::complete::one_of;
use nom::combinator::{flat_map, map};
use nom::multi::{fold_many0, many0};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;
pub static RUN: Day = Day { part1, part2 };

pub fn part1(input: &[u8]) -> anyhow::Result<String> {
    let validator = parsers::parse(p_prog, input)?;

    let input = find_input(&validator, (1..=9).rev());
    let result = input.into_iter().fold(0, |acc, d| acc * 10 + d);

    Ok(result.to_string())
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let validator = parsers::parse(p_prog, input)?;
    let input = find_input(&validator, 1..=9);
    let result = input.into_iter().fold(0, |acc, d| acc * 10 + d);

    Ok(result.to_string())
}

pub fn find_input<I: Iterator<Item = i64> + Clone>(validator: &[Inst], set: I) -> Vec<i64> {
    let mut cache = HashSet::new();

    let num_inputs = validator
        .iter()
        .filter(|inst| matches!(inst, Inst::Inp(_)))
        .count();
    let mut choices = Vec::new();
    let mut input = Vec::new();
    let mut states = Vec::new();
    let mut cur_state = State::new();

    choices.push(set.clone());

    while let Some(choice) = choices.last_mut() {
        if let Some(cur) = choice.next() {
            states.push(cur_state.clone());
            cur_state.step_input(validator, cur);

            if !cache.insert(cur_state.clone()) {
                // if we've seen this state before, it means we entered it with
                // higher preceding digits already, and didn't find a solution
                // then. So we won't find a solution now either.
                cur_state = states.pop().unwrap();
                continue;
            }

            if input.len() == num_inputs - 1 {
                if cur_state.state[Var::Z.index()] == 0 {
                    input.push(cur);
                    break;
                } else {
                    // backtrack
                    cur_state = states.pop().unwrap()
                }
            } else {
                // descend
                input.push(cur);
                choices.push(set.clone());
            }
        } else {
            // exhausted, backtrack
            choices.pop();
            input.pop();
            cur_state = states.pop().unwrap();
        }
    }
    input
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    state: [i64; 4],
    ip: usize,
}

impl State {
    pub fn new() -> Self {
        State {
            ip: 0,
            state: [0; 4],
        }
    }

    // The ALU is a four-dimensional processing unit: it has integer variables w, x, y, and z. These variables all start with the value 0. The ALU also supports six instructions:

    //     inp a - Read an input value and write it to variable a.
    //     add a b - Add the value of a to the value of b, then store the result in variable a.
    //     mul a b - Multiply the value of a by the value of b, then store the result in variable a.
    //     div a b - Divide the value of a by the value of b, truncate the result to an integer, then store the result in variable a. (Here, "truncate" means to round the value toward zero.)
    //     mod a b - Divide the value of a by the value of b, then store the remainder in variable a. (This is also called the modulo operation.)
    //     eql a b - If the value of a and b are equal, then store the value 1 in variable a. Otherwise, store the value 0 in variable a.

    /// Run the ALU program until the next input instruction.
    fn step_input(&mut self, prog: &[Inst], input: i64) {
        let mut consumed_input = false;
        while self.ip < prog.len() {
            let cur = prog[self.ip];
            match cur {
                Inst::Inp(var) => {
                    if consumed_input {
                        return;
                    }
                    self.state[var.index()] = input;
                    consumed_input = true;
                }
                Inst::Add(a, b) => self.state[a.index()] = self.var(a) + self.operand(b),
                Inst::Mul(a, b) => self.state[a.index()] = self.var(a) * self.operand(b),
                Inst::Div(a, b) => self.state[a.index()] = self.var(a) / self.operand(b),
                Inst::Mod(a, b) => self.state[a.index()] = self.var(a) % self.operand(b),
                Inst::Eql(a, b) => self.state[a.index()] = (self.var(a) == self.operand(b)) as i64,
            }
            self.ip += 1;
        }
    }

    fn var(&self, var: Var) -> i64 {
        self.state[var.index()]
    }

    fn operand(&self, op: Operand) -> i64 {
        match op {
            Operand::Var(v) => self.var(v),
            Operand::Val(val) => val,
        }
    }
}

fn p_prog(input: &[u8]) -> IResult<&[u8], Vec<Inst>> {
    many0(terminated(p_inst, parsers::newline))(input)
}

fn p_inst(input: &[u8]) -> IResult<&[u8], Inst> {
    alt((
        preceded(tag("inp "), map(p_var, Inst::Inp)),
        p_binop("add", Inst::Add),
        p_binop("mul", Inst::Mul),
        p_binop("div", Inst::Div),
        p_binop("mod", Inst::Mod),
        p_binop("eql", Inst::Eql),
    ))(input)
}

fn p_binop(
    name: &'static str,
    make: impl Fn(Var, Operand) -> Inst,
) -> impl for<'a> Fn(&'a [u8]) -> IResult<&'a [u8], Inst> {
    move |input: &[u8]| {
        map(
            tuple((tag(name), tag(" "), p_var, tag(" "), p_operand)),
            |(_, _, a, _, b)| make(a, b),
        )(input)
    }
}

fn p_operand(input: &[u8]) -> IResult<&[u8], Operand> {
    alt((map(p_var, Operand::Var), map(numbers::i64, Operand::Val)))(input)
}

fn p_var(input: &[u8]) -> IResult<&[u8], Var> {
    map(one_of("wxyz"), |ch| match ch {
        'w' => Var::W,
        'x' => Var::X,
        'y' => Var::Y,
        'z' => Var::Z,
        _ => unreachable!(),
    })(input)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Inst {
    Inp(Var),
    Add(Var, Operand),
    Mul(Var, Operand),
    Div(Var, Operand),
    Mod(Var, Operand),
    Eql(Var, Operand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Var(Var),
    Val(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    W,
    X,
    Y,
    Z,
}

impl Var {
    pub fn index(self) -> usize {
        match self {
            Var::W => 0,
            Var::X => 1,
            Var::Y => 2,
            Var::Z => 3,
        }
    }
}

#[cfg(test)]
mod alutest {
    use super::*;

    #[test]
    fn example() {
        let (rest, prog) = p_prog(
            b"inp w
add z w
mod z 2
div w 2
add y w
mod y 2
div w 2
add x w
mod x 2
div w 2
mod w 2
",
        )
        .unwrap();
        assert_eq!(rest.len(), 0);

        let mut st = State::new();
        st.step_input(&prog, 0b1010);
        assert_eq!(st.state, [1, 0, 1, 0]);

        let mut st = State::new();
        st.step_input(&prog, 0b0101);
        assert_eq!(st.state, [0, 1, 0, 1]);
    }
}

crate::test_day!(
    crate::day24::RUN,
    "day24",
    "74929995999389",
    "11118151637112"
);
