use codec::{Decode, Encode, EncodeLike, Input, Output};
use fixed::types::U64F64 as FixedPoint;
use scale_info::TypeInfo;

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

impl From<CodecFixedPoint> for FixedPoint {
	fn from(f: CodecFixedPoint) -> FixedPoint {
		f.0
	}
}

impl Encode for CodecFixedPoint {
	fn encode_to<T: Output + ?Sized>(&self, output: &mut T) {
		let bits = self.0.to_bits();
		Encode::encode_to(&bits, output);
	}
}

impl TypeInfo for CodecFixedPoint {
	type Identity = Self;

	fn type_info() -> scale_info::Type {
		scale_info::Type::builder()
			.path(scale_info::Path::new("FixedPoint", module_path!()))
			.composite(
				scale_info::build::Fields::unnamed()
					.field(|f| f.ty::<u128>().docs(&["Fixed point bits of U64F64"])),
			)
	}
}

impl EncodeLike for CodecFixedPoint {}

impl Decode for CodecFixedPoint {
	fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
		let bits: u128 = Decode::decode(input)?;
		Ok(CodecFixedPoint(FixedPoint::from_bits(bits)))
	}
}
