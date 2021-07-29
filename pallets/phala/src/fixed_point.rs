use codec::{Decode, Encode, EncodeLike, Input, Output};
use fixed::types::U64F64 as FixedPoint;

/// Wrapped FixedPoint (U64F64) to make scale-codec happy
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CodecFixedPoint(FixedPoint);

impl CodecFixedPoint {
	pub fn zero() -> CodecFixedPoint {
		Self(FixedPoint::from_bits(0))
	}

	pub fn get(&self) -> FixedPoint {
		self.0
	}

	pub fn get_mut(&mut self) -> &mut FixedPoint {
		&mut self.0
	}
}

impl From<FixedPoint> for CodecFixedPoint {
	fn from(f: FixedPoint) -> Self {
		Self(f)
	}
}

impl Into<FixedPoint> for CodecFixedPoint {
	fn into(self) -> FixedPoint {
		self.0
	}
}

impl Encode for CodecFixedPoint {
	fn encode_to<T: Output + ?Sized>(&self, output: &mut T) {
		let bits = self.0.to_bits();
		Encode::encode_to(&bits, output);
	}
}

impl EncodeLike for CodecFixedPoint {}

impl Decode for CodecFixedPoint {
	fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
		let bits: u128 = Decode::decode(input)?;
		Ok(CodecFixedPoint(FixedPoint::from_bits(bits)))
	}
}
