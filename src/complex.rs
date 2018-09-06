#![allow(dead_code)]
use std::ops::Neg;

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

    pub fn incr(&self) -> Complex {
        Complex {
            re: self.re + 1.0,
            im: self.im + 1.0,
        }
    }
    pub fn add(&self, other: &Complex) -> Complex {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
    pub fn mul(&self, other: &Complex) -> Complex {
        Complex {
            re: self.re * other.re,
            im: self.im * other.im,
        }
    }
    pub fn conj(&self) -> Complex {
        Complex {
            re: self.re,
            im: -self.im,
        }
    }
    pub fn norm(&self) -> Complex {
        self.mul(&self.conj())
    }
}


