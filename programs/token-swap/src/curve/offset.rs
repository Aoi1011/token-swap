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

    fn swap_without_fees(
        &self, 
        source_amount: u128, 
        swap_source_amount: u128, 
        swap_destination_amount: u128, 
        trade_direction: TradeDirection
    ) -> Option<SwapWithoutFeesResult> {
        let token_b_offset = self.token_b_offset as u128;
        let swap_source_amount = match trade_direction {
            TradeDirection::AtoB => swap_source_amount,
            TradeDirection::BtoA => swap_source_amount.checked_add(token_b_offset)?,
        };
        let swap_destination_amount = match trade_direction {
            TradeDirection::AtoB => swap_destination_amount.checked_add(token_b_offset)?,
            TradeDirection::BtoA => swap_destination_amount,
        };
        swap(source_amount, swap_source_amount, swap_destination_amount)
    }

    /// The conversion for the offset curve needs to take into account the
    /// offset
    fn pool_tokens_to_trading_tokens(
        &self, 
        pool_tokens: u128, 
        pool_token_supply: u128, 
        swap_token_a_amount: u128, 
        swap_token_b_amount: u128, 
        round_direction: RoundDirection
    ) -> Option<TradingTokenResult> {
        
        let token_b_offset = self.token_b_offset as u128;
        pool_tokens_to_trading_tokens(
            pool_tokens, 
            pool_token_supply, 
            swap_token_a_amount, 
            swap_token_b_amount, 
            round_direction
        )
    }

    /// Get the amount of pool tokens for the given amount of token A and B, 
    /// taking into account the offset
    fn deposit_single_token_type(
        &self, 
        source_amount: u128, 
        swap_token_a_amount: u128, 
        swap_token_b_amount: u128, 
        pool_supply: u128, 
        trade_direction: TradeDirection
    ) -> Option<u128> {
      let token_b_offset = self.token_b_offset as u128;
      deposit_single_token_type(
          source_amount, 
          swap_token_a_amount, 
          swap_token_b_amount, 
          pool_supply, 
          trade_direction,
          RoundDirection::Floor,
      )
    }

    fn withdraw_single_token_type_exact_out(
        &self, 
        source_amount: u128, 
        swap_token_a_amount: u128, 
        swap_token_b_amount: u128, 
        pool_supply: u128, 
        trade_direction: TradeDirection
    ) -> Option<u128> {
        let token_b_offset = self.token_b_offset as u128;
        withdraw_single_token_type_exact_out(
            source_amount, 
            swap_token_a_amount, 
            swap_token_b_amount, 
            pool_supply, 
            trade_direction, 
            RoundDirection::Ceiling,
        )
    }

    fn validate(&self) -> Result<(), SwapError> {
       if self.token_b_offset == 0 {
            Err(SwapError::InvalidCurve.into())
       } else {
            Ok(())
       }
    }

    fn validate_supply(&self, token_a_amount: u64, token_b_amount: u64) -> Result<(), SwapError> {
        if token_a_amount == 0 {
            return Err(SwapError::EmptySupply);
        }
        Ok(())
    }

    /// Offset curves can cause arbitrage opportunities if outside users are
    /// allowed to deposit. For example, in the offset curve, if there's swap
    /// with 1 million of token A against an offset of 2 million token B,
    /// someone else can deposit 1 million A and 2 million B for LP tokens.
    /// The pool creator can then use their LP tokens to steal the 2 million B,
    fn allows_deposits(&self) -> bool {
        false
    }

    /// The normalized_value of the offset curve simply needs to add the offset to 
    /// the token B side before calculating
    fn normalized_value(
        &self, 
        swap_token_a_amount: u128, 
        swap_token_b_amount: u128
    ) -> Option<PreciseNumber> {
        let token_b_offset = self.token_b_offset as u128;
        normalized_value(
            swap_token_a_amount, 
            swap_token_b_amount.checked_add(token_b_offset)?,
        )
    }
}


