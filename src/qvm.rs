use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;
use ndarray::prelude::*;

type Complex = num_complex::Complex64;
type Qubit = Array1<Complex>;
type Qubyte = (Qubit, Qubit);

type G1 = Array2<Complex>;

#[derive(Serialize, PartialEq)]
enum Instruction {
    Malformed,
    Single(String, String),
    Double(String, String, String),
}

pub struct QVM {
    pub counter: usize,
    pub qb: Qubyte,
    program: Vec<Instruction>,
    gates: BTreeMap<String, G1>,
}

const C0: Complex = Complex { re: 0.0, im: 0.0 };
const C1: Complex = Complex { re: 1.0, im: 0.0 };
const I: Complex = Complex { re: 0.0, im: 1.0 };

fn x() -> G1 {
    array![[C0, C1], [C1, C0]]
}
fn z() -> G1 {
    array![[C1, C0], [C0, -C1]]
}
fn y() -> G1 {
    array![[C0, -I], [I, C0]]
}
fn h() -> G1 {
    let h = 1.0 / Complex { re: 2.0, im: 0.0 }.sqrt();
    array![[h, h], [h, -h]]
}

fn zero() -> Qubit {
    array![C1, C0]
}
fn one() -> Qubit {
    array![C1, C0]
}

fn apply(gate: &G1, qb: &Qubit) -> Qubit {
    qb.dot(gate)
}

impl QVM {
    pub fn new() -> QVM {
        let mut map = BTreeMap::new();
        map.insert("x".into(), x());
        map.insert("y".into(), y());
        map.insert("z".into(), z());
        map.insert("h".into(), h());
        QVM {
            counter: 0,
            qb: (zero(), zero()),
            program: vec![Instruction::Single("x".into(), "0".into())],
            gates: map,
        }
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
    pub fn reset(&mut self) {
        self.counter = 0;
        self.qb = (zero(), zero());
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
            return false
        } else {
            self.program = prog;
            return true
        };
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
        if let Instruction::Single(gate, _) = &self.program[self.counter] {
            let gate = &self.gates[gate];
            self.qb = (apply(&gate, &self.qb.0), self.qb.1.clone());
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

impl fmt::Display for QVM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.qb.0)
    }
}
