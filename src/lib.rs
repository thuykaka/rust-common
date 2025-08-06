//! # Rust Common Library
//!
//! A comprehensive utility library for Rust projects providing common functionality
//! across different domains including mathematics, data structures, and more.
//!
//! ## Features
//!
//! - **Math**: Mathematical utilities and operations
//! - **Extensible**: Easy to add new modules
//! - **Well-tested**: Comprehensive test coverage
//!
//! ## Usage
//!
//! ```rust
//! use rust_common::math;
//!
//! let result = math::add(5, 3);
//! assert_eq!(result, 8);
//! ```
//!
//! ## Modules
//!
//! - `math`: Mathematical utilities and operations

// Math module is always available (basic feature)
pub mod math;

/// Re-export commonly used items for convenience
pub mod prelude {
    pub use crate::math::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_module() {
        assert_eq!(math::add(2, 3), 5);
        assert_eq!(math::subtract(5, 3), 2);
    }
}
