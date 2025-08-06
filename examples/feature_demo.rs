//! Feature Demo Example
//!
//! This example demonstrates how to use different features of the rust-common library.
//!
//! Run with different features:
//! - Basic: cargo run --example feature_demo
//! - Full: cargo run --example feature_demo --features full
//! - Advanced: cargo run --example feature_demo --features advanced
//! - Statistics: cargo run --example feature_demo --features statistics
//! - Trigonometry: cargo run --example feature_demo --features trigonometry

use rust_common::math;

fn main() {
    println!("=== Rust Common Library Feature Demo ===\n");

    // Basic features (always available)
    println!("📊 Basic Features (always available):");
    println!("  add(5, 3) = {}", math::add(5, 3));
    println!("  subtract(10, 4) = {}", math::subtract(10, 4));
    println!("  multiply(6, 7) = {}", math::multiply(6, 7));
    println!("  divide(20, 4) = {}", math::divide(20, 4));
    println!("  PI = {}", math::PI);
    println!("  is_even(42) = {}", math::is_even(42));
    println!("  factorial(5) = {}", math::factorial(5));
    println!();

    // Advanced features
    #[cfg(feature = "advanced")]
    {
        println!("🚀 Advanced Features:");
        println!("  pow(2, 10) = {}", math::pow(2, 10));
        println!("  gcd(48, 18) = {}", math::gcd(48, 18));
        println!("  lcm(12, 18) = {}", math::lcm(12, 18));
        println!();
    }

    #[cfg(not(feature = "advanced"))]
    {
        println!("🚀 Advanced Features: NOT AVAILABLE (enable with --features advanced)");
        println!();
    }

    // Statistics features
    #[cfg(feature = "statistics")]
    {
        println!("📈 Statistics Features:");
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        println!("  mean({:?}) = {}", data, math::mean(&data));
        println!("  median({:?}) = {}", data, math::median(&data));
        println!("  variance({:?}) = {}", data, math::variance(&data));
        println!();
    }

    #[cfg(not(feature = "statistics"))]
    {
        println!("📈 Statistics Features: NOT AVAILABLE (enable with --features statistics)");
        println!();
    }

    // Trigonometry features
    #[cfg(feature = "trigonometry")]
    {
        println!("📐 Trigonometry Features:");
        let angle = std::f64::consts::PI / 4.0; // 45 degrees
        println!(
            "  sin({:.2}°) = {:.4}",
            angle.to_degrees(),
            math::sin(angle)
        );
        println!(
            "  cos({:.2}°) = {:.4}",
            angle.to_degrees(),
            math::cos(angle)
        );
        println!(
            "  tan({:.2}°) = {:.4}",
            angle.to_degrees(),
            math::tan(angle)
        );
        println!();
    }

    #[cfg(not(feature = "trigonometry"))]
    {
        println!("📐 Trigonometry Features: NOT AVAILABLE (enable with --features trigonometry)");
        println!();
    }

    println!("=== Demo Complete ===");
    println!();
    println!("Available features:");
    println!("  - basic: arithmetic, constants, number_utils");
    #[cfg(feature = "advanced")]
    println!("  - advanced: advanced math functions");
    #[cfg(feature = "statistics")]
    println!("  - statistics: statistical functions");
    #[cfg(feature = "trigonometry")]
    println!("  - trigonometry: trigonometric functions");
    println!();
    println!("To enable all features, run:");
    println!("  cargo run --example feature_demo --features full");
}
