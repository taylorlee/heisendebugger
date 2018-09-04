#![allow(dead_code)]

pub struct Complex {
    pub re: f64,
    pub imag: f64,
}
pub struct Qubit {
    pub a: Complex,
    pub b: Complex,
}
pub struct QVM {
    pub qb1: Qubit,
}

impl Complex {
    fn new() -> Complex {
        Complex {
            re: 0.0,
            imag: 0.0,
        }
    }
}

pub fn init() -> QVM {
    QVM {
        qb1: Qubit {
            a: Complex::new(),
            b: Complex::new(),
        }
    }
}
