#![allow(dead_code)]

use std::collections::HashMap;
use num_complex;
use serde_json;

type Complex = num_complex::Complex32;
type Pair = (Complex, Complex);
type Qubit = Pair;

type G1 = (Pair, Pair);

pub struct QVM {
    pub counter: usize,
    pub qb: Pair,
    pub program: Vec<char>,
    gates: HashMap<char, G1>,
}

const C0: Complex = Complex {
    re: 0.0,
    im: 0.0,
};

const C1: Complex = Complex {
    re: 1.0,
    im: 0.0,
};
const I: Complex = Complex {
    re: 0.0,
    im: 1.0,
};

fn x() -> G1 {
    (
        (C0, C1),
        (C1, C0),
    )
}
fn z() -> G1 {
    (
        (C1, C0),
        (C0, -C1),
    )
}
fn y() -> G1 {
    (
        (C0, -I),
        (I, C0),
    )
}

const ZERO: Qubit = (C1, C0);
const ONE: Qubit = (C0, C1);

fn apply(gate: G1, qb: Qubit) -> Qubit {
    let (g0, g1) = gate;
    (
        g0.0 * qb.0 + g0.1 * qb.1,
        g1.0 * qb.0 + g1.1 * qb.1,
    )

}

impl QVM {
    pub fn new() -> QVM {
        let mut map = HashMap::new();
        map.insert('x', x());
        map.insert('y', y());
        map.insert('z', z());
        QVM {
            counter: 0,
            qb: ZERO,
            program: "".chars().collect(),
            gates: map,
        }
    }
    pub fn reset(&mut self) {
        self.counter = 0;
        self.qb = ZERO;
    }
    pub fn update(&mut self, program: String) {
        self.program = program.chars().collect();
    }
    pub fn set_gates(&mut self, gates: &str) {
        self.gates = serde_json::from_str(gates).unwrap();
    }
    pub fn show_gates(&self) -> String {
        serde_json::to_string_pretty(&self.gates).unwrap()
    }
    fn operate(&mut self) { 
        let op = &self.program[self.counter];
        let gate = self.gates[op];
        self.qb = apply(gate, self.qb); 
    }
    pub fn prev(&mut self) {
        if self.counter > 0  {
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
