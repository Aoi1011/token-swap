//! Swap calculations

use {crate::errors::SwapError, spl_math::precise_number::PreciseNumber, std::fmt::Debug};

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

/// Initial amount of pool tokens for swap contract, hard-coded to something
/// "sensible" given a maximum of u128
/// Note that on Ethereum, Uniswap uses the geometric mean of all provied
/// input amounts, and Balancer uses 100 * 10 ^ 18
pub const INITIAL_SWAP_POOL_AMOUNT: u128 = 1_000_000_000;

/// HardCode the number of token types in a pool, used to calculate the
/// equivalent pool tokens for the owner trading fee.
pub const TOKENS_IN_POOL: u128 = 2;

/// Helper function for mapping to SwapError::CalculationFailure
pub fn map_zero_to_none(x: u128) -> Option<u128> {
    if x == 0 {
        None
    } else {
        Some(x)
    }
}

/// The direction of a trade, since curves can be specialized to treat each
/// token differently (by adding offsets or weights)
#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TradeDirection {
    /// Input token A, output Token B
    AtoB,
    /// Input token B, output Token A
    BtoA,
}

/// The direction to round. Used for pool token to trading token conversions to
/// avoid losing value on any deposit or withdrawal
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundDirection {
    /// Floor the value, ie. 1.9 => 1.0, 1.1 => 1.0, 1.5 => 1.0
    Floor,
    /// Ceiling the value, ie. 1.9 => 2.0, 1.1 => 2.0, 1.5 => 2.0
    Ceiling,
}

impl TradeDirection {
    /// Given a trade direction gives the opposite direction of the trade, so
    /// A to B becomes B to A, and vice versa
    pub fn opposite(&self) -> TradeDirection {
        match self {
            TradeDirection::AtoB => TradeDirection::BtoA,
            TradeDirection::BtoA => TradeDirection::AtoB,
        }
    }
}

/// Encodes all results of swapping from a source token to a destination token
#[derive(Debug, PartialEq)]
pub struct SwapWithoutFeesResult {
    /// Amount of source token swapped
    pub source_amount_swapped: u128,

    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
}

/// Encodes results of depositing both sides at once
#[derive(Debug, PartialEq)]
pub struct TradingTokenResult {
    /// Amount of token A
    pub token_a_amount: u128,
    /// Amount of token B
    pub token_b_amount: u128,
}

/// Trait for packing of trait objects, required because structs that implement
/// `Pack` cannot be used as trait objects (as `dyn Pack`)
pub trait DynPack {
    fn pack_into_slice(&self, dst: &mut [u8]);
}

/// Trait representing operations required on a swap curve
pub trait CurveCalculator: Debug + DynPack {
    /// Calculate how much destination token will be provided given an amount
    /// of course token.
    fn swap_without_fees(
        &self,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        trade_direction: TradeDirection,
    ) -> Option<SwapWithoutFeesResult>;
}
