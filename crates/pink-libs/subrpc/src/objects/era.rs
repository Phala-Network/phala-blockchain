use scale::{Decode, Encode, Input, Output};

// Copied from sp_runtime to avoid a full dependency on it.

pub type Period = u64;
pub type Phase = u64;

/// An era to describe the longevity of a transaction.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Era {
    /// The transaction is valid forever. The genesis hash must be present in the signed content.
    Immortal,

    /// Period and phase are encoded:
    /// - The period of validity from the block hash found in the signing material.
    /// - The phase in the period that this transaction's lifetime begins (and, importantly,
    /// implies which block hash is included in the signature material). If the `period` is
    /// greater than 1 << 12, then it will be a factor of the times greater than 1<<12 that
    /// `period` is.
    ///
    /// When used on `FRAME`-based runtimes, `period` cannot exceed `BlockHashCount` parameter
    /// of `system` module.
    Mortal(Period, Phase),
}
impl Encode for Era {
    fn encode_to<T: Output + ?Sized>(&self, output: &mut T) {
        match self {
            Self::Immortal => output.push_byte(0),
            Self::Mortal(period, phase) => {
                let quantize_factor = (*period as u64 >> 12).max(1);
                let encoded = (period.trailing_zeros() - 1).max(1).min(15) as u16
                    | ((phase / quantize_factor) << 4) as u16;
                encoded.encode_to(output);
            }
        }
    }
}
impl scale::EncodeLike for Era {}
impl Decode for Era {
    fn decode<I: Input>(input: &mut I) -> Result<Self, scale::Error> {
        let first = input.read_byte()?;
        if first == 0 {
            Ok(Self::Immortal)
        } else {
            let encoded = first as u64 + ((input.read_byte()? as u64) << 8);
            let period = 2 << (encoded % (1 << 4));
            let quantize_factor = (period >> 12).max(1);
            let phase = (encoded >> 4) * quantize_factor;
            if period >= 4 && phase < period {
                Ok(Self::Mortal(period, phase))
            } else {
                Err("Invalid period and phase".into())
            }
        }
    }
}

/// Add Mortal{N}(u8) variants with the given indices, to describe custom encoding.
#[cfg(feature = "std")]
macro_rules! mortal_variants {
    ($variants:ident, $($index:literal),* ) => {
		$variants
		$(
			.variant(concat!(stringify!(Mortal), stringify!($index)), |v| v
				.index($index)
				.fields(scale_info::build::Fields::unnamed().field(|f| f.ty::<u8>()))
			)
		)*
    }
}

#[cfg(feature = "std")]
impl scale_info::TypeInfo for Era {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        let variants = scale_info::build::Variants::new().variant("Immortal", |v| v.index(0));

        // this is necessary since the size of the encoded Mortal variant is `u16`, conditional on
        // the value of the first byte being > 0.
        let variants = mortal_variants!(
            variants, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43,
            44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65,
            66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87,
            88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107,
            108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124,
            125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141,
            142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158,
            159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175,
            176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
            193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209,
            210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226,
            227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243,
            244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255
        );

        scale_info::Type::builder()
            .path(scale_info::Path::new("Era", module_path!()))
            .variant(variants)
    }
}
