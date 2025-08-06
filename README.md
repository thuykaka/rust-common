# Rust Common Library

A comprehensive utility library for Rust projects providing common functionality across different domains including mathematics, data structures, and more.

## Features

- **Math**: Mathematical utilities and operations
- **Extensible**: Easy to add new modules
- **Well-tested**: Comprehensive test coverage
- **Documented**: Full documentation with examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-common = "0.1.0"
```

## Features

The library supports conditional compilation through Cargo features to optimize for your specific needs:

### Available Features

- **`basic`** (default): Core mathematical utilities including arithmetic operations, constants, and number utilities
- **`advanced`**: Advanced mathematical functions like GCD, LCM, and power calculations
- **`statistics`**: Statistical functions and data analysis utilities
- **`trigonometry`**: Trigonometric functions and angle calculations
- **`full`**: All features including basic, advanced, statistics, and trigonometry

### Feature Usage

#### Default Installation (Basic Features Only)

```toml
[dependencies]
rust-common = "0.1.0"
```

#### Full Feature Set

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["full"] }
```

#### Selective Features

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["advanced", "statistics"] }
```

#### Command Line Installation

```bash
# Install with default features
cargo add rust-common

# Install with all features
cargo add rust-common --features full

# Install with specific features
cargo add rust-common --features "advanced trigonometry"
```

## Usage

### Basic Usage

```rust
use rust_common::math;

// Basic arithmetic
let sum = math::add(5, 3);
let difference = math::subtract(10, 4);
let product = math::multiply(6, 7);
let quotient = math::divide(20, 4);

// Mathematical constants
let pi = math::PI;
let e = math::E;

// Number utilities
let is_even = math::is_even(42);
let is_odd = math::is_odd(17);
let factorial = math::factorial(5);
```

### Using the Prelude

For convenience, you can use the prelude module:

```rust
use rust_common::prelude::*;

let result = add(2, 3);
assert_eq!(result, 5);
```

## Modules

### Math Module

The math module provides various mathematical utilities:

#### Basic Arithmetic

- `add(a, b)` - Add two numbers
- `subtract(a, b)` - Subtract two numbers
- `multiply(a, b)` - Multiply two numbers
- `divide(a, b)` - Divide two numbers

#### Mathematical Constants

- `PI` - The mathematical constant π
- `E` - The mathematical constant e
- `TAU` - The mathematical constant τ (2π)

#### Number Utilities

- `is_even(n)` - Check if a number is even
- `is_odd(n)` - Check if a number is odd
- `abs(n)` - Calculate absolute value
- `factorial(n)` - Calculate factorial

#### Advanced Functions

- `pow(base, exponent)` - Calculate power
- `gcd(a, b)` - Calculate greatest common divisor
- `lcm(a, b)` - Calculate least common multiple

## Examples

### Mathematical Operations

```rust
use rust_common::math;

// Basic calculations
let sum = math::add(10, 5);
let product = math::multiply(6, 8);
let power = math::pow(2, 10);

// Number properties
assert!(math::is_even(42));
assert!(math::is_odd(17));

// Advanced math
let gcd_result = math::gcd(48, 18); // 6
let lcm_result = math::lcm(12, 18); // 36
let factorial_result = math::factorial(5); // 120
```

### Using Constants

```rust
use rust_common::math;

let area = math::PI * 5.0 * 5.0; // Circle area
let natural_log = math::E.ln(); // Should be 1.0
```

## Development

### Running Tests

```bash
cargo test
```

### Running Documentation

```bash
cargo doc --open
```

### Building

```bash
cargo build
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [ ] Add more mathematical functions (trigonometry, statistics)
- [ ] Add data structure utilities
- [ ] Add string manipulation utilities
- [ ] Add date/time utilities
- [ ] Add file I/O utilities
- [ ] Add networking utilities
