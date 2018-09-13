#![allow(dead_code)]

use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::fmt;

use ndarray::prelude::*;

type Complex = num_complex::Complex64;
type Qubit = Array1<Complex>;

type G1 = Array2<Complex>;

pub struct QVM {
    pub counter: usize,
    //pub qb1: Qubit,
    pub qb0: Qubit,
    pub program: Vec<char>,
    gates: BTreeMap<char, G1>,
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
    let h = 1.0 / Complex{re: 2.0, im: 0.0}.sqrt();
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
        map.insert('x', x());
        map.insert('y', y());
        map.insert('z', z());
        map.insert('h', h());
        QVM {
            counter: 0,
            qb0: zero(),
            program: "".chars().collect(),
            gates: map,
        }
    }
    pub fn reset(&mut self) {
        self.counter = 0;
        self.qb0 = zero();
    }
    pub fn update(&mut self, program: String) {
        self.program = program.chars().collect();
    }
    pub fn set_gates(&mut self, gates: &str) {
        self.gates = serde_json::from_str(gates).unwrap(); }
    pub fn show_gates(&self) -> String {
        serde_json::to_string_pretty(&self.gates).unwrap()
    }
    fn operate(&mut self) {
        let op = &self.program[self.counter];
        let gate = &self.gates[op];
        self.qb0 = apply(&gate, &self.qb0);
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
         write!(f, "{}", self.qb0)
         //write!(f, "({}, {})", self.qb0, self.qb1)
     }
}
