use crate::errors::SwapError;
use anchor_lang::solana_program::program_pack::{IsInitialized, Pack, Sealed};
use anchor_lang::*;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use std::convert::TryFrom;

// Encapsulates all fee information and calculations for swap operations
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fees {
    /// Trade fees are extra token amounts that are held inside the token
    /// accounts during a trade, making the value of liquidity tokens rise.
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,

    /// Owner trading fees are extra token amounts that are held inside the token
    /// accounts during a trade. with the equivalent in pool tokens minted to
    /// the owner of the program.
    /// Owner trade fee numerator
    pub owner_trade_fee_numerator: u64,
    /// Owner trade fee denominator
    pub owner_trade_fee_denominator: u64,

    /// Owner withdraw fees are extra liquidity pool token amounts that are
    /// sent to the owner on every withdrawal
    /// Owner withdraw fee numerator
    pub owner_withdraw_fee_numerator: u64,
    /// Owner withdraw fee denominator
    pub owner_withdraw_fee_denominator: u64,

    /// Host fees are a proportion of the owner trading fees, sent to an
    /// extra account provided during the trade
    /// Host trading fee numerator
    pub host_fee_numerator: u64,
    /// Host trading fee denominator
    pub host_fee_denominator: u64,
}

pub fn calculate_fee(
    token_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;

        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee)
        }
    }
}

fn validate_fraction(numerator: u64, denominator: u64) -> Result<()> {
    if denominator == 0 && numerator == 0 {
        Ok(())
    } else if numerator >= denominator {
        Err(SwapError::InvalidFee.into())
    } else {
        Ok(())
    }
}

impl Fees {
    /// Calculates the withdraw fee in pool tokens
    pub fn owner_withdraw_fee(&self, pool_tokens: u128) -> Option<u128> {
        calculate_fee(
            pool_tokens,
            u128::try_from(self.owner_withdraw_fee_numerator).ok()?,
            u128::try_from(self.owner_withdraw_fee_denominator).ok()?,
        )
    }

    /// Calculate the trading fee in pool tokens
    pub fn trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::try_from(self.trade_fee_numerator).ok()?,
            u128::try_from(self.trade_fee_denominator).ok()?,
        )
    }

    /// Calculate the owner trading fee in trading tokens
    pub fn owner_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::try_from(self.owner_trade_fee_numerator).ok()?,
            u128::try_from(self.owner_trade_fee_denominator).ok()?,
        )
    }

    /// Calculate the host fee based on the owner fee, only used in production
    /// situation where a program is hosted by multiple frontend
    pub fn host_fee(&self, owner_fee: u128) -> Option<u128> {
        calculate_fee(
            owner_fee,
            u128::try_from(self.host_fee_numerator).ok()?,
            u128::try_from(self.host_fee_denominator).ok()?,
        )
    }

    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<()> {
        validate_fraction(self.trade_fee_numerator, self.trade_fee_denominator)?;
        validate_fraction(
            self.owner_trade_fee_numerator,
            self.owner_trade_fee_denominator,
        )?;
        validate_fraction(
            self.owner_withdraw_fee_numerator,
            self.owner_withdraw_fee_denominator,
        )?;
        validate_fraction(self.host_fee_numerator, self.host_fee_denominator)?;

        Ok(())
    }
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for Fees {
    fn is_initialized(&self) -> bool {
        true
    }
}

// impl Sealed for Fees {}
// impl Pack for Fees {
//     const LEN: usize = 64;

//     fn pack_into_slice(&self, output: &mut [u8]) {
//         let output = array_mut_ref![output, 0, 64];
//         let (
//             trade_fee_numerator,
//             trade_fee_denominator,
//             owner_trade_fee_numerator,
//             owner_trade_fee_denominator,
//             owner_withdraw_fee_numerator,
//             owner_withdraw_fee_denominator,
//             host_fee_numerator,
//             host_fee_denominator,
//         ) = mut_array_refs![output, 8, 8, 8, 8, 8, 8, 8, 8];
//         *trade_fee_numerator = self.trade_fee_numerator.to_le_bytes();

//     }

//     fn unpack_from_slice(input: &[u8]) -> Result<Fees> {
//         Ok()
//     }
// }
