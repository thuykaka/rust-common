//! # Number Utilities Module
//!
//! Provides utility functions for number operations.
//!
//! ## Examples
//!
//! ```rust
//! use rust_common::math::number_utils;
//!
//! let is_even = number_utils::is_even(42);
//! let is_odd = number_utils::is_odd(17);
//! let factorial = number_utils::factorial(5);
//! let abs_value = number_utils::abs(-5);
//! ```

/// Checks if a number is even
///
/// # Arguments
///
/// * `n` - The number to check
///
/// # Returns
///
/// `true` if the number is even, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert!(number_utils::is_even(42));
/// assert!(!number_utils::is_even(17));
/// ```
pub fn is_even(n: i64) -> bool {
    n % 2 == 0
}

/// Checks if a number is odd
///
/// # Arguments
///
/// * `n` - The number to check
///
/// # Returns
///
/// `true` if the number is odd, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert!(number_utils::is_odd(17));
/// assert!(!number_utils::is_odd(42));
/// ```
pub fn is_odd(n: i64) -> bool {
    n % 2 != 0
}

/// Calculates the absolute value of a number
///
/// # Arguments
///
/// * `n` - The number
///
/// # Returns
///
/// The absolute value of `n`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert_eq!(number_utils::abs(-5), 5);
/// assert_eq!(number_utils::abs(5), 5);
/// ```
pub fn abs(n: i64) -> i64 {
    n.abs()
}

/// Calculates the factorial of a number
///
/// # Arguments
///
/// * `n` - The number to calculate factorial for
///
/// # Returns
///
/// The factorial of `n`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// let result = number_utils::factorial(5);
/// assert_eq!(result, 120);
/// ```
pub fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

/// Calculates the power of a number
///
/// # Arguments
///
/// * `base` - The base number
/// * `exponent` - The exponent
///
/// # Returns
///
/// The result of `base` raised to the power of `exponent`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert_eq!(number_utils::pow(2, 3), 8);
/// assert_eq!(number_utils::pow(5, 2), 25);
/// ```
pub fn pow(base: i64, exponent: u32) -> i64 {
    base.pow(exponent)
}

/// Checks if a number is prime
///
/// # Arguments
///
/// * `n` - The number to check
///
/// # Returns
///
/// `true` if the number is prime, `false` otherwise
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert!(number_utils::is_prime(2));
/// assert!(number_utils::is_prime(3));
/// assert!(number_utils::is_prime(17));
/// assert!(!number_utils::is_prime(4));
/// assert!(!number_utils::is_prime(1));
/// ```
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    let sqrt_n = (n as f64).sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

/// Calculates the next prime number after a given number
///
/// # Arguments
///
/// * `n` - The starting number
///
/// # Returns
///
/// The next prime number after `n`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert_eq!(number_utils::next_prime(10), 11);
/// assert_eq!(number_utils::next_prime(17), 19);
/// ```
pub fn next_prime(mut n: u64) -> u64 {
    if n < 2 {
        return 2;
    }
    if n == 2 {
        return 3;
    }

    n += if n % 2 == 0 { 1 } else { 2 };
    while !is_prime(n) {
        n += 2;
    }
    n
}

/// Calculates the number of digits in a number
///
/// # Arguments
///
/// * `n` - The number
///
/// # Returns
///
/// The number of digits in `n`
///
/// # Examples
///
/// ```rust
/// use rust_common::math::number_utils;
///
/// assert_eq!(number_utils::digit_count(123), 3);
/// assert_eq!(number_utils::digit_count(1000), 4);
/// assert_eq!(number_utils::digit_count(0), 1);
/// ```
pub fn digit_count(mut n: u64) -> u32 {
    if n == 0 {
        return 1;
    }

    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_even() {
        assert!(is_even(0));
        assert!(is_even(2));
        assert!(is_even(42));
        assert!(!is_even(1));
        assert!(!is_even(17));
    }

    #[test]
    fn test_is_odd() {
        assert!(is_odd(1));
        assert!(is_odd(17));
        assert!(!is_odd(0));
        assert!(!is_odd(42));
    }

    #[test]
    fn test_abs() {
        assert_eq!(abs(-5), 5);
        assert_eq!(abs(5), 5);
        assert_eq!(abs(0), 0);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(6), 720);
    }

    #[test]
    fn test_pow() {
        assert_eq!(pow(2, 3), 8);
        assert_eq!(pow(5, 2), 25);
        assert_eq!(pow(1, 10), 1);
        assert_eq!(pow(0, 5), 0);
    }

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(is_prime(11));
        assert!(is_prime(13));
        assert!(is_prime(17));
        assert!(is_prime(19));
        assert!(is_prime(23));
        assert!(is_prime(29));
        assert!(is_prime(31));
        assert!(is_prime(37));
        assert!(is_prime(41));
        assert!(is_prime(43));
        assert!(is_prime(47));
        assert!(is_prime(53));
        assert!(is_prime(59));
        assert!(is_prime(61));
        assert!(is_prime(67));
        assert!(is_prime(71));
        assert!(is_prime(73));
        assert!(is_prime(79));
        assert!(is_prime(83));
        assert!(is_prime(89));
        assert!(is_prime(97));

        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(!is_prime(4));
        assert!(!is_prime(6));
        assert!(!is_prime(8));
        assert!(!is_prime(9));
        assert!(!is_prime(10));
        assert!(!is_prime(12));
        assert!(!is_prime(14));
        assert!(!is_prime(15));
        assert!(!is_prime(16));
        assert!(!is_prime(18));
        assert!(!is_prime(20));
        assert!(!is_prime(21));
        assert!(!is_prime(22));
        assert!(!is_prime(24));
        assert!(!is_prime(25));
        assert!(!is_prime(26));
        assert!(!is_prime(27));
        assert!(!is_prime(28));
        assert!(!is_prime(30));
        assert!(!is_prime(32));
        assert!(!is_prime(33));
        assert!(!is_prime(34));
        assert!(!is_prime(35));
        assert!(!is_prime(36));
        assert!(!is_prime(38));
        assert!(!is_prime(39));
        assert!(!is_prime(40));
        assert!(!is_prime(42));
        assert!(!is_prime(44));
        assert!(!is_prime(45));
        assert!(!is_prime(46));
        assert!(!is_prime(48));
        assert!(!is_prime(49));
        assert!(!is_prime(50));
        assert!(!is_prime(51));
        assert!(!is_prime(52));
        assert!(!is_prime(54));
        assert!(!is_prime(55));
        assert!(!is_prime(56));
        assert!(!is_prime(57));
        assert!(!is_prime(58));
        assert!(!is_prime(60));
        assert!(!is_prime(62));
        assert!(!is_prime(63));
        assert!(!is_prime(64));
        assert!(!is_prime(65));
        assert!(!is_prime(66));
        assert!(!is_prime(68));
        assert!(!is_prime(69));
        assert!(!is_prime(70));
        assert!(!is_prime(72));
        assert!(!is_prime(74));
        assert!(!is_prime(75));
        assert!(!is_prime(76));
        assert!(!is_prime(77));
        assert!(!is_prime(78));
        assert!(!is_prime(80));
        assert!(!is_prime(81));
        assert!(!is_prime(82));
        assert!(!is_prime(84));
        assert!(!is_prime(85));
        assert!(!is_prime(86));
        assert!(!is_prime(87));
        assert!(!is_prime(88));
        assert!(!is_prime(90));
        assert!(!is_prime(91));
        assert!(!is_prime(92));
        assert!(!is_prime(93));
        assert!(!is_prime(94));
        assert!(!is_prime(95));
        assert!(!is_prime(96));
        assert!(!is_prime(98));
        assert!(!is_prime(99));
        assert!(!is_prime(100));
    }

    #[test]
    fn test_next_prime() {
        assert_eq!(next_prime(1), 2);
        assert_eq!(next_prime(2), 3);
        assert_eq!(next_prime(3), 5);
        assert_eq!(next_prime(4), 5);
        assert_eq!(next_prime(5), 7);
        assert_eq!(next_prime(6), 7);
        assert_eq!(next_prime(7), 11);
        assert_eq!(next_prime(8), 11);
        assert_eq!(next_prime(9), 11);
        assert_eq!(next_prime(10), 11);
        assert_eq!(next_prime(11), 13);
        assert_eq!(next_prime(12), 13);
        assert_eq!(next_prime(13), 17);
        assert_eq!(next_prime(14), 17);
        assert_eq!(next_prime(15), 17);
        assert_eq!(next_prime(16), 17);
        assert_eq!(next_prime(17), 19);
        assert_eq!(next_prime(18), 19);
        assert_eq!(next_prime(19), 23);
        assert_eq!(next_prime(20), 23);
        assert_eq!(next_prime(21), 23);
        assert_eq!(next_prime(22), 23);
        assert_eq!(next_prime(23), 29);
        assert_eq!(next_prime(24), 29);
        assert_eq!(next_prime(25), 29);
        assert_eq!(next_prime(26), 29);
        assert_eq!(next_prime(27), 29);
        assert_eq!(next_prime(28), 29);
        assert_eq!(next_prime(29), 31);
        assert_eq!(next_prime(30), 31);
        assert_eq!(next_prime(31), 37);
        assert_eq!(next_prime(32), 37);
        assert_eq!(next_prime(33), 37);
        assert_eq!(next_prime(34), 37);
        assert_eq!(next_prime(35), 37);
        assert_eq!(next_prime(36), 37);
        assert_eq!(next_prime(37), 41);
        assert_eq!(next_prime(38), 41);
        assert_eq!(next_prime(39), 41);
        assert_eq!(next_prime(40), 41);
        assert_eq!(next_prime(41), 43);
        assert_eq!(next_prime(42), 43);
        assert_eq!(next_prime(43), 47);
        assert_eq!(next_prime(44), 47);
        assert_eq!(next_prime(45), 47);
        assert_eq!(next_prime(46), 47);
        assert_eq!(next_prime(47), 53);
        assert_eq!(next_prime(48), 53);
        assert_eq!(next_prime(49), 53);
        assert_eq!(next_prime(50), 53);
        assert_eq!(next_prime(51), 53);
        assert_eq!(next_prime(52), 53);
        assert_eq!(next_prime(53), 59);
        assert_eq!(next_prime(54), 59);
        assert_eq!(next_prime(55), 59);
        assert_eq!(next_prime(56), 59);
        assert_eq!(next_prime(57), 59);
        assert_eq!(next_prime(58), 59);
        assert_eq!(next_prime(59), 61);
        assert_eq!(next_prime(60), 61);
        assert_eq!(next_prime(61), 67);
        assert_eq!(next_prime(62), 67);
        assert_eq!(next_prime(63), 67);
        assert_eq!(next_prime(64), 67);
        assert_eq!(next_prime(65), 67);
        assert_eq!(next_prime(66), 67);
        assert_eq!(next_prime(67), 71);
        assert_eq!(next_prime(68), 71);
        assert_eq!(next_prime(69), 71);
        assert_eq!(next_prime(70), 71);
        assert_eq!(next_prime(71), 73);
        assert_eq!(next_prime(72), 73);
        assert_eq!(next_prime(73), 79);
        assert_eq!(next_prime(74), 79);
        assert_eq!(next_prime(75), 79);
        assert_eq!(next_prime(76), 79);
        assert_eq!(next_prime(77), 79);
        assert_eq!(next_prime(78), 79);
        assert_eq!(next_prime(79), 83);
        assert_eq!(next_prime(80), 83);
        assert_eq!(next_prime(81), 83);
        assert_eq!(next_prime(82), 83);
        assert_eq!(next_prime(83), 89);
        assert_eq!(next_prime(84), 89);
        assert_eq!(next_prime(85), 89);
        assert_eq!(next_prime(86), 89);
        assert_eq!(next_prime(87), 89);
        assert_eq!(next_prime(88), 89);
        assert_eq!(next_prime(89), 97);
        assert_eq!(next_prime(90), 97);
        assert_eq!(next_prime(91), 97);
        assert_eq!(next_prime(92), 97);
        assert_eq!(next_prime(93), 97);
        assert_eq!(next_prime(94), 97);
        assert_eq!(next_prime(95), 97);
        assert_eq!(next_prime(96), 97);
        assert_eq!(next_prime(97), 101);
        assert_eq!(next_prime(98), 101);
        assert_eq!(next_prime(99), 101);
        assert_eq!(next_prime(100), 101);
    }

    #[test]
    fn test_digit_count() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
        assert_eq!(digit_count(999), 3);
        assert_eq!(digit_count(1000), 4);
        assert_eq!(digit_count(9999), 4);
        assert_eq!(digit_count(10000), 5);
        assert_eq!(digit_count(99999), 5);
        assert_eq!(digit_count(100000), 6);
        assert_eq!(digit_count(999999), 6);
        assert_eq!(digit_count(1000000), 7);
        assert_eq!(digit_count(9999999), 7);
        assert_eq!(digit_count(10000000), 8);
        assert_eq!(digit_count(99999999), 8);
        assert_eq!(digit_count(100000000), 9);
        assert_eq!(digit_count(999999999), 9);
        assert_eq!(digit_count(1000000000), 10);
        assert_eq!(digit_count(9999999999), 10);
        assert_eq!(digit_count(10000000000), 11);
        assert_eq!(digit_count(99999999999), 11);
        assert_eq!(digit_count(100000000000), 12);
        assert_eq!(digit_count(999999999999), 12);
        assert_eq!(digit_count(1000000000000), 13);
        assert_eq!(digit_count(9999999999999), 13);
        assert_eq!(digit_count(10000000000000), 14);
        assert_eq!(digit_count(99999999999999), 14);
        assert_eq!(digit_count(100000000000000), 15);
        assert_eq!(digit_count(999999999999999), 15);
        assert_eq!(digit_count(1000000000000000), 16);
        assert_eq!(digit_count(9999999999999999), 16);
        assert_eq!(digit_count(10000000000000000), 17);
        assert_eq!(digit_count(99999999999999999), 17);
        assert_eq!(digit_count(100000000000000000), 18);
        assert_eq!(digit_count(999999999999999999), 18);
        assert_eq!(digit_count(1000000000000000000), 19);
        assert_eq!(digit_count(9999999999999999999), 19);
        assert_eq!(digit_count(10000000000000000000), 20);
    }
}
