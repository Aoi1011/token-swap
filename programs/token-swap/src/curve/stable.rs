//! The curve.fi invariant calculator
use {
    crate::{
        curve::calculator::{
            CurveCalculator, DynPack, RoundDirection, SwapWithoutFeesResult, TradeDirection,
            TradingTokenResult,
        },
        errors::SwapError,
    },
    anchor_lang::solana_program::{
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
    },
    arrayref::{array_mut_ref, array_ref},
    spl_math::{checked_ceil_div::CheckedCeilDiv, precise_number::PreciseNumber, uint::U256},
    std::convert::TryFrom,
};

const N_COINS: u8 = 2;
const N_COINS_SQUARED: u8 = 4;
const ITERATIONS: u8 = 32;

/// Calculaous A for deriving D
///
/// Per discussion with the designer and writer of stable curves, this A is not
/// the same as the A from the whitepaper, it's actually `A * n**(n-1)`, so when
/// you set A, you actually set `A * n**(n-1)`. This is because `D**n / prod(x)`
/// loses precision with a huge A value.
///
/// There is little information to document this choice, but the original contracts
/// use this same conversion, see a comment in the code: curve-contract
fn compute_a(amp: u64) -> Option<u64> {
    amp.checked_mul(N_COINS as u64)
}

/// Returns self to the power of b
fn checked_u8_power(a: &U256, b: u8) -> Option<U256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_mul(*a)?;
    }
    Some(result)
}

/// Returns self multiplied by b
fn checked_u8_mul(a: &U256, b: u8) -> Option<U256> {
    let mut result = *a;
    for _ in 1..b {
        result = result.checked_add(*a)?;
    }
    Some(result)
}

/// StableCurve struct implementing CurveCalculator

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StableCurve {
    /// Amplifier constant
    pub amp: u64,
}

/// d = (leverage * sum_x + d_product * n_coins) * initial_d / ((leverage - 1) * initial_d + (n_coins + 1) * d_product)
fn calculate_step(initial_d: &U256, leverage: u64, sum_x: u128, d_product: &U256) -> Option<U256> {
    let leverage_mul = U256::from(leverage).checked_mul(sum_x.into())?;
    let d_p_mul = checked_u8_mul(d_product, N_COINS)?;

    let l_val = leverage_mul.checked_add(d_p_mul)?.checked_mul(*initial_d)?;

    let leverage_sub = initial_d.checked_mul((leverage.checked_sub(1)?).into())?;
    let n_coins_sum = checked_u8_mul(d_product, N_COINS.checked_add(1)?)?;

    let r_val = leverage_sub.checked_add(n_coins_sum)?;

    l_val.checked_div(r_val)
}

/// Compute stable swap invariant (D)
/// Equation
/// A * sum(x_i) * n**n + D = A * D * n**n + D**(n + 1)  / (n**n * prod(x_i))
fn compute_d(leverage: u64, amount_a: u128, amount_b: u128) -> Option<u128> {
    let amount_a_times_coins =
        checked_u8_mul(&U256::from(amount_a), N_COINS)?.checked_add(U256::one())?;
    let amount_b_times_coins =
        checked_u8_mul(&U256::from(amount_b), N_COINS)?.checked_add(U256::one())?;
    let sum_x = amount_a.checked_add(amount_b)?;
    if sum_x == 0 {
        Some(0)
    } else {
        let mut d_previous: U256;
        let mut d: U256 = sum_x.into();

        // Newton's methos to approximate D
        for _ in 0..ITERATIONS {
            let mut d_product = d;
            d_product = d_product
                .checked_mul(d)?
                .checked_div(amount_a_times_coins)?;
            d_product = d_product
                .checked_mul(d)?
                .checked_div(amount_b_times_coins)?;
            d_previous = d;

            d = calculate_step(&d, leverage, sum_x, &d_product)?;
            // Equity with the precision of 1
            if d == d_previous {
                break;
            }
        }
        u128::try_from(d).ok()
    }
}
