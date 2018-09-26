use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;
use std::f64::EPSILON;

type Complex = num_complex::Complex64;

pub fn is_zero(c: Complex) -> bool {
    c.re.abs() < EPSILON && c.im.abs() < EPSILON
}
const N_QUBITS: usize = 4;
const Q_STATE_SIZE: usize = 2 * 2 * 2 * 2;

type Q1 = [Complex; 2];
type Q2 = [Complex; 2 * 2];
type Q3 = [Complex; Q_STATE_SIZE];
type Qstate = Q3;

type G1 = [Q1; 2];
type G2 = [Q2; 2 * 2];
type G3 = [Q3; 2 * 2 * 2 * 2];

#[derive(Serialize, Deserialize)]
enum Gate {
    A(Box<G1>),
    B(Box<G2>),
    C(Box<G3>),
}

#[derive(Serialize, PartialEq)]
pub enum Instruction {
    Malformed,
    Single(String, String),
    Double(String, String, String),
}

pub struct QVM {
    pub counter: usize,
    pub state: Qstate,
    pub program: Vec<Instruction>,
    gates: BTreeMap<String, Gate>,
}

trait Mut {
    fn apply(&mut self, _gate: &G3) {
    }
}

impl Mut for Qstate {
    fn apply(&mut self, gate: &G3) {
        let new_state = dot_product(&gate, &self);
        for i in 0..self.len() {
            self[i] = new_state[i];
        }
    }
}

pub const C0: Complex = Complex { re: 0.0, im: 0.0 };
pub const C1: Complex = Complex { re: 1.0, im: 0.0 };
const CI: Complex = Complex { re: 0.0, im: 1.0 };

const I1: G1 = [[C1, C0], [C0, C1]];
const I2: G2 = [
    [C1, C0, C0, C0],
    [C0, C0, C1, C0],
    [C0, C1, C0, C0],
    [C0, C0, C0, C1],
];
const SWAP: G2 = [
    [C1, C0, C0, C0],
    [C0, C0, C1, C0],
    [C0, C1, C0, C0],
    [C0, C0, C0, C1],
];

fn standard_gates() -> BTreeMap<String, Gate> {
    let mut map = BTreeMap::new();
    map.insert("x".to_string(), Gate::A(Box::new([[C0, C1], [C1, C0]])));
    map.insert("z".into(), Gate::A(Box::new([[C1, C0], [C0, -C1]])));
    map.insert("y".into(), Gate::A(Box::new([[C0, -CI], [CI, C0]])));

    let h = 1.0 / Complex { re: 2.0, im: 0.0 }.sqrt();
    map.insert("h".into(), Gate::A(Box::new([[h, h], [h, -h]])));
    map.insert("i1".into(), Gate::A(Box::new(I1)));
    map.insert("i2".into(), Gate::B(Box::new(I2)));
    map.insert(
        "cnot".into(),
        Gate::B(Box::new([
            [C1, C0, C0, C0],
            [C0, C1, C0, C0],
            [C0, C0, C0, C1],
            [C0, C0, C1, C0],
        ])),
    );
    map.insert("swap".into(), Gate::B(Box::new(SWAP)));
    map
}
fn zero() -> Qstate {
    let mut ret = [C0; Q_STATE_SIZE];
    ret[0] = C1;
    ret
}

fn dot_product(gate: &G3, state: &Qstate) -> Qstate {
    let mut ret = [C0; Q_STATE_SIZE];
    for (i, row) in gate.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            ret[i] += item * state[j];
        }
    }
    ret
}

fn tp1(a: &G1, b: &G1) -> G2 {
    let mut ret = [[C0; 4]; 4];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * 2 + b_i;
                    let cdx = a_j * 2 + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp2(a: &G2, b: &G2) -> G3 {
    let mut ret = [[C0; Q_STATE_SIZE]; Q_STATE_SIZE];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * N_QUBITS + b_i;
                    let cdx = a_j * N_QUBITS + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp(a: &G1, b: &G1, c: &G1, d: &G1) -> G3 {
    tp2(&tp1(a, b), &tp1(c, d))
}

impl QVM {
    pub fn new() -> QVM {
        QVM {
            counter: 0,
            state: zero(),
            program: vec![],
            gates: standard_gates(),
        }
    }
    pub fn reset(&mut self) {
        self.counter = 0;
        self.state = zero();
    }
    pub fn read_program(&self) -> String {
        String::from_iter(self.program.iter().map(|inst| match inst {
            Instruction::Single(gate, qb) => gate.clone() + " " + qb + "\n",
            Instruction::Double(gate, a1, a2) => gate.clone() + " " + a1 + " " + a2 + "\n",
            _ => "".into(),
        }))
    }
    pub fn update(&mut self, program: &str) -> bool {
        let prog = program
            .lines()
            .map(|line| {
                let words = line.split_whitespace().collect::<Vec<&str>>();
                match words.len() {
                    2 => Instruction::Single(words[0].into(), words[1].into()),
                    3 => Instruction::Double(words[0].into(), words[1].into(), words[2].into()),
                    _ => Instruction::Malformed,
                }
            }).collect::<Vec<Instruction>>();
        if prog.contains(&Instruction::Malformed) {
            false
        } else {
            self.program = prog;
            true
        }
    }
    pub fn set_gates(&mut self, gates: &str) -> bool {
        match serde_json::from_str(gates) {
            Ok(obj) => {
                self.gates = obj;
                true
            }
            Err(_) => false,
        }
    }
    pub fn show_gates(&self) -> String {
        serde_json::to_string_pretty(&self.gates).unwrap()
    }
    fn operate(&mut self) {
        if let Instruction::Single(gate, qb) = &self.program[self.counter] {
            if let Gate::A(gate) = &self.gates[gate] {
                let lifted = match qb.as_str() {
                    "0" => tp(&I1, &I1, &I1, gate),
                    "1" => tp(&I1, &I1, gate, &I1),
                    "2" => tp(&I1, gate, &I1, &I1),
                    "3" => tp(gate, &I1, &I1, &I1),
                    _ => panic!("bad target {}", qb),
                };
                self.state.apply(&lifted);
            }
        } else if let Instruction::Double(gate, qb0, qb1) = &self.program[self.counter] {
            if let Gate::B(gate) = &self.gates[gate] {
                match (qb0.as_str(), qb1.as_str()) {
                    // TODO why did it reverse from Q2?
                    ("0", "1") => {
                        let swapper = &tp2(&I2, &SWAP);
                        self.state.apply(swapper);
                        self.state.apply(&tp2(&I2, gate));
                        self.state.apply(swapper);
                    }
                    ("1", "0") => {
                        self.state.apply(&tp2(&I2, gate));
                    }
                    // TODO add "1", "2", etc
                    ("2", "3") => {
                        let swapper = &tp2(&SWAP, &I2);
                        self.state.apply(swapper);
                        self.state.apply(&tp2(&gate, &I2));
                        self.state.apply(swapper);
                    }
                    ("3", "2") => {
                        self.state.apply(&tp2(&gate, &I2));
                    }
                    _ => panic!("bad qbits: {} {}", qb0, qb1),
                }
            }
        }
    }
    pub fn prev(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
            self.operate();
        }
    }
    pub fn next(&mut self) {
        if self.counter < self.program.len() {
            self.operate();
            self.counter += 1;
        }
    }
}
