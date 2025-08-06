//! # Trigonometry Module
//!
//! Provides trigonometric functions.

use crate::math::constants::PI;

/// Calculates the sine of an angle (in radians)
pub fn sin(n: f64) -> f64 {
    n.sin()
}

/// Calculates the cosine of an angle (in radians)
pub fn cos(n: f64) -> f64 {
    n.cos()
}

/// Calculates the tangent of an angle (in radians)
pub fn tan(n: f64) -> f64 {
    n.tan()
}

/// Calculates the arcsine (inverse sine)
pub fn asin(n: f64) -> f64 {
    n.asin()
}

/// Calculates the arccosine (inverse cosine)
pub fn acos(n: f64) -> f64 {
    n.acos()
}

/// Calculates the arctangent (inverse tangent)
pub fn atan(n: f64) -> f64 {
    n.atan()
}

/// Calculates the arctangent of y/x
pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Converts degrees to radians
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// Converts radians to degrees
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / PI
}

/// Calculates the cosecant (1/sin)
pub fn csc(n: f64) -> f64 {
    1.0 / n.sin()
}

/// Calculates the secant (1/cos)
pub fn sec(n: f64) -> f64 {
    1.0 / n.cos()
}

/// Calculates the cotangent (1/tan)
pub fn cot(n: f64) -> f64 {
    1.0 / n.tan()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_trig() {
        assert!((sin(0.0) - 0.0).abs() < 1e-10);
        assert!((cos(0.0) - 1.0).abs() < 1e-10);
        assert!((sin(PI / 2.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_conversions() {
        assert!((deg_to_rad(180.0) - PI).abs() < 1e-10);
        assert!((rad_to_deg(PI) - 180.0).abs() < 1e-10);
    }
}
