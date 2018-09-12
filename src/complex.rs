#![allow(dead_code)]
use std::ops::{Neg, Add, Mul};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Complex {
    re: f64,
    im: f64,
}

pub const C0: Complex = Complex {
    re: 0.0,
    im: 0.0,
};
pub const C1: Complex = Complex {
    re: 1.0,
    im: 0.0,
};

impl Neg for Complex {
    type Output = Complex;
    fn neg(self) -> Complex {
        Complex {
            re: - self.re,
            im: - self.im,
        }
    }
}
impl Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}
impl Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Complex {
        Complex {
            re: re,
            im: im,
        }
    }
    pub fn repr(&self) -> String {
        format!("<{}+{}i>", self.re, self.im)
    }
    pub fn conj(&self) -> Complex {
        Complex {
            re: self.re,
            im: -self.im,
        }
    }
    pub fn norm(self) -> Complex {
        self * self.conj()
    }
}


