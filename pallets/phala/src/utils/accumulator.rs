use fixed::types::U64F64 as FixedPoint;

use crate::balance_convert::{mul, FixedPointConvert};

/// Accumulator algorithm for passive reward distribution
pub struct Accumulator<B>(sp_std::marker::PhantomData<B>);

impl<B> Accumulator<B>
where
	B: sp_runtime::traits::AtLeast32BitUnsigned + Copy + FixedPointConvert,
{
	/// Clear the accumulator
	pub fn clear_pending(share: B, acc: &FixedPoint, debt: &mut B) {
		*debt = mul(share, acc)
	}

	/// Returns the current pending value in the accumulator
	pub fn pending(share: B, acc: &FixedPoint, debt: B) -> B {
		mul(share, acc) - debt
	}

	/// Distributes propotionally to all the users in the accumulator
	pub fn distribute(total_share: B, acc: &mut FixedPoint, v: B) {
		*acc += v.to_fixed() / total_share.to_fixed();
	}
}
