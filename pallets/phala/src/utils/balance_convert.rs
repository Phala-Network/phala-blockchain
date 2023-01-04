use fixed::types::U80F48;
use fixed_macro::fixed;
use U80F48 as FixedPoint;

pub trait FixedPointConvert {
	fn from_bits(bits: u128) -> Self;
	fn from_fixed(v: &FixedPoint) -> Self;
	fn to_fixed(&self) -> FixedPoint;
}

const FIXED_1E12: FixedPoint = fixed!(1_000_000_000_000: U80F48);

// 12 decimals u128 conversion
impl FixedPointConvert for u128 {
	fn from_bits(bits: u128) -> Self {
		Self::from_fixed(&FixedPoint::from_bits(bits))
	}
	fn from_fixed(v: &FixedPoint) -> Self {
		v.checked_mul(FIXED_1E12)
			.expect("U80F48 mul overflow")
			.to_num()
	}
	fn to_fixed(&self) -> FixedPoint {
		let v = FixedPoint::checked_from_num(*self).expect("U80F48 convert overflow");
		v.saturating_div(FIXED_1E12)
	}
}

pub fn mul<B>(x: B, y: &FixedPoint) -> B
where
	B: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
{
	let f = x.to_fixed().checked_mul(*y).expect("U80F48 mul overflow");
	FixedPointConvert::from_fixed(&f)
}

pub fn div<B>(x: B, y: &FixedPoint) -> B
where
	B: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
{
	let f = x.to_fixed().checked_div(*y).expect("U80F48 div failed");
	FixedPointConvert::from_fixed(&f)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn max_supply_not_overflow() {
		let f = 1_000_000_000_000000000000.to_fixed();
		assert_eq!(f.to_string(), "1000000000");
	}
}
