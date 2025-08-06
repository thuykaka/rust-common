//! # Statistics Module
//!
//! Provides statistical functions.

/// Calculates the mean (average) of a slice of numbers
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

/// Calculates the median of a slice of numbers
pub fn median(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = sorted.len();
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}

/// Calculates the variance of a slice of numbers
pub fn variance(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mean_val = mean(data);
    let sum_squared_diff: f64 = data.iter().map(|x| (x - mean_val).powi(2)).sum();

    sum_squared_diff / data.len() as f64
}

/// Calculates the standard deviation of a slice of numbers
pub fn std_dev(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

/// Finds the minimum value in a slice
pub fn min(data: &[f64]) -> f64 {
    data.iter().fold(f64::INFINITY, |a, &b| a.min(b))
}

/// Finds the maximum value in a slice
pub fn max(data: &[f64]) -> f64 {
    data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
}

/// Calculates the range (max - min) of a slice
pub fn range(data: &[f64]) -> f64 {
    max(data) - min(data)
}

/// Calculates the sum of a slice
pub fn sum(data: &[f64]) -> f64 {
    data.iter().sum()
}

/// Calculates the product of a slice
pub fn product(data: &[f64]) -> f64 {
    data.iter().product()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&data), 3.0);
        assert_eq!(median(&data), 3.0);
        assert_eq!(min(&data), 1.0);
        assert_eq!(max(&data), 5.0);
        assert_eq!(sum(&data), 15.0);
    }
}
