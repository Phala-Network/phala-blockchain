use fixed::types::{U64F64 as FixedPoint, U80F48};

pub trait FixedPointConvert {
	fn from_bits(bits: u128) -> Self;
	fn from_fixed(v: &FixedPoint) -> Self;
	fn to_fixed(&self) -> FixedPoint;
}

const PHA: u128 = 1_000_000_000_000;

// 12 decimals u128 conversion
impl FixedPointConvert for u128 {
	fn from_bits(bits: u128) -> Self {
		Self::from_fixed(&FixedPoint::from_bits(bits))
	}
	fn from_fixed(v: &FixedPoint) -> Self {
		U80F48::unwrapped_from_num(*v).unwrapped_mul_int(PHA).to_num()
	}
	fn to_fixed(&self) -> FixedPoint {
		let v = U80F48::unwrapped_from_num(*self);
		FixedPoint::unwrapped_from_num(v.unwrapped_div_int(PHA))
	}
}

pub fn mul<B>(x: B, y: &FixedPoint) -> B
where
	B: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
{
	FixedPointConvert::from_fixed(&(x.to_fixed() * y))
}

pub fn div<B>(x: B, y: &FixedPoint) -> B
where
	B: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
{
	FixedPointConvert::from_fixed(&(x.to_fixed() / y))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn max_supply_not_overflow() {
		#[allow(clippy::inconsistent_digit_grouping)]
		let balance = 1_000_000_000__000_000_000_000_u128;
		let f = balance.to_fixed();
		assert_eq!(f.to_string(), "1000000000");
		assert_eq!(u128::from_fixed(&f), balance);
	}
}
