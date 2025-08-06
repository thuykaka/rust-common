//! # Arithmetic Module
//!
//! Provides basic arithmetic operations.
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::math::arithmetic;
//!
//! let sum = arithmetic::add(5, 3);
//! let difference = arithmetic::subtract(10, 4);
//! let product = arithmetic::multiply(6, 7);
//! let quotient = arithmetic::divide(20, 4);
//! ```

/// Adds two numbers
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The sum of `a` and `b`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::arithmetic;
///
/// let result = arithmetic::add(5, 3);
/// assert_eq!(result, 8);
/// ```
pub fn add<T: std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    a + b
}

/// Subtracts two numbers
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The difference of `a` and `b`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::arithmetic;
///
/// let result = arithmetic::subtract(10, 4);
/// assert_eq!(result, 6);
/// ```
pub fn subtract<T: std::ops::Sub<Output = T> + Copy>(a: T, b: T) -> T {
    a - b
}

/// Multiplies two numbers
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The product of `a` and `b`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::arithmetic;
///
/// let result = arithmetic::multiply(6, 7);
/// assert_eq!(result, 42);
/// ```
pub fn multiply<T: std::ops::Mul<Output = T> + Copy>(a: T, b: T) -> T {
    a * b
}

/// Divides two numbers
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The quotient of `a` and `b`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::arithmetic;
///
/// let result = arithmetic::divide(20, 4);
/// assert_eq!(result, 5);
/// ```
pub fn divide<T: std::ops::Div<Output = T> + Copy>(a: T, b: T) -> T {
    a / b
}

/// Calculates the remainder of division
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The remainder of `a` divided by `b`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::arithmetic;
///
/// let result = arithmetic::modulo(17, 5);
/// assert_eq!(result, 2);
/// ```
pub fn modulo<T: std::ops::Rem<Output = T> + Copy>(a: T, b: T) -> T {
    a % b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(2.5, 3.7), 6.2);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(subtract(1, 1), 0);
        assert_eq!(subtract(0, 5), -5);
        assert_eq!(subtract(10.5, 3.2), 7.3);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(2, 3), 6);
        assert_eq!(multiply(-2, 3), -6);
        assert_eq!(multiply(0, 5), 0);
        assert_eq!(multiply(2.5, 4.0), 10.0);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(6, 2), 3);
        assert_eq!(divide(10, 5), 2);
        assert_eq!(divide(0, 5), 0);
        assert_eq!(divide(10.0, 2.5), 4.0);
    }

    #[test]
    fn test_modulo() {
        assert_eq!(modulo(17, 5), 2);
        assert_eq!(modulo(10, 3), 1);
        assert_eq!(modulo(8, 4), 0);
    }
}
