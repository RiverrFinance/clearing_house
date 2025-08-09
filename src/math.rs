use primitive_types::{U128, U256};

//Abeg No toch am  !!!

use crate::floatmath;

pub const _ONE_PERCENT: u64 = 100_000;
pub const FLOAT_FACTOR_EXPONENT: u128 = 9;
pub const FLOAT_PRECISION: u128 = 100_000_000_000_000_000_000; //1e20
pub const FLOAT_TO_U60X18_DIVISOR: u128 = 100;

pub fn apply_precision(value: u128, factor: u128) -> u128 {
    return mul_div(value, factor, FLOAT_PRECISION);
}

pub fn apply_exponent(value: u128, exponent: u128) -> u128 {
    //float math does not allow for x less than 0
    if value < FLOAT_PRECISION {
        return 0;
    }

    if exponent == FLOAT_PRECISION {
        return value;
    }

    // `PRBMathUD60x18.pow` accepts 2 fixed point numbers 60x18
    // we need to convert float (30 decimals) to 60x18 (18 decimals) and then back to 30 decimals
    let x = floatmath::pow(
        U256::from(float_to_u60x18(value)),
        U256::from(float_to_u60x18(exponent)),
    );

    let result = u60x18_to_float(u256_to_u128_native(x));
    return result;
}

pub fn to_precision(value: u128, factor: u128) -> u128 {
    return mul_div(value, FLOAT_PRECISION, factor);
}

/// Bound magnitude signed
///
/// sets a lower and an upper bound for the magnitude of a signed integer

pub fn bound_magnitude_signed(value: i128, min: u128, max: u128) -> i128 {
    let magnitude = bound_unsigned(value.abs() as u128, min, max);

    let sign = if value == 0 { 1 } else { value / value.abs() };
    return magnitude as i128 * sign;
}

/// Bound signed
///
/// sets the bound range for a signed integer type
pub fn bound_signed(value: i128, min: i128, max: i128) -> i128 {
    bound_below_signed(bound_above_signed(value, max), min)
}

/// Bound above signed
///
/// sets the upper bound  of value for a signed integer
pub fn bound_below_signed(value: i128, min: i128) -> i128 {
    if value < min { min } else { value }
}

/// Bound beleow signed
///
/// Sets the lower bound of value for a signed
pub fn bound_above_signed(value: i128, max: i128) -> i128 {
    if value > max { max } else { value }
}

// sets the upper bound for an unsigned integer
pub fn bound_above_unsigned(value: u128, max: u128) -> u128 {
    if value < max { value } else { max }
}

pub fn bound_below_unsigned(value: u128, min: u128) -> u128 {
    if value > min { value } else { min }
}

pub fn bound_unsigned(value: u128, min: u128, max: u128) -> u128 {
    bound_below_unsigned(bound_above_unsigned(value, max), min)
}

/// Percentage Functions
///
/// These functions  calculates percentages  

pub fn _percentage<T>(x: u64, value: T) -> T
where
    T: std::ops::Mul<Output = T> + std::ops::Div<Output = T> + From<u64>,
{
    ((T::from(x)) * value) / T::from(100 * _ONE_PERCENT)
}

pub fn diff(a: u128, b: u128) -> u128 {
    if a > b { a - b } else { b - a }
}

fn float_to_u60x18(value: u128) -> u128 {
    value / FLOAT_TO_U60X18_DIVISOR
}

fn u60x18_to_float(value: u128) -> u128 {
    value * FLOAT_TO_U60X18_DIVISOR
}

pub fn mul_div(a: u128, b: u128, c: u128) -> u128 {
    let result = (U256::from(a) * U256::from(b)) / U256::from(c);
    u256_to_u128_native(result)
}

fn u256_to_u128_native(value: U256) -> u128 {
    U128::try_from(value).map(|u| u.low_u128()).unwrap() // low_u128() returns the primitive u128
}
