use {
    crate::{
        curve::{
            calculator::{
                CurveCalculator, RoundDirection, SwapWithoutFeesResult, TradeDirection,
                TradingTokenResult,
            },
            constant_product::{
                deposit_single_token_type, normalized_value, pool_tokens_to_trading_tokens,
                swap, withdraw_single_token_type_exact_out,
            },
        },
        errors::SwapError,
    },
    arrayref::{array_mut_ref, array_ref},
    anchor_lang::prelude::*;
    spl_math::precise_number::PreciseNumber,
};

/// Offset curve, uses ConstantProduct under the hood, but adds an offset to 
/// one side on swap calculations
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Offset {
    /// Amount to offset the token B liquidity account
    pub token_b_offset: u64,
}

impl CurveCalculator for Offset {
    /// Constant product swap ensures token a * (token b + offset) = constant
    /// This is quaranteed to work for all values such that:
    /// -1 <= source_amount <= u64::MAX
    /// -1 <= (swap_source_amount * (swap_destination_amount + token_b_offset)) <= u128::MAX
    /// If the offset and token B are both close to u64::MAX, there can be 
    /// overflow errors with the invariant. 

}


