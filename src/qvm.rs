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

const S1: usize = 2;
const S2: usize = 2 * 2;
const S3: usize = 2 * 2 * 2;
const S4: usize = 2 * 2 * 2 * 2;

type Q1 = [Complex; S1];
type Q2 = [Complex; S2];
type Q3 = [Complex; S3];
type Q4 = [Complex; S4];


type Qstate = Q3;

type G1 = [Q1; S1];
type G2 = [Q2; S2];
type G3 = [Q3; S3];
type G4 = [Q4; S4];

#[derive(Serialize, Deserialize)]
enum Gate {
    A(Box<G1>),
    B(Box<G2>),
    C(Box<G4>),
}

struct UGate {
    size: usize,
    data: Gate,
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
    let mut ret = [C0; S3];
    ret[0] = C1;
    ret
}

fn dot_product(gate: &G3, state: &Qstate) -> Qstate {
    let mut ret = [C0; S3];
    for (i, row) in gate.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            ret[i] += item * state[j];
        }
    }
    ret
}

fn tp11(a: &G1, b: &G1) -> G2 {
    let nq = 2;
    let mut ret = [[C0; S2]; S2];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * nq + b_i;
                    let cdx = a_j * nq + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp21(a: &G2, b: &G1) -> G3 {
    let nq = 2;
    let mut ret = [[C0; S3]; S3];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * nq + b_i;
                    let cdx = a_j * nq + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp12(a: &G1, b: &G2) -> G3 {
    let nq = 4;
    let mut ret = [[C0; S3]; S3];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * nq + b_i;
                    let cdx = a_j * nq + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp13(a: &G1, b: &G3) -> G4 {
    let nq = 2;
    let mut ret = [[C0; S4]; S4];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * nq + b_i;
                    let cdx = a_j * nq + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp22(a: &G2, b: &G2) -> G4 {
    let nq = 4;
    let mut ret = [[C0; S4]; S4];
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * nq + b_i;
                    let cdx = a_j * nq + b_j;
                    ret[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    ret
}
fn tp(a: &G1, b: &G1, c: &G1) -> G3 {
    tp21(&tp11(a, b), c)
}

fn g01(g: &G2) -> G3 {
    tp12(&I1, g)
}
fn g12(g: &G2) -> G3 {
    let g3 = tp21(g, &I1);
    //let arr2: Vec<Vec<f64>> = g3.iter().map(|row| row.iter().map(|elem| elem.re).collect()).collect();
    //println!("\n{:?}", arr2);
    g3

}
//fn g23(g: &G2) -> G4 {
    //tp22(g,&I2)
//}
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
                    "0" => tp(&I1, &I1, gate),
                    "1" => tp(&I1, gate, &I1),
                    "2" => tp(gate, &I1, &I1),
                    //"3" => tp(gate, &I1, &I1, &I1),
                    _ => panic!("bad target {}", qb),
                };
                self.state.apply(&lifted);
            }
        } else if let Instruction::Double(gate, qb0, qb1) = &self.program[self.counter] {
            if let Gate::B(gate) = &self.gates[gate] {
                match (qb0.as_str(), qb1.as_str()) {
                    // TODO why did it reverse from Q2? FIXIT
                    ("0", "1") => {
                        let swap01 = &g01(&SWAP);
                        self.state.apply(swap01);
                        self.state.apply(&g01(&gate));
                        self.state.apply(swap01);
                    }
                    ("1", "0") => {
                        self.state.apply(&g01(&gate));
                    }
                    ("2", "1") => {
                        self.state.apply(&g12(&gate));
                    }
                    ("1", "2") => {
                        let swap12 = &g12(&SWAP);
                        self.state.apply(swap12);
                        self.state.apply(&g12(&gate));
                        self.state.apply(swap12);
                    }
                    //("2", "3") => {
                        //let swap23 = &g23(&SWAP);
                        //self.state.apply(swap23);
                        //self.state.apply(&g23(&gate));
                        //self.state.apply(swap23);
                    //}
                    //("3", "2") => {
                        //self.state.apply(&g23(&gate));
                    //}
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
