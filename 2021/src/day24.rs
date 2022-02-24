#![allow(unused_imports)]

use std::collections::btree_map::Range;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Rem};

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
    let mut cache = rustc_hash::FxHashSet::default();

    // Analyze problem for better pruning
    let mut ranges = range_analysis(
        validator,
        RangeVal::inclusive(set.clone().min().unwrap(), set.clone().max().unwrap()),
    );
    // After the last instruction, Z must be zero.
    ranges.last_mut().unwrap()[Var::Z.index()] = RangeVal::exact(0);
    restrict_ranges(validator, &mut ranges);

    // Perform the actual search

    let num_inputs = validator
        .iter()
        .filter(|inst| matches!(inst, Inst::Inp(_)))
        .count();
    let mut choices = Vec::new();
    let mut input = Vec::new();
    let mut states = Vec::new();
    let mut cur_state = State::new();

    choices.push(set.clone());

    'outer: while let Some(choice) = choices.last_mut() {
        if let Some(cur) = choice.next() {
            states.push(cur_state.clone());
            cur_state.step_input(validator, cur);

            for v in 0..4 {
                if !ranges[cur_state.ip][v].contains(cur_state.state[v]) {
                    cur_state = states.pop().unwrap();
                    continue 'outer;
                }
            }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RangeVal {
    from: i64,
    to: i64,
}

impl Display for RangeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..={}", self.from, self.to)
    }
}

impl RangeVal {
    fn inclusive(a: i64, b: i64) -> RangeVal {
        RangeVal {
            from: a.min(b),
            to: a.max(b),
        }
    }

    const fn exact(i: i64) -> RangeVal {
        RangeVal { from: i, to: i }
    }

    const fn as_exact(self) -> Option<i64> {
        if self.from == self.to {
            Some(self.from)
        } else {
            None
        }
    }

    fn limit_exclude(self, value: i64) -> Option<RangeVal> {
        if self.from == value && self.to == value {
            None
        } else if self.from == value {
            Some(RangeVal::inclusive(self.from + 1, self.to))
        } else if self.to == value {
            Some(RangeVal::inclusive(self.from, self.to - 1))
        } else {
            Some(self)
        }
    }

    const fn contains(self, value: i64) -> bool {
        value >= self.from && value <= self.to
    }

    fn eql(self, rhs: RangeVal) -> RangeVal {
        match (self.as_exact(), rhs.as_exact()) {
            (Some(va), Some(vb)) => RangeVal::exact((va == vb) as i64),
            _ => RangeVal::inclusive(0, 1),
        }
    }

    fn restrict(self, restriction: Self) -> Option<Self> {
        if restriction.to < self.from || restriction.from > self.to {
            None // no overlap, invalid
        } else {
            Some(RangeVal::inclusive(
                self.from.max(restriction.from),
                self.to.min(restriction.to),
            ))
        }
    }
}

impl Add for RangeVal {
    type Output = RangeVal;

    fn add(self, rhs: Self) -> Self::Output {
        RangeVal {
            from: self.from + rhs.from,
            to: self.to + rhs.to,
        }
    }
}

impl Rem for RangeVal {
    type Output = RangeVal;

    fn rem(self, rhs: Self) -> Self::Output {
        assert!(self.from >= 0);
        assert!(rhs.from > 0);

        if self.to < rhs.from {
            // Remainder fits entirely into divisor
            self
        } else {
            RangeVal::inclusive(0, (rhs.to - 1).min(self.to))
        }
    }
}

impl Mul for RangeVal {
    type Output = RangeVal;

    fn mul(self, rhs: Self) -> Self::Output {
        let (min, max) = [self.from, self.to]
            .into_iter()
            .flat_map(move |a| [a * rhs.from, a * rhs.to])
            .fold(
                (None, None),
                |(min, max): (Option<i64>, Option<i64>), elem| {
                    (
                        Some(min.map_or(elem, |prev| prev.min(elem))),
                        Some(max.map_or(elem, |prev| prev.max(elem))),
                    )
                },
            );
        RangeVal::inclusive(min.unwrap(), max.unwrap())
    }
}

impl Div for RangeVal {
    type Output = RangeVal;

    fn div(self, rhs: Self) -> Self::Output {
        let rhs = rhs.limit_exclude(0).expect("divisor must not be zero");
        if self.from > 0 {
            // 0 < from <= to
            if rhs.from > 0 {
                // 0 < from <= to
                RangeVal::inclusive(self.from / rhs.to, self.to / rhs.from)
            } else if rhs.to < 0 {
                // from <= to < 0
                RangeVal::inclusive(self.to / rhs.to, self.from / rhs.from)
            } else {
                // from < 0 < to (due to limit_exlude)
                RangeVal::inclusive(-self.to, self.to)
            }
        } else if self.to < 0 {
            // from <= to < 0
            if rhs.from > 0 {
                // 0 < from <= to
                RangeVal::inclusive(self.from / rhs.from, self.to / rhs.to)
            } else if rhs.to < 0 {
                // from <= to < 0
                RangeVal::inclusive(self.from / rhs.to, self.to / rhs.from)
            } else {
                // from < 0 < to (due to limit_exlude)
                RangeVal::inclusive(self.from, -self.from)
            }
        } else {
            // from <= 0 <= to
            if rhs.from > 0 {
                // 0 < from <= to
                RangeVal::inclusive(self.from / rhs.from, self.to / rhs.from)
            } else if rhs.to < 0 {
                // from <= to < 0
                RangeVal::inclusive(self.to / rhs.to, self.from / rhs.to)
            } else {
                // from < 0 < to (due to limit_exlude)
                RangeVal::inclusive(self.from.min(-self.to), self.to.max(-self.from))
            }
        }
    }
}

/// Forward-pass through the program computing the possible ranges of each register after each
/// instruction.
fn range_analysis(prog: &[Inst], input: RangeVal) -> Vec<[RangeVal; 4]> {
    let mut state = [RangeVal::exact(0); 4];
    let mut states = vec![state];

    for inst in prog {
        match inst {
            Inst::Inp(v) => state[v.index()] = input,
            Inst::Add(a, b) => state[a.index()] = state[a.index()] + range_operand(*b, &state),
            Inst::Mul(a, b) => state[a.index()] = state[a.index()] * range_operand(*b, &state),
            Inst::Div(a, b) => state[a.index()] = state[a.index()] / range_operand(*b, &state),
            Inst::Mod(a, b) => state[a.index()] = state[a.index()] % range_operand(*b, &state),
            Inst::Eql(a, b) => state[a.index()] = state[a.index()].eql(range_operand(*b, &state)),
        }

        states.push(state);
    }

    states
}

/// Backward pass through the program restricting the allowed ranges of each register before each
/// instruction based on the restricted range behind them.
/// The last entry in the `ranges` array is the restricted final state of the program.
fn restrict_ranges(prog: &[Inst], ranges: &mut [[RangeVal; 4]]) {
    assert!(ranges.len() == prog.len() + 1);

    for (ip, inst) in prog.into_iter().enumerate().rev() {
        let after = ranges[ip + 1];
        let before = &mut ranges[ip];

        // copy untouched variables
        for var in [Var::W, Var::X, Var::Y, Var::Z] {
            if inst.out_var() != var {
                before[var.index()] = after[var.index()];
            }
        }

        match inst {
            // Static RHS
            Inst::Add(a, Operand::Val(b)) => {
                let out = after[a.index()];
                before[a.index()] = before[a.index()]
                    .restrict(RangeVal::inclusive(out.from - b, out.to - b))
                    .unwrap();
            }
            Inst::Mul(a, Operand::Val(b)) => {
                let out = after[a.index()];
                if *b != 0 {
                    before[a.index()] = before[a.index()]
                        .restrict(RangeVal::inclusive(out.from / b, out.to / b))
                        .unwrap();
                }
            }
            Inst::Div(a, Operand::Val(b)) => {
                let out = after[a.index()];
                if *b > 0 {
                    before[a.index()] = before[a.index()]
                        .restrict(RangeVal::inclusive(out.from * b, out.to * b + (b - 1)))
                        .unwrap();
                } else {
                    // TODO: For a complete implementation, the other cases should also be
                    // implemented. For the concrete advent of code solution, this should be enough.
                }
            }

            // Dynamic RHS
            Inst::Add(a, Operand::Var(b)) => {
                let ra = after[a.index()];
                let rb = after[b.index()];

                before[a.index()] = before[a.index()]
                    .restrict(RangeVal::inclusive(ra.from - rb.from, ra.to - rb.from))
                    .unwrap();

                let ba = before[a.index()];
                before[b.index()] = before[b.index()]
                    .restrict(RangeVal::inclusive(ra.from - ba.from, ra.to - ba.from))
                    .unwrap();
            }
            Inst::Mul(a, Operand::Var(b)) => {
                let ra = after[a.index()];
                let rb = after[b.index()];

                if ra.from >= 0 && rb.from > 0 {
                    before[a.index()] = before[a.index()]
                        .restrict(RangeVal::inclusive(ra.from / rb.to, ra.to / rb.from))
                        .unwrap();
                    let ba = before[a.index()];
                    if ba.from > 0 {
                        before[b.index()] = before[b.index()]
                            .restrict(RangeVal::inclusive(ra.from / ba.to, ra.to / ba.from))
                            .unwrap();
                    }
                } else {
                    // TODO: For a complete implementation, the other cases should also be
                    // implemented. For the concrete advent of code solution, this should be enough.
                }
            }

            // Division with a dynamic RHS doesn't seem to occur in the code for now
            Inst::Div(_, Operand::Var(_)) => {}

            // By implementing these as well, it might be possible to propagate constraints back to
            // the inputs, and use them to restrict the set of inputs that are even tried.
            Inst::Inp(_) => {}
            Inst::Eql(_, _) => {}

            // Cannot infer any useful information from mod, input values could be anything.
            Inst::Mod(_, _) => {}
        }
    }
}

fn range_operand(op: Operand, state: &[RangeVal; 4]) -> RangeVal {
    match op {
        Operand::Var(v) => state[v.index()],
        Operand::Val(v) => RangeVal::exact(v),
    }
}

#[allow(unused)]
fn debug_print_ranges(
    out: &mut dyn std::io::Write,
    prog: &[Inst],
    ranges: &[[RangeVal; 4]],
) -> std::io::Result<()> {
    assert!(ranges.len() == prog.len() + 1);

    fn print_range_state(
        out: &mut dyn std::io::Write,
        state: [RangeVal; 4],
    ) -> std::io::Result<()> {
        writeln!(
            out,
            "w={}, x={}, y={}, z={}",
            state[0], state[1], state[2], state[3]
        )
    }

    print_range_state(out, ranges[0])?;
    for (inst, state) in prog.into_iter().zip(ranges[1..].into_iter()) {
        writeln!(out, "  {:?}", inst)?;
        print_range_state(out, *state)?;
    }
    Ok(())
}

/// Cacheable state for the "seen states" HashSet.
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

    #[inline(always)]
    fn var(&self, var: Var) -> i64 {
        self.state[var.index()]
    }

    #[inline(always)]
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

impl Inst {
    pub fn out_var(&self) -> Var {
        match *self {
            Inst::Inp(v) => v,
            Inst::Add(v, _) => v,
            Inst::Mul(v, _) => v,
            Inst::Div(v, _) => v,
            Inst::Mod(v, _) => v,
            Inst::Eql(v, _) => v,
        }
    }

    pub fn uses_var(&self, var: Var) -> bool {
        match *self {
            Inst::Inp(v) => v == var,
            Inst::Add(v, op) => v == var || op.uses_var(var),
            Inst::Mul(v, op) => v == var || op.uses_var(var),
            Inst::Div(v, op) => v == var || op.uses_var(var),
            Inst::Mod(v, op) => v == var || op.uses_var(var),
            Inst::Eql(v, op) => v == var || op.uses_var(var),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Var(Var),
    Val(i64),
}

impl Operand {
    pub fn uses_var(&self, var: Var) -> bool {
        *self == Operand::Var(var)
    }
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

mod compiler {
    use std::arch::asm;

    use super::{Inst, Operand, Var};

    // W: RDI
    // X: RSI
    // Y: RDX
    // Z: RCX
    //
    // Input: R8, R9,

    struct Assembler {
        code: Vec<u8>,
    }

    impl Assembler {
        fn new() -> Self {
            Self { code: vec![] }
        }

        fn position(&self) -> usize {
            self.code.len()
        }

        fn emit_ret(&mut self) {
            self.code.push(0xc3);
        }

        fn emit_sete(&mut self, target: u8) {
            self.code
                .extend([0x0f, 0x94, 0b1100_0000 | (target & 0b111)]);
        }

        fn emit_mod_rm<const N: usize>(&mut self, opcode: [u8; N], reg: u8, rm: u8) {
            // [0100|WRXC]
            // W: wide
            // R: extension reg
            // X: extension SIB index
            // C: extenion rm
            let mut rex = 0b0100_1000;
            if reg >= 8 {
                rex |= 0b0100;
            }
            if rm >= 8 {
                rex |= 0b0001;
            }
            self.code.push(rex);
            self.code.extend(opcode);
            // 11 means register direct addressing
            // next 3 bits are reg, last 3 bits rm
            let mod_rm = 0b11000000 | (reg & 0b111) << 3 | (rm & 0b111);
            self.code.push(mod_rm);
        }
    }

    fn compile(prog: &[Inst]) -> Vec<u8> {
        let mut offsets = vec![0];
        let mut asm = Assembler::new();
        let mut first_input = true;

        for inst in prog {
            match inst {
                Inst::Inp(a) => {
                    if first_input {
                        first_input = false;
                    } else {
                        offsets.push(asm.position());
                        asm.emit_ret();
                    }
                    asm.emit_mod_rm([0x89], 12, var_reg(*a));
                }
                Inst::Add(a, Operand::Var(b)) => {
                    asm.emit_mod_rm([0x01], var_reg(*b), var_reg(*a));
                }
                Inst::Mul(a, Operand::Var(b)) => {
                    asm.emit_mod_rm([0x0F, 0xAF], var_reg(*b), var_reg(*a));
                }
                Inst::Div(a, Operand::Var(b)) => {
                    // clear RDX
                    asm.emit_mod_rm([0x31], 2, 2);
                    // store dividend in RAX
                    asm.emit_mod_rm([0x89], var_reg(*a), 0);
                    // divide by var
                    asm.emit_mod_rm([0xF7], 0b111, var_reg(*b));
                    // copy result (RAX) into var
                    asm.emit_mod_rm([0x89], 0, var_reg(*a));
                }
                Inst::Mod(a, Operand::Var(b)) => {
                    // clear RDX
                    asm.emit_mod_rm([0x31], 2, 2);
                    // store dividend in RAX
                    asm.emit_mod_rm([0x89], var_reg(*a), 0);
                    // divide by var
                    asm.emit_mod_rm([0xF7], 0b111, var_reg(*b));
                    // copy remainder (RDX) into var
                    asm.emit_mod_rm([0x89], 2, var_reg(*a));
                }
                Inst::Eql(a, Operand::Var(b)) => {
                    // clear RAX
                    asm.emit_mod_rm([0x31], 0, 0);
                    // compare
                    asm.emit_mod_rm([0x39], var_reg(*a), var_reg(*b));
                    // store result in RAX
                    asm.emit_sete(0);
                    // copy RAX to var
                    asm.emit_mod_rm([0x89], 0, var_reg(*a));
                }

                Inst::Add(a, Operand::Val(b)) => todo!(),
                Inst::Mul(a, Operand::Val(b)) => todo!(),
                Inst::Div(a, Operand::Val(b)) => todo!(),
                Inst::Mod(a, Operand::Val(b)) => todo!(),
                Inst::Eql(a, Operand::Val(b)) => todo!(),
            }
        }

        asm.emit_ret();

        asm.code
    }

    struct Exe {
        base: *mut libc::c_void,
        len: usize,
    }

    impl Exe {
        pub fn new(code: &[u8]) -> Option<Self> {
            let len = (code.len() + 0xfff) / 0x1000 * 0x1000;
            unsafe {
                let base = libc::mmap(
                    std::ptr::null_mut(),
                    len,
                    libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                    -1,
                    0,
                );
                if base.is_null() {
                    return None;
                }
                // create nop slide
                libc::memset(base, 0x90, len);
                // copy code to start
                libc::memcpy(base, code.as_ptr() as *const libc::c_void, code.len());
                Some(Exe { base, len })
            }
        }

        pub unsafe fn exec(&self, offset: usize, mut state: [i64; 4]) -> [i64; 4] {
            let target = self.base as usize + offset;
            asm!(
                "call rax",
                in("rax") target,
                inout("r8") state[0],
                inout("r9") state[1],
                inout("r10") state[2],
                inout("r11") state[3],
            );
            state
        }
    }

    impl Drop for Exe {
        fn drop(&mut self) {
            unsafe {
                libc::munmap(self.base, self.len);
            }
        }
    }

    fn var_reg(var: Var) -> u8 {
        match var {
            Var::W => 8,  // R8
            Var::X => 9,  // R9
            Var::Y => 10, // R10
            Var::Z => 11, // R11
        }
    }

    #[test]
    fn test_compile() {
        let code = compile(&[Inst::Div(Var::Y, Operand::Var(Var::X))]);
        assert_eq!(
            code,
            vec![0x48, 0x31, 0xD2, 0x4C, 0x89, 0xD0, 0x49, 0xF7, 0xF9, 0x49, 0x89, 0xC2, 0xC3]
        );
        unsafe {
            let exe = Exe::new(&code).unwrap();
            let new = exe.exec(0, [1, 2, 10, 11]);
            assert_eq!(new, [1, 2, 5, 11]);
        }
    }
}

#[cfg(test)]
mod range_test {
    use super::RangeVal;

    fn forall_ranges_filter(
        op: &str,
        valid_r: impl Fn(RangeVal, RangeVal) -> bool,
        valid_v: impl Fn(i64, i64) -> bool,
        rop: impl Fn(RangeVal, RangeVal) -> RangeVal,
        vop: impl Fn(i64, i64) -> i64,
    ) {
        for f1 in -10..=10 {
            for t1 in f1..=10 {
                for v1 in f1..=t1 {
                    let r1 = RangeVal::inclusive(f1, t1);

                    for f2 in -10..=10 {
                        for t2 in f2..=10 {
                            for v2 in f2..=t2 {
                                let r2 = RangeVal::inclusive(f2, t2);

                                if !valid_r(r1, r2) || !valid_v(v1, v2) {
                                    continue;
                                }

                                let output_range = rop(r1, r2);
                                let output_value = vop(v1, v2);

                                assert!(
                                    output_range.contains(output_value),
                                    "{} {} {} = {} but {} {} {} = {}",
                                    r1,
                                    op,
                                    r2,
                                    output_range,
                                    v1,
                                    op,
                                    v2,
                                    output_value
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    fn forall_ranges(
        op: &str,
        rop: impl Fn(RangeVal, RangeVal) -> RangeVal,
        vop: impl Fn(i64, i64) -> i64,
    ) {
        forall_ranges_filter(op, |_, _| true, |_, _| true, rop, vop)
    }

    #[test]
    fn basic_ops() {
        forall_ranges("+", |a, b| a + b, |a, b| a + b);
        forall_ranges("*", |a, b| a * b, |a, b| a * b);
        forall_ranges_filter(
            "/",
            |_r1, r2| r2.as_exact() != Some(0),
            |_v1, v2| v2 != 0,
            |a, b| a / b,
            |a, b| a / b,
        );
        forall_ranges_filter(
            "%",
            |r1, r2| r1.from >= 0 && r2.from > 0,
            |_v1, v2| v2 != 0,
            |a, b| a % b,
            |a, b| a % b,
        );
        forall_ranges("==", |a, b| a.eql(b), |a, b| (a == b) as i64);
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
