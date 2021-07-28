use fixed::types::U64F64;
use fixed_macro::fixed;
use U64F64 as FixedPoint;

pub trait FixedPointConvert {
	fn from_fixed(v: &FixedPoint) -> Self;
	fn to_fixed(&self) -> FixedPoint;
}

const FIXED_1E12: FixedPoint = fixed!(1_000_000_000_000: U64F64);

// 12 decimals u128 conversion
impl FixedPointConvert for u128 {
	fn from_fixed(v: &FixedPoint) -> Self {
		v.saturating_mul(FIXED_1E12).to_num()
	}
	fn to_fixed(&self) -> FixedPoint {
		let v = FixedPoint::from_num(*self);
		v.saturating_div(FIXED_1E12)
	}
}
