#![allow(unused_imports)]

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
    // let input = &[1, 3, 5, 7, 9, 2, 4, 6, 8, 9, 9, 9, 9, 9];

    // println!("{:?}", run(&validator, input));

    // let mut input = [9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9];

    // for i in (0..input.len()).rev() {
    //     while input[i] > 0 {
    //         let out = run(&validator, &input);
    //         if out[Var::Z.index()] == 0 {
    //             break;
    //         } else {
    //             input[i] -= 1;
    //         }
    //     }
    //     if input[i] == 0 {
    //         input[i] = 9;
    //     }
    // }

    // println!("{:?}", input);

    let ssa = to_ssa(&validator);
    println!("{}", ssa.dot());

    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expr {
    Inp(usize),
    Const(i64),
    Add(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Mod(usize, usize),
    Eql(usize, usize),
}

struct SsaProg {
    exprs: Vec<Expr>,
    state: [usize; 4],
}

impl SsaProg {
    pub fn new() -> Self {
        Self {
            exprs: vec![Expr::Const(0)],
            state: [0; 4],
        }
    }

    pub fn push_expr(&mut self, expr: Expr) -> usize {
        let id = self.exprs.len();
        self.exprs.push(expr);
        id
    }

    pub fn push_binop(&mut self, a: Var, b: Operand, op: impl Fn(usize, usize) -> Expr) {
        let ae = self.state[a.index()];
        let be = match b {
            Operand::Val(v) => self.push_expr(Expr::Const(v)),
            Operand::Var(v) => self.state[v.index()],
        };
        let e = self.push_expr(op(ae, be));
        self.state[a.index()] = e;
    }

    pub fn dot(&self) -> String {
        use std::fmt::Write;

        let mut viz = String::new();
        viz.push_str("digraph G {\n");
        for (i, e) in self.exprs.iter().enumerate() {
            writeln!(&mut viz, "  v{} [label=\"{}: {:?}\"];", i, i, e).unwrap();
            let sources = match e {
                Expr::Inp(_) => None,
                Expr::Const(_) => None,
                Expr::Add(a, b) => Some((a, b)),
                Expr::Mul(a, b) => Some((a, b)),
                Expr::Div(a, b) => Some((a, b)),
                Expr::Mod(a, b) => Some((a, b)),
                Expr::Eql(a, b) => Some((a, b)),
            };
            if let Some((a, b)) = sources {
                writeln!(&mut viz, "  v{} -> v{};", a, i).unwrap();
                writeln!(&mut viz, "  v{} -> v{};", b, i).unwrap();
            }
        }
        viz.push_str("}\n");
        viz
    }
}

fn to_ssa(prog: &[Inst]) -> SsaProg {
    let mut out = SsaProg::new();
    let mut input_index = 0;
    for inst in prog {
        match inst {
            Inst::Inp(var) => {
                let e = out.push_expr(Expr::Inp(input_index));
                input_index += 1;
                out.state[var.index()] = e;
            }
            Inst::Add(a, b) => out.push_binop(*a, *b, Expr::Add),
            Inst::Mul(a, b) => out.push_binop(*a, *b, Expr::Mul),
            Inst::Div(a, b) => out.push_binop(*a, *b, Expr::Div),
            Inst::Mod(a, b) => out.push_binop(*a, *b, Expr::Mod),
            Inst::Eql(a, b) => out.push_binop(*a, *b, Expr::Eql),
        }
    }
    out
}

pub fn part2(input: &[u8]) -> anyhow::Result<String> {
    let validator = parsers::parse(p_prog, input)?;

    todo!()
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

// The ALU is a four-dimensional processing unit: it has integer variables w, x, y, and z. These variables all start with the value 0. The ALU also supports six instructions:

//     inp a - Read an input value and write it to variable a.
//     add a b - Add the value of a to the value of b, then store the result in variable a.
//     mul a b - Multiply the value of a by the value of b, then store the result in variable a.
//     div a b - Divide the value of a by the value of b, truncate the result to an integer, then store the result in variable a. (Here, "truncate" means to round the value toward zero.)
//     mod a b - Divide the value of a by the value of b, then store the remainder in variable a. (This is also called the modulo operation.)
//     eql a b - If the value of a and b are equal, then store the value 1 in variable a. Otherwise, store the value 0 in variable a.

pub fn run(prog: &[Inst], input: &[i64]) -> [i64; 4] {
    let mut state = [0; 4];
    let mut input_index = 0;
    for inst in prog {
        match inst {
            Inst::Inp(target) => {
                state[target.index()] = input[input_index];
                input_index += 1
            }
            Inst::Add(a, b) => binop(&mut state, *a, *b, |av, bv| av + bv),
            Inst::Mul(a, b) => binop(&mut state, *a, *b, |av, bv| av * bv),
            Inst::Div(a, b) => binop(&mut state, *a, *b, |av, bv| av / bv),
            Inst::Mod(a, b) => binop(&mut state, *a, *b, |av, bv| av % bv),
            Inst::Eql(a, b) => binop(&mut state, *a, *b, |av, bv| (av == bv) as i64),
        }
    }
    state
}

fn binop<F: Fn(i64, i64) -> i64>(state: &mut [i64; 4], a: Var, b: Operand, run: F) {
    let aval = state[a.index()];
    let bval = match b {
        Operand::Var(var) => state[var.index()],
        Operand::Val(val) => val,
    };
    state[a.index()] = run(aval, bval);
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

        assert_eq!(run(&prog, &[0b1010]), [1, 0, 1, 0]);
        assert_eq!(run(&prog, &[0b0101]), [0, 1, 0, 1]);
    }
}

crate::test_day!(crate::day24::RUN, "day24", "not solved", "not solved");
