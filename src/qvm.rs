
type Complex = (f64, f64);
type Qubit = (Complex, Complex);


pub fn init() -> Qubit {
    (
        (0.0, 0.0),
        (0.0, 0.0),
    )
}
