//! Base curve implementation

use crate::curve::{
    calculator::{CurveCalculator, SwapWioutFeeResult, TradeDirection},
    constant_price::ConstantPriceCurve, 
    constant_product::ConstantProductCurve,
    fees::Fees,
}

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;



