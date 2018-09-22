use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;
use ndarray::prelude::*;

type Complex = num_complex::Complex64;
type Qubit = Array1<Complex>;
type Qubyte = (Qubit, Qubit);

type Gate = Array2<Complex>;

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
    gates: BTreeMap<String, Gate>,
}

const C0: Complex = Complex { re: 0.0, im: 0.0 };
const C1: Complex = Complex { re: 1.0, im: 0.0 };
const I: Complex = Complex { re: 0.0, im: 1.0 };

fn x() -> Gate {
    array![[C0, C1], [C1, C0]]
}
fn z() -> Gate {
    array![[C1, C0], [C0, -C1]]
}
fn y() -> Gate {
    array![[C0, -I], [I, C0]]
}
fn h() -> Gate {
    let h = 1.0 / Complex { re: 2.0, im: 0.0 }.sqrt();
    array![[h, h], [h, -h]]
}
//TODO make this work w/ tensor state
fn cnot() -> Gate {
    array![
        [C1, C0, C0, C0],
        [C0, C1, C0, C0],
        [C0, C0, C0, C1],
        [C0, C0, C1, C0],
    ]
}

fn zero() -> Qubit {
    array![C1, C0]
}
fn one() -> Qubit {
    array![C1, C0]
}

fn apply(gate: &Gate, qb: &Qubit) -> Qubit {
    qb.dot(gate)
}


//wat goin on here?
//fn tensor_product(a: Qubit, b: Qubit) -> Qubit {
    //Array::from_iter(
        //a.iter().map(|ai| {
            //b.iter().map(|bi| {
                //ai * bi
            //})
        //}
    //})
//}

impl QVM {
    pub fn new() -> QVM {
        let mut map = BTreeMap::new();
        map.insert("x".into(), x());
        map.insert("y".into(), y());
        map.insert("z".into(), z());
        map.insert("h".into(), h());
        map.insert("cnot".into(), cnot());
        QVM {
            counter: 0,
            qb: (zero(), zero()),
            program: vec![],
            gates: map,
        }
    }
    pub fn reset(&mut self) {
        self.counter = 0;
        self.qb = (zero(), zero());
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
            let qb = match qb.as_str() {
                "0" => (apply(&gate, &self.qb.0), self.qb.1.clone()),
                "1" => (self.qb.0.clone(), apply(&gate, &self.qb.1)),
                _ => panic!("bad target {}", qb),
            };
            self.qb = qb;
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
        write!(f, "[{}, {}]", self.qb.0, self.qb.1)
    }
}
