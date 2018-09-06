#![allow(dead_code)]

use std::collections::HashMap;
use complex::*;

type Pair = (Complex, Complex);
type Qubit = Pair;

type G1 = (Pair, Pair);

pub struct QVM {
    pub counter: usize,
    pub qb: Pair,
    pub program: Vec<char>,
    gates: HashMap<char, G1>,
}

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

const ZERO: Qubit = (C1, C0);
const ONE: Qubit = (C0, C1);

fn apply(gate: &G1, qb: &Qubit) -> Qubit {
    let (g0, g1) = gate;
    (
        g0.0.mul(&qb.0).add(&g0.1.mul(&qb.1)),
        g1.0.mul(&qb.0).add(&g1.1.mul(&qb.1)),
    )

}

impl QVM {
    pub fn new() -> QVM {
        let mut gates = HashMap::new();
        gates.insert('x',x());
        gates.insert('z',z());
        QVM {
            counter: 0,
            qb: ZERO,
            program: "xxxzzz".chars().collect(),
            gates: gates,
        }
    }
    fn operate(&mut self) { 
        let op = &self.program[self.counter];
        let gate = &self.gates[op];
        self.qb = apply(gate, &self.qb); 
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
