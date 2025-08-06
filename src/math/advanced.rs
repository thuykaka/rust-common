//! # Advanced Math Module
//!
//! Provides advanced mathematical functions.

/// Calculates the greatest common divisor of two numbers
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// Calculates the least common multiple of two numbers
pub fn lcm(a: u64, b: u64) -> u64 {
    (a * b) / gcd(a, b)
}

/// Calculates the square root of a number
pub fn sqrt(n: f64) -> f64 {
    n.sqrt()
}

/// Calculates the cube root of a number
pub fn cbrt(n: f64) -> f64 {
    n.cbrt()
}

/// Calculates the natural logarithm
pub fn ln(n: f64) -> f64 {
    n.ln()
}

/// Calculates the base-10 logarithm
pub fn log10(n: f64) -> f64 {
    n.log10()
}

/// Calculates the base-2 logarithm
pub fn log2(n: f64) -> f64 {
    n.log2()
}

/// Calculates the exponential function (e^x)
pub fn exp(n: f64) -> f64 {
    n.exp()
}

/// Calculates the hyperbolic sine
pub fn sinh(n: f64) -> f64 {
    n.sinh()
}

/// Calculates the hyperbolic cosine
pub fn cosh(n: f64) -> f64 {
    n.cosh()
}

/// Calculates the hyperbolic tangent
pub fn tanh(n: f64) -> f64 {
    n.tanh()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(54, 24), 6);
        assert_eq!(gcd(7, 13), 1);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(12, 18), 36);
        assert_eq!(lcm(8, 12), 24);
        assert_eq!(lcm(5, 7), 35);
    }

    #[test]
    fn test_sqrt() {
        assert!((sqrt(4.0) - 2.0).abs() < 1e-10);
        assert!((sqrt(9.0) - 3.0).abs() < 1e-10);
    }
}
