# Features Documentation

## Overview of Features

The `rust-common` library uses Cargo features to allow users to customize which functionality gets compiled into the binary. This helps optimize binary size and compile time.

## Feature Structure

### 1. Basic Features (Default)

**Feature:** `basic` (default)

**Modules included:**

- `arithmetic` - Basic arithmetic operations
- `constants` - Mathematical constants
- `number_utils` - Number utilities

**Available functions:**

```rust
// Arithmetic
add(a, b)
subtract(a, b)
multiply(a, b)
divide(a, b)

// Constants
PI, E, TAU

// Number utilities
is_even(n)
is_odd(n)
abs(n)
factorial(n)
```

### 2. Advanced Features

**Feature:** `advanced`

**Module included:**

- `advanced` - Advanced mathematical functions

**Available functions:**

```rust
pow(base, exponent)
gcd(a, b)
lcm(a, b)
```

### 3. Statistics Features

**Feature:** `statistics`

**Module included:**

- `statistics` - Statistical functions

**Available functions:**

```rust
mean(values)
median(values)
mode(values)
variance(values)
standard_deviation(values)
```

### 4. Trigonometry Features

**Feature:** `trigonometry`

**Module included:**

- `trigonometry` - Trigonometric functions

**Available functions:**

```rust
sin(angle)
cos(angle)
tan(angle)
asin(value)
acos(value)
atan(value)
```

### 5. Full Features

**Feature:** `full`

**Includes all features:**

- `basic`
- `advanced`
- `statistics`
- `trigonometry`

## Usage

### Installation with default features

```toml
[dependencies]
rust-common = "0.1.0"
```

### Installation with all features

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["full"] }
```

### Installation with specific features

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["advanced", "statistics"] }
```

### Installation via command line

```bash
# Default (basic)
cargo add rust-common

# All features
cargo add rust-common --features full

# Specific features
cargo add rust-common --features "advanced trigonometry"
```

## Conditional Compilation

In code, features are checked using `#[cfg(feature = "feature_name")]`:

```rust
// Module only compiled when "advanced" feature is enabled
#[cfg(feature = "advanced")]
pub mod advanced;

// Function only exported when "statistics" feature is enabled
#[cfg(feature = "statistics")]
pub use statistics::*;
```

## Feature Checking

### Check if a feature is enabled

```rust
#[cfg(feature = "advanced")]
fn advanced_function() {
    // Code only runs when "advanced" feature is enabled
}

#[cfg(not(feature = "basic"))]
fn alternative_function() {
    // Code only runs when "basic" feature is NOT enabled
}
```

### Check multiple features

```rust
#[cfg(all(feature = "advanced", feature = "statistics"))]
fn combined_function() {
    // Code only runs when both "advanced" and "statistics" are enabled
}

#[cfg(any(feature = "advanced", feature = "trigonometry"))]
fn either_function() {
    // Code runs when at least one of the two features is enabled
}
```

## Benefits of Features

1. **Optimize binary size**: Only compile what's necessary
2. **Reduce compile time**: Less code = faster compilation
3. **Flexibility**: Users can choose features that fit their needs
4. **Compatibility**: Can be used in restricted environments

## Real-world Examples

### Small project (only basic calculations needed)

```toml
[dependencies]
rust-common = "0.1.0"  # Only basic features
```

### Scientific project (needs statistics and trigonometry)

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["statistics", "trigonometry"] }
```

### Full mathematical project

```toml
[dependencies]
rust-common = { version = "0.1.0", features = ["full"] }
```
