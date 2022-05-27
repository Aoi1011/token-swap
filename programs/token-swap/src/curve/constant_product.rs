use {
    crate::{
        curve::calculator::{
            map_zero_to_none, CurveCalculator, DynPack, RoundDirection, SwapWithoutFeesResult,
            TradeDirection, TradingTokenResult,
        },
        errors::SwapError,
    },
    anchor_lang::solana_program::program_error::ProgramError,
    spl_math::{checked_ceil_div::CheckedCeilDiv, precise_number::PreciseNumber},
};

/// ConstantProductCurve struct implementing CurveCalculator
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstantProductCurve;

/// The constant product swap calculation, factored out of its class for reuse.
///
/// This is guranteed to work for all values such that:
///  -1 <= swap_source_amount * swap_destination_amount <= u128::MAX
///  -1 <= source_amount <= u64::MAX
pub fn swap(
    source_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
) -> Option<SwapWithoutFeesResult> {
    let invariant = swap_source_amount.checked_mul(swap_destination_amount)?;

    let new_swap_source_amount = swap_source_amount.checked_add(source_amount)?;
    let (new_swap_destination_amount, new_swap_source_amount) =
        invariant.checked_ceil_div(new_swap_source_amount)?;

    let source_amount_swapped = new_swap_source_amount.checked_sub(swap_source_amount)?;
    let destination_amount_swapped =
        map_zero_to_none(swap_destination_amount.checked_sub(new_swap_destination_amount)?)?;

    Some(SwapWithoutFeesResult {
        source_amount_swapped,
        destination_amount_swapped,
    })
}

/// Get the amount of trading tokens for the given amount of pool tokens,
/// provided the total trading tokens and supply of pool tokens.
///
/// The constant product implementation is a simple ratio calculation for how many
/// trading tokens correspond to a certain number of pool tokens
pub fn pool_tokens_to_trading_tokens(
    pool_tokens: u128,
    pool_token_supply: u128,
    swap_token_a_amount: u128,
    swap_token_b_amount: u128,
    round_direction: RoundDirection,
) -> Option<TradingTokenResult> {
    let mut token_a_amount = pool_tokens
        .checked_mul(swap_token_a_amount)?
        .checked_div(pool_token_supply)?;
    let mut token_b_amount = pool_tokens
        .checked_mul(swap_token_b_amount)?
        .checked_div(pool_token_supply)?;
    let (token_a_amount, token_b_amount) = match round_direction {
        RoundDirection::Floor => (token_a_amount, token_b_amount),
        RoundDirection::Ceiling => {
            let token_a_remainder = pool_tokens
                .checked_mul(swap_token_a_amount)?
                .checked_rem(pool_token_supply)?;

            if token_a_remainder > 0 && token_a_amount > 0 {
                token_a_amount += 1;
            }
            let token_b_remainder = pool_tokens
                .checked_mul(swap_token_b_amount)?
                .checked_rem(pool_token_supply)?;
            if token_b_remainder > 0 && token_b_amount > 0 {
                token_b_amount += 1;
            }

            (token_a_amount, token_b_amount)
        }
    };

    Some(TradingTokenResult {
        token_a_amount,
        token_b_amount,
    })
}

/// Get the amount of pool tokens for the deposited amount of token A or B
///
/// The constant product implementation uses the Balancer formulas found at
/// in the case for 2 tokens, each weighted at 1/2.
pub fn deposit_single_token_type(
    source_amount: u128,
    swap_token_a_amount: u128,
    swap_token_b_amount: u128,
    pool_supply: u128,
    trade_direction: TradeDirection,
    round_direction: RoundDirection,
) -> Option<u128> {
    let swap_source_amount = match trade_direction {
        TradeDirection::AtoB => swap_token_a_amount,
        TradeDirection::BtoA => swap_token_b_amount,
    };
}
