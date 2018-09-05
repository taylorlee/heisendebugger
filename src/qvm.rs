#![allow(dead_code)]

use complex::*;

type Pair = (Complex, Complex);
type Qubit = Pair;

type G1 = (Pair, Pair);

pub struct QVM {
    pub counter: usize,
    pub qb: Pair,
    pub program: Vec<G1>,
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
        QVM {
            counter: 0,
            qb: ZERO,
            program: vec!(x(),x(),x(),z(),z(),z()),
        }
    }
    pub fn prev(&mut self) {
        if self.counter > 0  {
            self.counter -= 1;
            let op = &self.program[self.counter];
            self.qb = apply(op, &self.qb); 
        }
    }
    pub fn next(&mut self) {
        if self.counter < self.program.len() {
            let op = &self.program[self.counter];
            self.qb = apply(op, &self.qb); 
            self.counter += 1;
        }
    }

}
