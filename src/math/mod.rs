//! # Math Module
//!
//! Provides mathematical utilities and operations for common use cases.
//!
//! ## Features
//!
//! - Basic arithmetic operations
//! - Mathematical constants
//! - Number utilities
//! - Statistical functions
//! - Trigonometric functions
//! - Advanced mathematical operations
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::math;
//!
//! // Basic arithmetic
//! let sum = math::add(5, 3);
//! let difference = math::subtract(10, 4);
//! let product = math::multiply(6, 7);
//! let quotient = math::divide(20, 4);
//!
//! // Mathematical constants
//! let pi = math::PI;
//! let e = math::E;
//!
//! // Number utilities
//! let is_even = math::is_even(42);
//! let is_odd = math::is_odd(17);
//! let factorial = math::factorial(5);
//!
//! // Advanced functions
//! let gcd_result = math::gcd(48, 18);
//! let lcm_result = math::lcm(12, 18);
//! ```

// Basic features - always available
pub mod arithmetic;
pub mod constants;
pub mod number_utils;

// Re-export basic features
pub use arithmetic::*;
pub use constants::*;
pub use number_utils::*;

// Advanced features - conditional compilation
#[cfg(feature = "advanced")]
pub mod advanced;

#[cfg(feature = "advanced")]
pub use advanced::*;

// Statistics features - conditional compilation
#[cfg(feature = "statistics")]
pub mod statistics;

#[cfg(feature = "statistics")]
pub use statistics::*;

// Trigonometry features - conditional compilation
#[cfg(feature = "trigonometry")]
pub mod trigonometry;

#[cfg(feature = "trigonometry")]
pub use trigonometry::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_functions() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(divide(6, 2), 3);
    }

    #[test]
    fn test_module_structure() {
        // Test that all sub-modules are accessible
        assert_eq!(arithmetic::add(2, 3), 5);
        assert_eq!(constants::PI, std::f64::consts::PI);
        assert!(number_utils::is_even(42));

        #[cfg(feature = "advanced")]
        assert_eq!(advanced::gcd(48, 18), 6);
    }
}
