use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;

type Complex = num_complex::Complex64;

const N_QUBITS: usize = 2;
const Q_STATE_SIZE: usize = N_QUBITS*2;

type Q1 = [Complex; 2];
type Q2 = [Complex; 2*2];
type Qstate = Q2;
type G1 = [Q1; 2];
type G2 = [Q2; 2*2];
type Gate = G2;

#[derive(Serialize, PartialEq)]
enum Instruction {
    Malformed,
    Single(String, String),
    Double(String, String, String),
}

pub struct QVM {
    pub counter: usize,
    pub state: Qstate,
    program: Vec<Instruction>,
    gates: BTreeMap<String, G1>,
    gates2: BTreeMap<String, G2>,
}

const C0: Complex = Complex { re: 0.0, im: 0.0 };
const C1: Complex = Complex { re: 1.0, im: 0.0 };
const I: Complex = Complex { re: 0.0, im: 1.0 };

fn x() -> G1 {
    [[C0, C1], [C1, C0]]
}
fn z() -> G1 {
    [[C1, C0], [C0, -C1]]
}
fn y() -> G1 {
    [[C0, -I], [I, C0]]
}
fn h() -> G1 {
    let h = 1.0 / Complex { re: 2.0, im: 0.0 }.sqrt();
    [[h, h], [h, -h]]
}

fn i() -> G1 {
    [
        [C1, C0],
        [C0, C1],
    ]
}
fn cnot() -> Gate {
    [
        [C1, C0, C0, C0],
        [C0, C1, C0, C0],
        [C0, C0, C0, C1],
        [C0, C0, C1, C0],
    ]
}

fn swap () -> Gate {
    [
        [C1, C0, C0, C0],
        [C0, C0, C1, C0],
        [C0, C1, C0, C0],
        [C0, C0, C0, C1],
    ]
}

fn zero() -> Qstate {
    [C1, C0, C0, C0]
}

fn dot_product(gate: &Gate, state: &Qstate) -> Qstate {
    let mut ret = [C0; Q_STATE_SIZE];
    for (i, row) in gate.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            ret[i] += item * state[j];
        }
    }
    ret

}
fn lift(a: &G1, b: &G1) -> G2 {
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

impl QVM {
    pub fn new() -> QVM {
        let mut map = BTreeMap::new();
        let mut map2 = BTreeMap::new();
        map.insert("x".into(), x());
        map.insert("y".into(), y());
        map.insert("z".into(), z());
        map.insert("h".into(), h());
        map2.insert("cnot".into(), cnot());
        QVM {
            counter: 0,
            state : zero(),
            program: vec![],
            gates: map,
            gates2: map2,
        }
    }
    pub fn reset(&mut self) {
        self.counter = 0;
        self.state = zero();
    }
    pub fn read_program(&self) -> String {
        String::from_iter(self.program.iter().map(|inst| {
            match inst {
                Instruction::Single(gate, qb) => gate.clone() + " " + qb + "\n",
                Instruction::Double(gate, a1, a2) => gate.clone() + " " + a1 + " " + a2 + "\n",
                _ => "".into(),
            }
        }))
    }
    pub fn update(&mut self, program: &str) -> bool {
        let prog = program.lines().map(|line| {
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
            },
            Err(_) => {
                false
            }
        }
    }
    pub fn show_gates(&self) -> String {
        serde_json::to_string_pretty(&self.gates).unwrap()
    }
    fn operate(&mut self) {
        if let Instruction::Single(gate, qb) = &self.program[self.counter] {
            let gate = &self.gates[gate];
            let i = &i();
            let lifted = match qb.as_str() {
                "0" => lift(gate, i),
                "1" => lift(i, gate),
                _ => panic!("bad target {}", qb),
            };
            self.state = dot_product(&lifted, &self.state);
        } else if let Instruction::Double(gate, qb0, qb1) = &self.program[self.counter] {
            let gate = &self.gates2[gate];
            match (qb0.as_str(), qb1.as_str()) {
                ("0","1") => {
                    self.state = dot_product(gate, &self.state);
                },
                ("1","0") => {
                    let swapper = swap();
                    self.state = dot_product(&swapper, &self.state);
                    self.state = dot_product(gate, &self.state);
                    self.state = dot_product(&swapper, &self.state);
                },
                _ => panic!("bad qbits: {} {}", qb0, qb1)
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
