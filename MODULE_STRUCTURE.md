# Module Structure Documentation

## Math Module Organization

The `math` module is organized into specialized sub-modules for easy management and extensibility:

```
src/math/
├── mod.rs           # Entry point, re-exports all sub-modules
├── arithmetic.rs    # Basic arithmetic operations
├── constants.rs     # Mathematical constants
├── number_utils.rs  # Number utilities
├── advanced.rs      # Advanced mathematical functions
├── trigonometry.rs  # Trigonometric functions
└── statistics.rs    # Statistical functions
```

## Usage Methods

### 1. Direct usage from main module

```rust
use rust_common::math;

let sum = math::add(5, 3);
let pi = math::PI;
let is_even = math::is_even(42);
```

### 2. Using specific sub-modules

```rust
use rust_common::math::arithmetic;
use rust_common::math::constants;
use rust_common::math::number_utils;

let sum = arithmetic::add(5, 3);
let pi = constants::PI;
let is_prime = number_utils::is_prime(17);
```

### 3. Import all from a sub-module

```rust
use rust_common::math::arithmetic::*;

let result = add(2, 3) + multiply(4, 5);
```

## Sub-modules

### `arithmetic.rs`

- `add(a, b)` - Add two numbers
- `subtract(a, b)` - Subtract two numbers
- `multiply(a, b)` - Multiply two numbers
- `divide(a, b)` - Divide two numbers
- `modulo(a, b)` - Get remainder

### `constants.rs`

- `PI` - Pi constant π
- `E` - Euler's number e
- `TAU` - Tau constant τ (2π)
- `PHI` - Golden ratio φ
- `SQRT_2`, `SQRT_3` - Square root of 2, 3
- `LN_2`, `LN_10` - Natural logarithm of 2, 10
- `LOG2_E`, `LOG10_E` - Base-2, Base-10 logarithm of e

### `number_utils.rs`

- `is_even(n)`, `is_odd(n)` - Check even/odd
- `abs(n)` - Absolute value
- `factorial(n)` - Factorial
- `pow(base, exponent)` - Power
- `is_prime(n)` - Check if prime
- `next_prime(n)` - Next prime number
- `digit_count(n)` - Count digits

### `advanced.rs`

- `gcd(a, b)` - Greatest common divisor
- `lcm(a, b)` - Least common multiple
- `sqrt(n)`, `cbrt(n)` - Square root, cube root
- `ln(n)`, `log10(n)`, `log2(n)` - Logarithms
- `exp(n)` - Exponential function
- `sinh(n)`, `cosh(n)`, `tanh(n)` - Hyperbolic functions

### `trigonometry.rs`

- `sin(n)`, `cos(n)`, `tan(n)` - Basic trigonometric functions
- `asin(n)`, `acos(n)`, `atan(n)` - Inverse trigonometric functions
- `deg_to_rad(deg)`, `rad_to_deg(rad)` - Angle unit conversions
- `csc(n)`, `sec(n)`, `cot(n)` - Reciprocal trigonometric functions

### `statistics.rs`

- `mean(data)` - Mean/average
- `median(data)` - Median
- `variance(data)` - Variance
- `std_dev(data)` - Standard deviation
- `min(data)`, `max(data)` - Min/max values
- `range(data)` - Range
- `sum(data)`, `product(data)` - Sum, product

## Benefits of this structure

1. **Clear organization**: Each file contains related functions
2. **Easy maintenance**: Modifying one group doesn't affect others
3. **Easy extensibility**: Adding new modules is straightforward
4. **Flexibility**: Can import individual modules or all at once
5. **Backward compatibility**: Maintains old functions in mod.rs

## Adding a new module

To add a new module (e.g., `geometry.rs`):

1. Create file `src/math/geometry.rs`
2. Add `pub mod geometry;` to `src/math/mod.rs`
3. Add `pub use geometry::*;` to `src/math/mod.rs`
4. Write geometry functions in the new file
