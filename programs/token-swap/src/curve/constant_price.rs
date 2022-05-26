use {
    crate::{
        curve::calculator::{
            map_zero_to_none, CurveCalculator, DynPack, RoundDirection, SwapWithoutFeesResult,
            TradeDirection, TradingTokenResult,
        },
        errors::SwapError,
    },
    anchor_lang::prelude::*,
    anchor_lang::solana_program::program_error::ProgramError,
    arrayref::{array_mut_ref, array_ref},
    spl_math::{checked_ceil_div::CheckedCeilDiv, precise_number::PreciseNumber, uint::U256},
};

// Single asset deposit
pub fn trading_tokens_to_pool_tokens(
    token_b_price: u64,
    source_amount: u128,
    swap_token_a_amount: u128,
    swap_token_b_amount: u128,
    pool_supply: u128,
    trade_direction: TradeDirection,
    round_direction: RoundDirection,
) -> Option<u128> {
    let token_b_price = U256::from(token_b_price);
    let given_value = match trade_direction {
        TradeDirection::AtoB => U256::from(source_amount),
        TradeDirection::BtoA => U256::from(source_amount).checked_mul(token_b_price)?,
    };
    let tatal_value = U256::from(swap_token_b_amount)
        .checked_mul(token_b_price)?
        .checked_add(U256::from(swap_token_a_amount))?;
    let pool_supply = U256::from(pool_supply);

    match round_direction {
        RoundDirection::Floor => Some(
            pool_supply
                .checked_mul(given_value)?
                .checked_div(tatal_value)?
                .as_u128(),
        ),
        RoundDirection::Ceiling => Some(
            pool_supply
                .checked_mul(given_value)?
                .checked_ceil_div(tatal_value)?
                .0
                .as_u128(),
        ),
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstantPriceCurve {
    pub token_b_price: u64,
}

impl CurveCalculator for ConstantPriceCurve {
    /// Constant price curve always returns 1:1
    fn swap_without_fees(
        &self,
        source_amount: u128,            // 100
        _swap_source_amount: u128,      // 0
        _swap_destination_amount: u128, // 0
        trade_direction: TradeDirection,
    ) -> Option<SwapWithoutFeesResult> {
        let token_b_price = self.token_b_price as u128; // 1

        let (source_amount_swapped, destination_amount_swapped) = match trade_direction {
            TradeDirection::BtoA => (source_amount, source_amount.checked_mul(token_b_price)?),
            TradeDirection::AtoB => {
                let destination_amount_swapped = source_amount.checked_div(token_b_price)?;
                let mut source_amount_swapped = source_amount;

                let remainder = source_amount_swapped.checked_rem(token_b_price)?;
                println!("remainder {:?}", remainder);
                if remainder > 0 {
                    source_amount_swapped = source_amount.checked_sub(remainder)?;
                    println!("source_amount_swapped {:?}", source_amount_swapped);
                }

                (source_amount_swapped, destination_amount_swapped)
            }
        };

        let source_amount_swapped = map_zero_to_none(source_amount_swapped)?;
        let destination_amount_swapped = map_zero_to_none(destination_amount_swapped)?;
        Some(SwapWithoutFeesResult {
            source_amount_swapped,
            destination_amount_swapped,
        })
    }

    /// Get the amount of trading tokens for the given amount of pool tokens,
    /// provided the total trading tokens and supply of pool tokens.
    /// For the constant price curve, the total value of the pool is weighted
    /// by the price of token B
    fn pool_tokens_to_trading_tokens(
        &self,
        pool_tokens: u128,
        pool_token_supply: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        round_direction: RoundDirection,
    ) -> Option<TradingTokenResult> {
        let token_b_price = self.token_b_price as u128;
        let total_value = self
            .normalized_value(swap_token_a_amount, swap_token_b_amount)?
            .to_imprecise()?;

        let (token_a_amount, token_b_amount) = match round_direction {
            RoundDirection::Floor => {
                let token_a_amount = pool_tokens
                    .checked_mul(total_value)?
                    .checked_div(pool_token_supply)?;
                let token_b_amount = pool_tokens
                    .checked_mul(total_value)?
                    .checked_div(token_b_price)?
                    .checked_div(pool_token_supply)?;
                (token_a_amount, token_b_amount)
            }
            RoundDirection::Ceiling => {
                let (token_a_amount, _) = pool_tokens
                    .checked_mul(total_value)?
                    .checked_ceil_div(pool_token_supply)?;
                let (pool_value_as_token_b, _) = pool_tokens
                    .checked_mul(total_value)?
                    .checked_ceil_div(token_b_price)?;
                let (token_b_amount, _) =
                    pool_value_as_token_b.checked_ceil_div(pool_token_supply)?;
                (token_a_amount, token_b_amount)
            }
        };

        Some(TradingTokenResult {
            token_a_amount,
            token_b_amount,
        })
    }

    /// Get the amount of pool tokens for the given amount of token A and B
    /// For the constant price curve, the total value of the pool is weighted
    /// by the price of token B
    fn deposit_single_token_type(
        &self,
        source_amount: u128,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
        pool_supply: u128,
        trade_direction: TradeDirection,
    ) -> Option<u128> {
        trading_tokens_to_pool_tokens(
            self.token_b_price,
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
        trade_direction: TradeDirection,
    ) -> Option<u128> {
        trading_tokens_to_pool_tokens(
            self.token_b_price,
            source_amount,
            swap_token_a_amount,
            swap_token_b_amount,
            pool_supply,
            trade_direction,
            RoundDirection::Ceiling,
        )
    }

    fn validate(&self) -> Result<()> {
        if self.token_b_price == 0 {
            Err(SwapError::InvalidCurve.into())
        } else {
            Ok(())
        }
    }

    fn validate_supply(&self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        if token_a_amount == 0 {
            return Err(SwapError::EmptySupply.into());
        }
        Ok(())
    }

    /// The total normalized value of the constant price curve adds the total
    /// value of the token B side to the token A side.
    ///
    /// Note that since most other curves use a multiplicative invariant, ie.
    /// `token_a * token_b`, whereas this one uses an addition,
    /// ie. `token_a + token_b`
    ///
    /// At the end, we divide by 2 to normalized the value between the two token
    /// types
    fn normalized_value(
        &self,
        swap_token_a_amount: u128,
        swap_token_b_amount: u128,
    ) -> Option<PreciseNumber> {
        let swap_token_b_value = swap_token_b_amount.checked_mul(self.token_b_price as u128)?;
        // special logic in case we're close to the limits, avoid overflow u128
        let value = if swap_token_b_value.saturating_sub(std::u64::MAX.into())
            > (std::u128::MAX.saturating_sub(std::u64::MAX.into()))
        {
            swap_token_b_value
                .checked_div(2)?
                .checked_add(swap_token_a_amount.checked_div(2)?)?
        } else {
            swap_token_a_amount
                .checked_add(swap_token_b_value)?
                .checked_div(2)?
        };
        PreciseNumber::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::calculator::{
        test::{
            check_curve_value_from_swap, check_deposit_token_conversion,
            check_withdraw_token_conversion, total_and_intermediate,
            CONVERSION_BASIS_POINTS_GURANTEE,
        },
        INITIAL_SWAP_POOL_AMOUNT,
    };
    use proptest::prelude::*;

    #[test]
    fn swap_calculation_on_price() {
        let swap_source_amount: u128 = 0;
        let swap_destination_amount: u128 = 0;
        let source_amount: u128 = 100;
        let token_b_price = 1;
        let curve = ConstantPriceCurve { token_b_price };

        let expected_result = SwapWithoutFeesResult {
            source_amount_swapped: source_amount,
            destination_amount_swapped: source_amount,
        };

        let result = curve
            .swap_without_fees(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                TradeDirection::BtoA,
            )
            .unwrap();

        // println!("BtoA {:?}", result);

        assert_eq!(result, expected_result);

        let result = curve
            .swap_without_fees(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                TradeDirection::AtoB,
            )
            .unwrap();

        println!("AtoB {:?}", result);

        assert_eq!(result, expected_result);
    }

    // #[test]
    // fn pack_flat_curve() {
    //     let token_b_price = 1_251_258;
    //     let curve = ConstantPriceCurve { token_b_price };

    //     let mut
    // }

    #[test]
    fn swap_calculation_large_price() {
        let token_b_price = 1123513_u128;
        let curve = ConstantPriceCurve {
            token_b_price: token_b_price as u64,
        };
        let token_b_amount = 500_u128;
        let token_a_amount = token_b_amount * token_b_price;
        let bad_result = curve.swap_without_fees(
            token_b_price - 1_u128,
            token_a_amount,
            token_b_amount,
            TradeDirection::AtoB,
        );
        assert!(bad_result.is_none());
        let bad_result =
            curve.swap_without_fees(1u128, token_a_amount, token_b_amount, TradeDirection::AtoB);
        assert!(bad_result.is_none());
        let result = curve
            .swap_without_fees(
                token_b_price,
                token_a_amount,
                token_b_amount,
                TradeDirection::AtoB,
            )
            .unwrap();
        assert_eq!(result.source_amount_swapped, token_b_price);
        assert_eq!(result.destination_amount_swapped, 1u128);
    }

    #[test]
    fn swap_calculation_max_min() {
        let token_b_price = u64::MAX as u128;
        let curve = ConstantPriceCurve {
            token_b_price: token_b_price as u64,
        };
        let token_b_amount = 1u128;
        let token_a_amount = token_b_price;
        let bad_result = curve.swap_without_fees(
            token_b_price - 1u128,
            token_a_amount,
            token_b_amount,
            TradeDirection::AtoB,
        );
        assert!(bad_result.is_none());
        let bad_result =
            curve.swap_without_fees(1u128, token_a_amount, token_b_amount, TradeDirection::AtoB);
        assert!(bad_result.is_none());
        let bad_result =
            curve.swap_without_fees(0u128, token_a_amount, token_b_amount, TradeDirection::AtoB);
        assert!(bad_result.is_none());
        let result = curve
            .swap_without_fees(
                token_b_price,
                token_a_amount,
                token_b_amount,
                TradeDirection::AtoB,
            )
            .unwrap();
        assert_eq!(result.source_amount_swapped, token_b_price);
        assert_eq!(result.destination_amount_swapped, 1u128);
    }

    proptest! {
        #[test]
        fn deposit_token_conversion_a_to_b(
            // in the pool token conversion calcs, we simulate trading half of
            // source_token_amount, so this needs to be at least 2
            source_token_amount in 2..u64::MAX,
            swap_source_amount in 1..u64::MAX,
            swap_destination_amount in 1..u64::MAX,
            pool_supply in INITIAL_SWAP_POOL_AMOUNT..u64::MAX as u128,
            token_b_price in 1..u64::MAX,
        ) {
            let traded_source_amount = source_token_amount / 2;
            // make sure that the trade yields at least 1 token B
            prop_assume!(traded_source_amount / token_b_price >= 1);
            // make sure there's enough tokens to get back on the other side
            prop_assume!(traded_source_amount / token_b_price <= swap_destination_amount);

            let curve = ConstantPriceCurve {
                token_b_price,
            };

            check_deposit_token_conversion(
                &curve,
                source_token_amount as u128,
                swap_source_amount as u128,
                swap_destination_amount as u128,
                TradeDirection::AtoB,
                pool_supply,
                CONVERSION_BASIS_POINTS_GURANTEE
            );
        }
    }

    proptest! {
        #[test]
        fn deposit_token_conversion_b_to_a(
            // in the pool token conversion calcs, we simulate trading half of
            // source_token_amount, so this needs to be at least 2
            source_token_amount in 2..u32::MAX,
            swap_source_amount in 1..u64::MAX,
            swap_destination_amount in 1..u64::MAX,
            pool_supply in INITIAL_SWAP_POOL_AMOUNT..u64::MAX as u128,
            token_b_price in 1..u32::MAX,
        ) {
            let curve = ConstantPriceCurve {
                token_b_price: token_b_price as u64,
            };
            let token_b_price = token_b_price as u128;
            let source_token_amount = source_token_amount as u128;
            let swap_source_amount = swap_source_amount as u128;
            let swap_destination_amount = swap_destination_amount as u128;

            // The constant price curve needs to have enough destination amount
            // on the other side to complete swap
            prop_assume!(token_b_price * source_token_amount / 2 <= swap_destination_amount);

            check_deposit_token_conversion(
                &curve,
                source_token_amount,
                swap_source_amount,
                swap_destination_amount,
                TradeDirection::BtoA,
                pool_supply,
                CONVERSION_BASIS_POINTS_GURANTEE
            );
        }
    }

    proptest! {
        #[test]
        fn withdraw_token_coversion(
            (pool_token_supply, pool_token_amount) in total_and_intermediate(),
            swap_token_a_amount in 1..u64::MAX,
            swap_token_b_amount in 1..u32::MAX,
            token_b_price in 1..u32::MAX,
        ) {
            let curve = ConstantPriceCurve {
                token_b_price: token_b_price as u64
            };
            let token_b_price = token_b_price as u128;
            let pool_token_amount = pool_token_amount as u128;
            let pool_token_supply = pool_token_supply as u128;
            let swap_token_a_amount = swap_token_a_amount as u128;
            let swap_token_b_amount = swap_token_b_amount as u128;

            let value = curve.normalized_value(swap_token_a_amount, swap_token_b_amount).unwrap();

            // make sure we trade at least one of each token
            prop_assume!(pool_token_amount * value.to_imprecise().unwrap() >= 2 * token_b_price * pool_token_supply);

            let withdraw_result = curve.pool_tokens_to_trading_tokens(
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
                RoundDirection::Floor
            ).unwrap();

            prop_assume!(withdraw_result.token_a_amount <= swap_token_a_amount);
            prop_assume!(withdraw_result.token_b_amount <= swap_token_b_amount);

            check_withdraw_token_conversion(
                &curve,
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
                TradeDirection::AtoB,
                CONVERSION_BASIS_POINTS_GURANTEE
            );

            check_withdraw_token_conversion(
                &curve,
                pool_token_amount,
                pool_token_supply,
                swap_token_a_amount,
                swap_token_b_amount,
                TradeDirection::BtoA,
                CONVERSION_BASIS_POINTS_GURANTEE
            );
        }
    }
}
