use num_complex;
use serde_json;
use std::collections::BTreeMap;
use std::f32::EPSILON;
use std::fmt;
use std::iter::FromIterator;

type Complex = num_complex::Complex32;

pub fn is_zero(c: Complex) -> bool {
    c.re.abs() < EPSILON && c.im.abs() < EPSILON
}

pub fn eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}
fn pow(a: usize, b: usize) -> usize {
    let mut prod = 1;
    for _ in 0..b {
        prod = a;
    }
    prod
}
const S1: usize = 2;
const S2: usize = 2 * 2;
const S3: usize = 2 * 2 * 2;
const S4: usize = 2 * 2 * 2 * 2;

type Qstate = Vec<Complex>;
type Gate = Vec<Vec<Complex>>;

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

trait MutableState {
    fn apply(&mut self, _gates: Vec<&Gate>) {
    }
}

impl MutableState for Qstate {
    fn apply(&mut self, gates: Vec<&Gate>) {
        // apply a series of lifted gates to a quantum state
        let gate = join_gates(gates);
        let new_state = dot_product(&gate, &self);
        for (i, elem) in new_state.iter().enumerate() {
            self[i] = *elem;
        }
    }
}

fn mul(this: &Gate, other: &Gate) -> Gate {
    // matrix multiplication
    let size = this.len();
    assert!(size == other.len());
    let mut ret = other.clone();
    for i in 0..size {
        for j in 0..size {
            let mut val = C0;
            for k in 0..size {
                val += this[i][k] * other[k][j];
            }
            ret[i][j] = val;
        }
    }
    ret
}
pub const C0: Complex = Complex { re: 0.0, im: 0.0 };
pub const C1: Complex = Complex { re: 1.0, im: 0.0 };
const CI: Complex = Complex { re: 0.0, im: 1.0 };

type G1 = [[Complex; 2]; 2];
type G2 = [[Complex; 4]; 4];

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

fn vecify(a: G1) -> Gate {
    let mut outer = Vec::new();
    for row in &a {
        let mut inner = Vec::new();
        for elem in row.iter() {
            inner.push(*elem);
        }
        outer.push(inner);
    }
    outer
}
fn vecify2(a: G2) -> Gate {
    let mut outer = Vec::new();
    for row in &a {
        let mut inner = Vec::new();
        for elem in row.iter() {
            inner.push(*elem);
        }
        outer.push(inner);
    }
    outer
}
fn standard_gates() -> BTreeMap<String, Gate> {
    let mut map = BTreeMap::new();
    map.insert("x".into(), vecify([[C0, C1], [C1, C0]]));
    map.insert("z".into(), vecify([[C1, C0], [C0, -C1]]));
    map.insert("y".into(), vecify([[C0, -CI], [CI, C0]]));

    let h = 1.0 / Complex { re: 2.0, im: 0.0 }.sqrt();
    map.insert("h".into(), vecify([[h, h], [h, -h]]));
    map.insert("i1".into(), vecify(I1));
    map.insert("i2".into(), vecify2(I2));
    map.insert(
        "cnot".into(),
        vecify2([
            [C1, C0, C0, C0],
            [C0, C1, C0, C0],
            [C0, C0, C0, C1],
            [C0, C0, C1, C0],
        ]),
    );
    map.insert("swap".into(), vecify2(SWAP));
    map
}
fn zero() -> Qstate {
    let mut ret = vec![C0; S4];
    ret[0] = C1;
    ret
}

fn dot_product(gate: &Gate, state: &Qstate) -> Qstate {
    let mut ret = vec![C0; S4];
    for (i, row) in gate.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            ret[i] += item * state[j];
        }
    }
    ret
}

fn tensor_product(a: &Gate, b: &Gate) -> Gate {
    let b_dim = b.len();
    let dim = b_dim * a.len();
    let mut mat = Vec::new();
    for _ in 0..dim {
        let mut row = Vec::new();
        for _ in 0..dim {
            row.push(C0);
        }
        mat.push(row);
    }
    for (a_i, a_row) in a.iter().enumerate() {
        for (a_j, a_item) in a_row.iter().enumerate() {
            for (b_i, b_row) in b.iter().enumerate() {
                for (b_j, b_item) in b_row.iter().enumerate() {
                    let rdx = a_i * b_dim + b_i;
                    let cdx = a_j * b_dim + b_j;
                    mat[rdx][cdx] = a_item * b_item;
                }
            }
        }
    }
    mat
}
fn tp(a: &Gate, b: &Gate, c: &Gate, d: &Gate) -> Gate {
   tensor_product(&tensor_product(&tensor_product(a, b), c), d)
}

fn g01(g: &Gate) -> Gate {
    let i1 = &vecify(I1);
    tensor_product(&tensor_product(i1, i1), g)
}
fn g12(g: &Gate) -> Gate {
    let i1 = &vecify(I1);
    tensor_product(&tensor_product(i1, g), i1)
}
fn g23(g: &Gate) -> Gate {
    let i1 = &vecify(I1);
    tensor_product(&tensor_product(g, i1), i1)
}

fn join_gates(gates: Vec<&Gate>) -> Gate {
    // [a,b,c] -> a*b*c*b*a
    let size = gates.len();
    let mut ret = gates[0].clone();
    for i in 1..size {
        ret = mul(gates[i], &ret);
    }
    for i in 1..size {
        ret = mul(gates[size-1-i], &ret);
    }
    ret
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
            let i1 = &self.gates["i1"];
            let gate = &self.gates[gate];
            let lifted = match qb.as_str() {
                "0" => tp(&i1, &i1, &i1, gate),
                "1" => tp(&i1, &i1, gate, &i1),
                "2" => tp(&i1, gate, &i1, &i1),
                "3" => tp(gate, &i1, &i1, &i1),
                _ => panic!("bad target {}", qb),
            };
            self.state.apply(vec![&lifted]);
        } else if let Instruction::Double(gate, qb0, qb1) = &self.program[self.counter] {
            let gate = &self.gates[gate];
            let swap = &self.gates["swap"];
            let swap01 = &g01(swap);
            let swap12 = &g12(swap);
            let swap23 = &g23(swap);
            match (qb0.as_str(), qb1.as_str()) {
                // TODO reflexive gates allowed?
                ("0", "1") => self.state.apply(vec![swap01, &g01(&gate)]),
                ("0", "2") => self.state.apply(vec![swap12, swap01, &g01(&gate)]),
                ("0", "3") => self.state.apply(vec![swap23, swap12, swap01, &g01(&gate)]),
                ("1", "0") => {
                    self.state.apply(vec![&g01(&gate)]);
                }
                ("2", "0") => {
                    self.state.apply(vec![swap12, &g01(&gate)]);
                }
                ("3", "0") => {
                    self.state.apply(vec![swap23, swap12, &g01(&gate)]);
                }
                // later
                ("1", "2") => {
                    self.state.apply(vec![swap12, &g12(&gate)]);
                }
                ("1", "3") => {
                    self.state.apply(vec![swap23, swap12, &g12(&gate)]);
                }
                ("2", "1") => {
                    self.state.apply(vec![&g12(&gate)]);
                }
                ("2", "3") => {
                    self.state.apply(vec![swap23, &g23(&gate)]);
                }
                ("3", "1") => {
                    self.state.apply(vec![swap23, &g12(&gate)]);
                }
                ("3", "2") => {
                    self.state.apply(vec![&g23(&gate)]);
                }
                _ => panic!("bad qbits: {} {}", qb0, qb1),
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
pub fn fmt_tensor(value: Complex, n: usize) -> String {
    if is_zero(value) {
        "".into()
    } else {
        let tensors = [
            "0000", "0001", "0010", "0011", "0100", "0101", "0110", "0111", "1000", "1001", "1010",
            "1011", "1100", "1101", "1110", "1111",
        ];
        format!("|{}> {}", &tensors[n], value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn run_test(prog: String) -> QVM {
        let mut qvm = QVM::new();
        qvm.update(&prog);
        loop {
            debug_state(qvm.state.clone());
            if qvm.counter == qvm.program.len() {
                break;
            }
            qvm.next();
        }
        qvm
    }
    fn debug_state(state: Qstate) {
        println!(
            "{:#?}",
            state
                .iter()
                .enumerate()
                .map(|(i, elem)| fmt_tensor(*elem, i))
                .collect::<Vec<String>>()
        )
    }
    #[test]
    fn bell() {
        let prog = "h 0
cnot 0 1
".into();
        let qvm = run_test(prog);
        let n = 1.0 / 2.0_f32.sqrt();
        assert!(eq(qvm.state[0].re, n)); // 0000
        assert!(eq(qvm.state[3].re, n)); // 0011
    }
    #[test]
    fn swaps() {
        let prog = "x 0
cnot 0 1
swap 1 2
".into();
        let qvm = run_test(prog);
        assert!(eq(qvm.state[5].re, 1.0)); // 0101
    }
    #[test]
    fn swap02() {
        let prog = "x 0
swap 0 2
swap 1 2
x 0
swap 2 0
".into();
        let qvm = run_test(prog);
        assert!(eq(qvm.state[6].re, 1.0)); // 0110
    }
    #[test]
    fn swap23() {
        let prog = "x 3
swap 2 3
x 3
swap 3 1
x 3
swap 3 0
".into();
        let qvm = run_test(prog);
        assert!(eq(qvm.state[7].re, 1.0)); // 0111
    }

    #[test]
    fn q4() {
        let prog = "x 1
x 3
".into();
        let qvm = run_test(prog);
        assert!(eq(qvm.state[10].re, 1.0)); // 1010
    }

}
