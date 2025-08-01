use primitive_types::{U256, U512};

/// float math library for exponention in float number
///
/// This is based on PRB math floating point arihtmetic library here https://github.com/PaulRBerg/prb-math/tree/main/src
/// follows an unsigned 60.18-decimal fixed point number arithmeitc for operation   

const ZERO: U256 = U256([0, 0, 0, 0]);
const UNIT: U256 = U256([1_000_000_000_000_000_000u64, 0, 0, 0]); //1e18

const UNIT_SQUARED: U256 = U256([12919594847110692864, 54210108624275221, 0, 0]);
const DOUBLE_UNIT: U256 = U256([2_000_000_000_000_000_000u64, 0, 0, 0]);
const HALF_UNIT: U256 = U256([500_000_000_000_000_000u64, 0, 0, 0]);

/// @notice Calculates the binary exponent of x using the binary fraction method.
/// @dev Has to use 192.64-bit fixed-point numbers. See https://ethereum.stackexchange.com/a/96594/24693.
/// @param x The exponent as an unsigned 192.64-bit fixed-point number.
/// @return result The result as an unsigned 60.18-decimal fixed-point number.
/// @custom:smtchecker abstract-function-nondet

pub fn exp192x64(x: U256) -> U256 {
    // Start from 0.5 in the 192.64-bit fixed-point format.
    let mut result = U256::from(1) << 191;

    // The following logic multiplies the result by $\sqrt{2^{-i}}$ when the bit at position i is 1. Key points:
    //
    // 1. Intermediate results will not overflow, as the starting point is 2^191 and all magic factors are under 2^65.
    // 2. The rationale for organizing the if statements into groups of 8 is gas savings. If the result of performing
    // a bitwise AND operation between x and any value in the array [0x80; 0x40; 0x20; 0x10; 0x08; 0x04; 0x02; 0x01] is 1,
    // we know that `x & 0xFF` is also 1.
    // Each batch: (mask, [(magic, bitmask), ...])
    const BATCHES: [(u128, [(u128, u128); 8]); 8] = [
        (
            0xFF00000000000000,
            [
                (0x16A09E667F3BCC909, 0x8000000000000000),
                (0x1306FE0A31B7152DF, 0x4000000000000000),
                (0x1172B83C7D517ADCE, 0x2000000000000000),
                (0x10B5586CF9890F62A, 0x1000000000000000),
                (0x1059B0D31585743AE, 0x800000000000000),
                (0x102C9A3E778060EE7, 0x400000000000000),
                (0x10163DA9FB33356D8, 0x200000000000000),
                (0x100B1AFA5ABCBED61, 0x100000000000000),
            ],
        ),
        (
            0xFF000000000000,
            [
                (0x10058C86DA1C09EA2, 0x80000000000000),
                (0x1002C605E2E8CEC50, 0x40000000000000),
                (0x100162F3904051FA1, 0x20000000000000),
                (0x1000B175EFFDC76BA, 0x10000000000000),
                (0x100058BA01FB9F96D, 0x8000000000000),
                (0x10002C5CC37DA9492, 0x4000000000000),
                (0x1000162E525EE0547, 0x2000000000000),
                (0x10000B17255775C04, 0x1000000000000),
            ],
        ),
        (
            0xFF0000000000,
            [
                (0x1000058B91B5BC9AE, 0x800000000000),
                (0x100002C5C89D5EC6D, 0x400000000000),
                (0x10000162E43F4F831, 0x200000000000),
                (0x100000B1721BCFC9A, 0x100000000000),
                (0x10000058B90CF1E6E, 0x80000000000),
                (0x1000002C5C863B73F, 0x40000000000),
                (0x100000162E430E5A2, 0x20000000000),
                (0x1000000B172183551, 0x10000000000),
            ],
        ),
        (
            0xFF00000000,
            [
                (0x100000058B90C0B49, 0x8000000000),
                (0x10000002C5C8601CC, 0x4000000000),
                (0x1000000162E42FFF0, 0x2000000000),
                (0x10000000B17217FBB, 0x1000000000),
                (0x1000000058B90BFCE, 0x800000000),
                (0x100000002C5C85FE3, 0x400000000),
                (0x10000000162E42FF1, 0x200000000),
                (0x100000000B17217F8, 0x100000000),
            ],
        ),
        (
            0xFF000000,
            [
                (0x10000000058B90BFC, 0x80000000),
                (0x1000000002C5C85FE, 0x40000000),
                (0x100000000162E42FF, 0x20000000),
                (0x1000000000B17217F, 0x10000000),
                (0x100000000058B90C0, 0x8000000),
                (0x10000000002C5C860, 0x4000000),
                (0x1000000000162E430, 0x2000000),
                (0x10000000000B17218, 0x1000000),
            ],
        ),
        (
            0xFF0000,
            [
                (0x1000000000058B90C, 0x800000),
                (0x100000000002C5C86, 0x400000),
                (0x10000000000162E43, 0x200000),
                (0x100000000000B1721, 0x100000),
                (0x10000000000058B91, 0x80000),
                (0x1000000000002C5C8, 0x40000),
                (0x100000000000162E4, 0x20000),
                (0x1000000000000B172, 0x10000),
            ],
        ),
        (
            0xFF00,
            [
                (0x100000000000058B9, 0x8000),
                (0x10000000000002C5D, 0x4000),
                (0x1000000000000162E, 0x2000),
                (0x10000000000000B17, 0x1000),
                (0x1000000000000058C, 0x800),
                (0x100000000000002C6, 0x400),
                (0x10000000000000163, 0x200),
                (0x100000000000000B1, 0x100),
            ],
        ),
        (
            0xFF,
            [
                (0x10000000000000059, 0x80),
                (0x1000000000000002C, 0x40),
                (0x10000000000000016, 0x20),
                (0x1000000000000000B, 0x10),
                (0x10000000000000006, 0x8),
                (0x10000000000000003, 0x4),
                (0x10000000000000001, 0x2),
                (0x10000000000000001, 0x1),
            ],
        ),
    ];

    for (batch_mask, batch) in BATCHES.iter() {
        //checks for all bateches
        if (x & U256::from(*batch_mask)) > U256::zero() {
            //if condition is satisfied,it moves tothe next
            for (magic_str, bit_mask) in batch.iter() {
                if (x & U256::from(*bit_mask)) > U256::zero() {
                    let magic = U256::from(*magic_str);
                    result = (result * magic) >> 64;
                }
            }
        }
    }

    // In the code snippet below, two operations are executed simultaneously:
    //
    // 1. The result is multiplied by $(2^n + 1)$, where $2^n$ represents the integer part, and the additional 1
    // accounts for the initial guess of 0.5. This is achieved by subtracting from 191 instead of 192.
    // 2. The result is then converted to an unsigned 60.18-decimal fixed-point format.
    //
    // The underlying logic is based on the relationship $2^{191-ip} = 2^{ip} / 2^{191}$, where $ip$ denotes the,
    // integer part, $2^n$.

    result *= UNIT; // 1e18
    // Shift right by (191 - x >> 64) to adjust for the exponent
    let shift = 191u32 - ((x >> 64).as_u32());
    result = result >> shift;
    // Scale to 60.18-decimal fixed-point format
    return result;
}

/// @notice Calculates the binary exponent of x using the binary fraction method.
///
/// @dev See https://ethereum.stackexchange.com/q/79903/24693
///
/// Requirements:
/// - x < 192e18
/// - The result must fit in UD60x18.
///
/// @param x The exponent as a UD60x18 number.
/// @return result The result as a UD60x18 number.
/// @custom:smtchecker abstract-function-nondet
pub fn exp2(x: U256) -> U256 {
    let max_x = U256::from_dec_str("192000000000000000000").unwrap() - U256::one();

    // Numbers greater than or equal to 192e18 don't fit in the 192.64-bit format.
    assert!(x <= max_x);
    let x_192x64 = (x << 64) / UNIT;

    print!("the result of exp2 in 192.64 is {}", exp192x64(x_192x64));

    // Pass x to the {Common.exp2} function, which uses the 192.64-bit fixed-point number representation.
    return exp192x64(x_192x64);
}

/// @notice Calculates the binary logarithm of x using the iterative approximation algorithm:
///
/// $$
/// log_2{x} = n + log_2{y}, \text{ where } y = x*2^{-n}, \ y \in [1, 2)
/// $$
///
/// For $0 \leq x \lt 1$, the input is inverted:
///
/// $$
/// log_2{x} = -log_2{\frac{1}{x}}
/// $$
///
/// @dev See https://en.wikipedia.org/wiki/Binary_logarithm#Iterative_approximation
///
/// Notes:
/// - Due to the lossy precision of the iterative approximation, the results are not perfectly accurate to the last decimal.
///
/// Requirements:
/// - x ≥ UNIT
///
/// @param x The UD60x18 number for which to calculate the binary logarithm.
/// @return result The binary logarithm as a UD60x18 number.
/// @custom:smtchecker abstract-function-nondet

fn log2(x: U256) -> U256 {
    assert!(x >= UNIT);

    // Calculate the integer part of the logarithm.
    let n = U256::from((x / UNIT).trailing_zeros());

    // This is the integer part of the logarithm as a UD60x18 number. The operation can't overflow because n
    // n is at most 255 and UNIT is 1e18.
    let mut result = n * UNIT;

    // Calculate $y = x * 2^{-n}$.
    let mut y = x >> n;
    // If y is the unit number, the fractional part is zero.
    if y == UNIT {
        return result;
    };

    // Calculate the fractional part via the iterative approximation.
    // The `delta >>= 1` part is equivalent to `delta /= 2`, but shifting bits is more gas efficient.

    let mut delta = HALF_UNIT;

    while delta > ZERO {
        y = (y * y) / UNIT;

        // Is y^2 >= 2e18 and so in the range [2e18, 4e18)?
        if y >= DOUBLE_UNIT {
            // Add the 2^{-m} factor to the logarithm.
            result += delta;
            // Halve y, which corresponds to z/2 in the Wikipedia article.
            y >>= 1
        }

        delta >>= 1
    }

    return result;
}

/// @notice Raises x to the power of y.
///
/// For $1 \leq x \leq \infty$, the following standard formula is used:
///
/// $$
/// x^y = 2^{log_2{x} * y}
/// $$
///
/// For $0 \leq x \lt 1$, since the unsigned {log2} is undefined, an equivalent formula is used:
///
/// $$
/// i = \frac{1}{x}
/// w = 2^{log_2{i} * y}
/// x^y = \frac{1}{w}
/// $$
///
/// @dev Notes:
/// - Refer to the notes in {log2} and {mul}.
/// - Returns `UNIT` for 0^0.
/// - It may not perform well with very small values of x. Consider using SD59x18 as an alternative.
///
/// Requirements:
/// - Refer to the requirements in {exp2}, {log2}, and {mul}.
///
/// @param x The base as a UD60x18 number.
/// @param y The exponent as a UD60x18 number.
/// @return result The result as a UD60x18 number.
/// @custom:smtchecker abstract-function-nondet

pub fn pow(x: U256, y: U256) -> U256 {
    // If both x and y are zero, the result is `UNIT`. If just x is zero, the result is always zero.
    if x == ZERO {
        return if y == ZERO { UNIT } else { return ZERO };
    }
    // If x is `UNIT`, the result is always `UNIT`.
    else if x == UNIT {
        return UNIT;
    }

    // If y is zero, the result is always `UNIT`
    if y == ZERO {
        return UNIT;
    }
    // If y is `UNIT`, the result is always x.
    else if y == UNIT {
        return x;
    }

    // If x is > UNIT, use the standard formula.

    if x > UNIT {
        return exp2(mulldiv18(log2(x), y));
    }
    // Conversely, if x < UNIT, use the equivalent formula.
    else {
        let i = UNIT_SQUARED / x;
        let w = exp2(mulldiv18(log2(i), y));
        return UNIT_SQUARED / w;
    }
}

fn mulldiv18(a: U256, b: U256) -> U256 {
    let result = a.full_mul(b) / U512::from(UNIT);

    result.try_into().unwrap()
}
