//! # Constants Module
//!
//! Provides mathematical constants.
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::math::constants;
//!
//! let pi = constants::PI;
//! let e = constants::E;
//! let tau = constants::TAU;
//! ```

/// The mathematical constant π (pi)
pub const PI: f64 = std::f64::consts::PI;

/// The mathematical constant e (Euler's number)
pub const E: f64 = std::f64::consts::E;

/// The mathematical constant τ (tau) = 2π
pub const TAU: f64 = 2.0 * PI;

/// The golden ratio φ (phi)
pub const PHI: f64 = 1.618033988749895;

/// The square root of 2
pub const SQRT_2: f64 = 1.4142135623730951;

/// The square root of 3
pub const SQRT_3: f64 = 1.7320508075688772;

/// Natural logarithm of 2
pub const LN_2: f64 = std::f64::consts::LN_2;

/// Natural logarithm of 10
pub const LN_10: f64 = std::f64::consts::LN_10;

/// Base-2 logarithm of e
pub const LOG2_E: f64 = std::f64::consts::LOG2_E;

/// Base-10 logarithm of e
pub const LOG10_E: f64 = std::f64::consts::LOG10_E;

/// The mathematical constant γ (gamma) - Euler-Mascheroni constant
pub const EULER_GAMMA: f64 = 0.5772156649015329;

/// Catalan's constant
pub const CATALAN: f64 = 0.915965594177219;

/// Apéry's constant
pub const APERY: f64 = 1.202056903159594;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi() {
        assert!((PI - 3.141592653589793).abs() < 1e-15);
    }

    #[test]
    fn test_e() {
        assert!((E - 2.718281828459045).abs() < 1e-15);
    }

    #[test]
    fn test_tau() {
        assert!((TAU - 6.283185307179586).abs() < 1e-15);
        assert!((TAU - 2.0 * PI).abs() < 1e-15);
    }

    #[test]
    fn test_phi() {
        assert!((PHI - 1.618033988749895).abs() < 1e-15);
    }

    #[test]
    fn test_sqrt_2() {
        assert!((SQRT_2 - 2.0_f64.sqrt()).abs() < 1e-15);
    }

    #[test]
    fn test_sqrt_3() {
        assert!((SQRT_3 - 3.0_f64.sqrt()).abs() < 1e-15);
    }

    #[test]
    fn test_logarithmic_constants() {
        assert!((LN_2 - 2.0_f64.ln()).abs() < 1e-15);
        assert!((LN_10 - 10.0_f64.ln()).abs() < 1e-15);
        assert!((LOG2_E - E.log2()).abs() < 1e-15);
        assert!((LOG10_E - E.log10()).abs() < 1e-15);
    }
}
