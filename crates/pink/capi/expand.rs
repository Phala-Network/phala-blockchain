#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod types {
    use scale::{Decode, Encode};
    use sp_core::Hasher;
    use sp_runtime::{traits::BlakeTwo256, AccountId32};
    pub type Hash = <BlakeTwo256 as Hasher>::Out;
    pub type Hashing = BlakeTwo256;
    pub type AccountId = AccountId32;
    pub type Balance = u128;
    pub type BlockNumber = u32;
    pub type Index = u64;
    pub type Address = AccountId32;
    pub type Weight = u64;
    pub use pink_extension::{HookPoint, PinkEvent};
    pub enum ExecSideEffects {
        V1 {
            pink_events: Vec<(AccountId, PinkEvent)>,
            ink_events: Vec<(AccountId, Vec<Hash>, Vec<u8>)>,
            instantiated: Vec<(AccountId, AccountId)>,
        },
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Decode for ExecSideEffects {
            fn decode<__CodecInputEdqy: ::scale::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::scale::Error> {
                match __codec_input_edqy.read_byte().map_err(|e| {
                    e.chain("Could not decode `ExecSideEffects`, failed to read variant byte")
                })? {
                    __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                        ::core::result::Result::Ok(ExecSideEffects::V1 {
                            pink_events: {
                                let __codec_res_edqy =
                                    <Vec<(AccountId, PinkEvent)> as ::scale::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `ExecSideEffects::V1::pink_events`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            ink_events: {
                                let __codec_res_edqy = < Vec < (AccountId , Vec < Hash > , Vec < u8 >) > as :: scale :: Decode > :: decode (__codec_input_edqy) ;
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `ExecSideEffects::V1::ink_events`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            instantiated: {
                                let __codec_res_edqy =
                                    <Vec<(AccountId, AccountId)> as ::scale::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(e.chain(
                                            "Could not decode `ExecSideEffects::V1::instantiated`",
                                        ))
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                    _ => ::core::result::Result::Err(<_ as ::core::convert::Into<_>>::into(
                        "Could not decode `ExecSideEffects`, variant doesn't exist",
                    )),
                }
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for ExecSideEffects {
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    ExecSideEffects::V1 {
                        ref pink_events,
                        ref ink_events,
                        ref instantiated,
                    } => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::scale::Encode::encode_to(pink_events, __codec_dest_edqy);
                        ::scale::Encode::encode_to(ink_events, __codec_dest_edqy);
                        ::scale::Encode::encode_to(instantiated, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for ExecSideEffects {}
    };
}
pub mod v1 {
    #[allow(non_upper_case_globals)]
    #[allow(non_camel_case_types)]
    #[allow(non_snake_case)]
    #[allow(dead_code)]
    mod types {
        pub const _INTTYPES_H: u32 = 1;
        pub const _FEATURES_H: u32 = 1;
        pub const _DEFAULT_SOURCE: u32 = 1;
        pub const __GLIBC_USE_ISOC2X: u32 = 0;
        pub const __USE_ISOC11: u32 = 1;
        pub const __USE_ISOC99: u32 = 1;
        pub const __USE_ISOC95: u32 = 1;
        pub const __USE_POSIX_IMPLICITLY: u32 = 1;
        pub const _POSIX_SOURCE: u32 = 1;
        pub const _POSIX_C_SOURCE: u32 = 200809;
        pub const __USE_POSIX: u32 = 1;
        pub const __USE_POSIX2: u32 = 1;
        pub const __USE_POSIX199309: u32 = 1;
        pub const __USE_POSIX199506: u32 = 1;
        pub const __USE_XOPEN2K: u32 = 1;
        pub const __USE_XOPEN2K8: u32 = 1;
        pub const _ATFILE_SOURCE: u32 = 1;
        pub const __USE_MISC: u32 = 1;
        pub const __USE_ATFILE: u32 = 1;
        pub const __USE_FORTIFY_LEVEL: u32 = 0;
        pub const __GLIBC_USE_DEPRECATED_GETS: u32 = 0;
        pub const __GLIBC_USE_DEPRECATED_SCANF: u32 = 0;
        pub const _STDC_PREDEF_H: u32 = 1;
        pub const __STDC_IEC_559__: u32 = 1;
        pub const __STDC_IEC_559_COMPLEX__: u32 = 1;
        pub const __STDC_ISO_10646__: u32 = 201706;
        pub const __GNU_LIBRARY__: u32 = 6;
        pub const __GLIBC__: u32 = 2;
        pub const __GLIBC_MINOR__: u32 = 31;
        pub const _SYS_CDEFS_H: u32 = 1;
        pub const __glibc_c99_flexarr_available: u32 = 1;
        pub const __WORDSIZE: u32 = 64;
        pub const __WORDSIZE_TIME64_COMPAT32: u32 = 1;
        pub const __SYSCALL_WORDSIZE: u32 = 64;
        pub const __LONG_DOUBLE_USES_FLOAT128: u32 = 0;
        pub const __HAVE_GENERIC_SELECTION: u32 = 1;
        pub const _STDINT_H: u32 = 1;
        pub const __GLIBC_USE_LIB_EXT2: u32 = 0;
        pub const __GLIBC_USE_IEC_60559_BFP_EXT: u32 = 0;
        pub const __GLIBC_USE_IEC_60559_BFP_EXT_C2X: u32 = 0;
        pub const __GLIBC_USE_IEC_60559_FUNCS_EXT: u32 = 0;
        pub const __GLIBC_USE_IEC_60559_FUNCS_EXT_C2X: u32 = 0;
        pub const __GLIBC_USE_IEC_60559_TYPES_EXT: u32 = 0;
        pub const _BITS_TYPES_H: u32 = 1;
        pub const __TIMESIZE: u32 = 64;
        pub const _BITS_TYPESIZES_H: u32 = 1;
        pub const __OFF_T_MATCHES_OFF64_T: u32 = 1;
        pub const __INO_T_MATCHES_INO64_T: u32 = 1;
        pub const __RLIM_T_MATCHES_RLIM64_T: u32 = 1;
        pub const __STATFS_MATCHES_STATFS64: u32 = 1;
        pub const __FD_SETSIZE: u32 = 1024;
        pub const _BITS_TIME64_H: u32 = 1;
        pub const _BITS_WCHAR_H: u32 = 1;
        pub const _BITS_STDINT_INTN_H: u32 = 1;
        pub const _BITS_STDINT_UINTN_H: u32 = 1;
        pub const INT8_MIN: i32 = -128;
        pub const INT16_MIN: i32 = -32768;
        pub const INT32_MIN: i32 = -2147483648;
        pub const INT8_MAX: u32 = 127;
        pub const INT16_MAX: u32 = 32767;
        pub const INT32_MAX: u32 = 2147483647;
        pub const UINT8_MAX: u32 = 255;
        pub const UINT16_MAX: u32 = 65535;
        pub const UINT32_MAX: u32 = 4294967295;
        pub const INT_LEAST8_MIN: i32 = -128;
        pub const INT_LEAST16_MIN: i32 = -32768;
        pub const INT_LEAST32_MIN: i32 = -2147483648;
        pub const INT_LEAST8_MAX: u32 = 127;
        pub const INT_LEAST16_MAX: u32 = 32767;
        pub const INT_LEAST32_MAX: u32 = 2147483647;
        pub const UINT_LEAST8_MAX: u32 = 255;
        pub const UINT_LEAST16_MAX: u32 = 65535;
        pub const UINT_LEAST32_MAX: u32 = 4294967295;
        pub const INT_FAST8_MIN: i32 = -128;
        pub const INT_FAST16_MIN: i64 = -9223372036854775808;
        pub const INT_FAST32_MIN: i64 = -9223372036854775808;
        pub const INT_FAST8_MAX: u32 = 127;
        pub const INT_FAST16_MAX: u64 = 9223372036854775807;
        pub const INT_FAST32_MAX: u64 = 9223372036854775807;
        pub const UINT_FAST8_MAX: u32 = 255;
        pub const UINT_FAST16_MAX: i32 = -1;
        pub const UINT_FAST32_MAX: i32 = -1;
        pub const INTPTR_MIN: i64 = -9223372036854775808;
        pub const INTPTR_MAX: u64 = 9223372036854775807;
        pub const UINTPTR_MAX: i32 = -1;
        pub const PTRDIFF_MIN: i64 = -9223372036854775808;
        pub const PTRDIFF_MAX: u64 = 9223372036854775807;
        pub const SIG_ATOMIC_MIN: i32 = -2147483648;
        pub const SIG_ATOMIC_MAX: u32 = 2147483647;
        pub const SIZE_MAX: i32 = -1;
        pub const WINT_MIN: u32 = 0;
        pub const WINT_MAX: u32 = 4294967295;
        pub const ____gwchar_t_defined: u32 = 1;
        pub const __PRI64_PREFIX: &[u8; 2usize] = b"l\0";
        pub const __PRIPTR_PREFIX: &[u8; 2usize] = b"l\0";
        pub const PRId8: &[u8; 2usize] = b"d\0";
        pub const PRId16: &[u8; 2usize] = b"d\0";
        pub const PRId32: &[u8; 2usize] = b"d\0";
        pub const PRId64: &[u8; 3usize] = b"ld\0";
        pub const PRIdLEAST8: &[u8; 2usize] = b"d\0";
        pub const PRIdLEAST16: &[u8; 2usize] = b"d\0";
        pub const PRIdLEAST32: &[u8; 2usize] = b"d\0";
        pub const PRIdLEAST64: &[u8; 3usize] = b"ld\0";
        pub const PRIdFAST8: &[u8; 2usize] = b"d\0";
        pub const PRIdFAST16: &[u8; 3usize] = b"ld\0";
        pub const PRIdFAST32: &[u8; 3usize] = b"ld\0";
        pub const PRIdFAST64: &[u8; 3usize] = b"ld\0";
        pub const PRIi8: &[u8; 2usize] = b"i\0";
        pub const PRIi16: &[u8; 2usize] = b"i\0";
        pub const PRIi32: &[u8; 2usize] = b"i\0";
        pub const PRIi64: &[u8; 3usize] = b"li\0";
        pub const PRIiLEAST8: &[u8; 2usize] = b"i\0";
        pub const PRIiLEAST16: &[u8; 2usize] = b"i\0";
        pub const PRIiLEAST32: &[u8; 2usize] = b"i\0";
        pub const PRIiLEAST64: &[u8; 3usize] = b"li\0";
        pub const PRIiFAST8: &[u8; 2usize] = b"i\0";
        pub const PRIiFAST16: &[u8; 3usize] = b"li\0";
        pub const PRIiFAST32: &[u8; 3usize] = b"li\0";
        pub const PRIiFAST64: &[u8; 3usize] = b"li\0";
        pub const PRIo8: &[u8; 2usize] = b"o\0";
        pub const PRIo16: &[u8; 2usize] = b"o\0";
        pub const PRIo32: &[u8; 2usize] = b"o\0";
        pub const PRIo64: &[u8; 3usize] = b"lo\0";
        pub const PRIoLEAST8: &[u8; 2usize] = b"o\0";
        pub const PRIoLEAST16: &[u8; 2usize] = b"o\0";
        pub const PRIoLEAST32: &[u8; 2usize] = b"o\0";
        pub const PRIoLEAST64: &[u8; 3usize] = b"lo\0";
        pub const PRIoFAST8: &[u8; 2usize] = b"o\0";
        pub const PRIoFAST16: &[u8; 3usize] = b"lo\0";
        pub const PRIoFAST32: &[u8; 3usize] = b"lo\0";
        pub const PRIoFAST64: &[u8; 3usize] = b"lo\0";
        pub const PRIu8: &[u8; 2usize] = b"u\0";
        pub const PRIu16: &[u8; 2usize] = b"u\0";
        pub const PRIu32: &[u8; 2usize] = b"u\0";
        pub const PRIu64: &[u8; 3usize] = b"lu\0";
        pub const PRIuLEAST8: &[u8; 2usize] = b"u\0";
        pub const PRIuLEAST16: &[u8; 2usize] = b"u\0";
        pub const PRIuLEAST32: &[u8; 2usize] = b"u\0";
        pub const PRIuLEAST64: &[u8; 3usize] = b"lu\0";
        pub const PRIuFAST8: &[u8; 2usize] = b"u\0";
        pub const PRIuFAST16: &[u8; 3usize] = b"lu\0";
        pub const PRIuFAST32: &[u8; 3usize] = b"lu\0";
        pub const PRIuFAST64: &[u8; 3usize] = b"lu\0";
        pub const PRIx8: &[u8; 2usize] = b"x\0";
        pub const PRIx16: &[u8; 2usize] = b"x\0";
        pub const PRIx32: &[u8; 2usize] = b"x\0";
        pub const PRIx64: &[u8; 3usize] = b"lx\0";
        pub const PRIxLEAST8: &[u8; 2usize] = b"x\0";
        pub const PRIxLEAST16: &[u8; 2usize] = b"x\0";
        pub const PRIxLEAST32: &[u8; 2usize] = b"x\0";
        pub const PRIxLEAST64: &[u8; 3usize] = b"lx\0";
        pub const PRIxFAST8: &[u8; 2usize] = b"x\0";
        pub const PRIxFAST16: &[u8; 3usize] = b"lx\0";
        pub const PRIxFAST32: &[u8; 3usize] = b"lx\0";
        pub const PRIxFAST64: &[u8; 3usize] = b"lx\0";
        pub const PRIX8: &[u8; 2usize] = b"X\0";
        pub const PRIX16: &[u8; 2usize] = b"X\0";
        pub const PRIX32: &[u8; 2usize] = b"X\0";
        pub const PRIX64: &[u8; 3usize] = b"lX\0";
        pub const PRIXLEAST8: &[u8; 2usize] = b"X\0";
        pub const PRIXLEAST16: &[u8; 2usize] = b"X\0";
        pub const PRIXLEAST32: &[u8; 2usize] = b"X\0";
        pub const PRIXLEAST64: &[u8; 3usize] = b"lX\0";
        pub const PRIXFAST8: &[u8; 2usize] = b"X\0";
        pub const PRIXFAST16: &[u8; 3usize] = b"lX\0";
        pub const PRIXFAST32: &[u8; 3usize] = b"lX\0";
        pub const PRIXFAST64: &[u8; 3usize] = b"lX\0";
        pub const PRIdMAX: &[u8; 3usize] = b"ld\0";
        pub const PRIiMAX: &[u8; 3usize] = b"li\0";
        pub const PRIoMAX: &[u8; 3usize] = b"lo\0";
        pub const PRIuMAX: &[u8; 3usize] = b"lu\0";
        pub const PRIxMAX: &[u8; 3usize] = b"lx\0";
        pub const PRIXMAX: &[u8; 3usize] = b"lX\0";
        pub const PRIdPTR: &[u8; 3usize] = b"ld\0";
        pub const PRIiPTR: &[u8; 3usize] = b"li\0";
        pub const PRIoPTR: &[u8; 3usize] = b"lo\0";
        pub const PRIuPTR: &[u8; 3usize] = b"lu\0";
        pub const PRIxPTR: &[u8; 3usize] = b"lx\0";
        pub const PRIXPTR: &[u8; 3usize] = b"lX\0";
        pub const SCNd8: &[u8; 4usize] = b"hhd\0";
        pub const SCNd16: &[u8; 3usize] = b"hd\0";
        pub const SCNd32: &[u8; 2usize] = b"d\0";
        pub const SCNd64: &[u8; 3usize] = b"ld\0";
        pub const SCNdLEAST8: &[u8; 4usize] = b"hhd\0";
        pub const SCNdLEAST16: &[u8; 3usize] = b"hd\0";
        pub const SCNdLEAST32: &[u8; 2usize] = b"d\0";
        pub const SCNdLEAST64: &[u8; 3usize] = b"ld\0";
        pub const SCNdFAST8: &[u8; 4usize] = b"hhd\0";
        pub const SCNdFAST16: &[u8; 3usize] = b"ld\0";
        pub const SCNdFAST32: &[u8; 3usize] = b"ld\0";
        pub const SCNdFAST64: &[u8; 3usize] = b"ld\0";
        pub const SCNi8: &[u8; 4usize] = b"hhi\0";
        pub const SCNi16: &[u8; 3usize] = b"hi\0";
        pub const SCNi32: &[u8; 2usize] = b"i\0";
        pub const SCNi64: &[u8; 3usize] = b"li\0";
        pub const SCNiLEAST8: &[u8; 4usize] = b"hhi\0";
        pub const SCNiLEAST16: &[u8; 3usize] = b"hi\0";
        pub const SCNiLEAST32: &[u8; 2usize] = b"i\0";
        pub const SCNiLEAST64: &[u8; 3usize] = b"li\0";
        pub const SCNiFAST8: &[u8; 4usize] = b"hhi\0";
        pub const SCNiFAST16: &[u8; 3usize] = b"li\0";
        pub const SCNiFAST32: &[u8; 3usize] = b"li\0";
        pub const SCNiFAST64: &[u8; 3usize] = b"li\0";
        pub const SCNu8: &[u8; 4usize] = b"hhu\0";
        pub const SCNu16: &[u8; 3usize] = b"hu\0";
        pub const SCNu32: &[u8; 2usize] = b"u\0";
        pub const SCNu64: &[u8; 3usize] = b"lu\0";
        pub const SCNuLEAST8: &[u8; 4usize] = b"hhu\0";
        pub const SCNuLEAST16: &[u8; 3usize] = b"hu\0";
        pub const SCNuLEAST32: &[u8; 2usize] = b"u\0";
        pub const SCNuLEAST64: &[u8; 3usize] = b"lu\0";
        pub const SCNuFAST8: &[u8; 4usize] = b"hhu\0";
        pub const SCNuFAST16: &[u8; 3usize] = b"lu\0";
        pub const SCNuFAST32: &[u8; 3usize] = b"lu\0";
        pub const SCNuFAST64: &[u8; 3usize] = b"lu\0";
        pub const SCNo8: &[u8; 4usize] = b"hho\0";
        pub const SCNo16: &[u8; 3usize] = b"ho\0";
        pub const SCNo32: &[u8; 2usize] = b"o\0";
        pub const SCNo64: &[u8; 3usize] = b"lo\0";
        pub const SCNoLEAST8: &[u8; 4usize] = b"hho\0";
        pub const SCNoLEAST16: &[u8; 3usize] = b"ho\0";
        pub const SCNoLEAST32: &[u8; 2usize] = b"o\0";
        pub const SCNoLEAST64: &[u8; 3usize] = b"lo\0";
        pub const SCNoFAST8: &[u8; 4usize] = b"hho\0";
        pub const SCNoFAST16: &[u8; 3usize] = b"lo\0";
        pub const SCNoFAST32: &[u8; 3usize] = b"lo\0";
        pub const SCNoFAST64: &[u8; 3usize] = b"lo\0";
        pub const SCNx8: &[u8; 4usize] = b"hhx\0";
        pub const SCNx16: &[u8; 3usize] = b"hx\0";
        pub const SCNx32: &[u8; 2usize] = b"x\0";
        pub const SCNx64: &[u8; 3usize] = b"lx\0";
        pub const SCNxLEAST8: &[u8; 4usize] = b"hhx\0";
        pub const SCNxLEAST16: &[u8; 3usize] = b"hx\0";
        pub const SCNxLEAST32: &[u8; 2usize] = b"x\0";
        pub const SCNxLEAST64: &[u8; 3usize] = b"lx\0";
        pub const SCNxFAST8: &[u8; 4usize] = b"hhx\0";
        pub const SCNxFAST16: &[u8; 3usize] = b"lx\0";
        pub const SCNxFAST32: &[u8; 3usize] = b"lx\0";
        pub const SCNxFAST64: &[u8; 3usize] = b"lx\0";
        pub const SCNdMAX: &[u8; 3usize] = b"ld\0";
        pub const SCNiMAX: &[u8; 3usize] = b"li\0";
        pub const SCNoMAX: &[u8; 3usize] = b"lo\0";
        pub const SCNuMAX: &[u8; 3usize] = b"lu\0";
        pub const SCNxMAX: &[u8; 3usize] = b"lx\0";
        pub const SCNdPTR: &[u8; 3usize] = b"ld\0";
        pub const SCNiPTR: &[u8; 3usize] = b"li\0";
        pub const SCNoPTR: &[u8; 3usize] = b"lo\0";
        pub const SCNuPTR: &[u8; 3usize] = b"lu\0";
        pub const SCNxPTR: &[u8; 3usize] = b"lx\0";
        pub type __u_char = ::core::ffi::c_uchar;
        pub type __u_short = ::core::ffi::c_ushort;
        pub type __u_int = ::core::ffi::c_uint;
        pub type __u_long = ::core::ffi::c_ulong;
        pub type __int8_t = ::core::ffi::c_schar;
        pub type __uint8_t = ::core::ffi::c_uchar;
        pub type __int16_t = ::core::ffi::c_short;
        pub type __uint16_t = ::core::ffi::c_ushort;
        pub type __int32_t = ::core::ffi::c_int;
        pub type __uint32_t = ::core::ffi::c_uint;
        pub type __int64_t = ::core::ffi::c_long;
        pub type __uint64_t = ::core::ffi::c_ulong;
        pub type __int_least8_t = __int8_t;
        pub type __uint_least8_t = __uint8_t;
        pub type __int_least16_t = __int16_t;
        pub type __uint_least16_t = __uint16_t;
        pub type __int_least32_t = __int32_t;
        pub type __uint_least32_t = __uint32_t;
        pub type __int_least64_t = __int64_t;
        pub type __uint_least64_t = __uint64_t;
        pub type __quad_t = ::core::ffi::c_long;
        pub type __u_quad_t = ::core::ffi::c_ulong;
        pub type __intmax_t = ::core::ffi::c_long;
        pub type __uintmax_t = ::core::ffi::c_ulong;
        pub type __dev_t = ::core::ffi::c_ulong;
        pub type __uid_t = ::core::ffi::c_uint;
        pub type __gid_t = ::core::ffi::c_uint;
        pub type __ino_t = ::core::ffi::c_ulong;
        pub type __ino64_t = ::core::ffi::c_ulong;
        pub type __mode_t = ::core::ffi::c_uint;
        pub type __nlink_t = ::core::ffi::c_ulong;
        pub type __off_t = ::core::ffi::c_long;
        pub type __off64_t = ::core::ffi::c_long;
        pub type __pid_t = ::core::ffi::c_int;
        #[repr(C)]
        pub struct __fsid_t {
            pub __val: [::core::ffi::c_int; 2usize],
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for __fsid_t {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "__fsid_t",
                    "__val",
                    &&self.__val,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for __fsid_t {
            #[inline]
            fn default() -> __fsid_t {
                __fsid_t {
                    __val: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for __fsid_t {}
        #[automatically_derived]
        impl ::core::clone::Clone for __fsid_t {
            #[inline]
            fn clone(&self) -> __fsid_t {
                let _: ::core::clone::AssertParamIsClone<[::core::ffi::c_int; 2usize]>;
                *self
            }
        }
        pub type __clock_t = ::core::ffi::c_long;
        pub type __rlim_t = ::core::ffi::c_ulong;
        pub type __rlim64_t = ::core::ffi::c_ulong;
        pub type __id_t = ::core::ffi::c_uint;
        pub type __time_t = ::core::ffi::c_long;
        pub type __useconds_t = ::core::ffi::c_uint;
        pub type __suseconds_t = ::core::ffi::c_long;
        pub type __daddr_t = ::core::ffi::c_int;
        pub type __key_t = ::core::ffi::c_int;
        pub type __clockid_t = ::core::ffi::c_int;
        pub type __timer_t = *mut ::core::ffi::c_void;
        pub type __blksize_t = ::core::ffi::c_long;
        pub type __blkcnt_t = ::core::ffi::c_long;
        pub type __blkcnt64_t = ::core::ffi::c_long;
        pub type __fsblkcnt_t = ::core::ffi::c_ulong;
        pub type __fsblkcnt64_t = ::core::ffi::c_ulong;
        pub type __fsfilcnt_t = ::core::ffi::c_ulong;
        pub type __fsfilcnt64_t = ::core::ffi::c_ulong;
        pub type __fsword_t = ::core::ffi::c_long;
        pub type __ssize_t = ::core::ffi::c_long;
        pub type __syscall_slong_t = ::core::ffi::c_long;
        pub type __syscall_ulong_t = ::core::ffi::c_ulong;
        pub type __loff_t = __off64_t;
        pub type __caddr_t = *mut ::core::ffi::c_char;
        pub type __intptr_t = ::core::ffi::c_long;
        pub type __socklen_t = ::core::ffi::c_uint;
        pub type __sig_atomic_t = ::core::ffi::c_int;
        pub type int_least8_t = __int_least8_t;
        pub type int_least16_t = __int_least16_t;
        pub type int_least32_t = __int_least32_t;
        pub type int_least64_t = __int_least64_t;
        pub type uint_least8_t = __uint_least8_t;
        pub type uint_least16_t = __uint_least16_t;
        pub type uint_least32_t = __uint_least32_t;
        pub type uint_least64_t = __uint_least64_t;
        pub type int_fast8_t = ::core::ffi::c_schar;
        pub type int_fast16_t = ::core::ffi::c_long;
        pub type int_fast32_t = ::core::ffi::c_long;
        pub type int_fast64_t = ::core::ffi::c_long;
        pub type uint_fast8_t = ::core::ffi::c_uchar;
        pub type uint_fast16_t = ::core::ffi::c_ulong;
        pub type uint_fast32_t = ::core::ffi::c_ulong;
        pub type uint_fast64_t = ::core::ffi::c_ulong;
        pub type intmax_t = __intmax_t;
        pub type uintmax_t = __uintmax_t;
        pub type __gwchar_t = ::core::ffi::c_int;
        #[repr(C)]
        pub struct imaxdiv_t {
            pub quot: ::core::ffi::c_long,
            pub rem: ::core::ffi::c_long,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for imaxdiv_t {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "imaxdiv_t",
                    "quot",
                    &&self.quot,
                    "rem",
                    &&self.rem,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for imaxdiv_t {
            #[inline]
            fn default() -> imaxdiv_t {
                imaxdiv_t {
                    quot: ::core::default::Default::default(),
                    rem: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for imaxdiv_t {}
        #[automatically_derived]
        impl ::core::clone::Clone for imaxdiv_t {
            #[inline]
            fn clone(&self) -> imaxdiv_t {
                let _: ::core::clone::AssertParamIsClone<::core::ffi::c_long>;
                let _: ::core::clone::AssertParamIsClone<::core::ffi::c_long>;
                *self
            }
        }
        extern "C" {
            pub fn imaxabs(__n: intmax_t) -> intmax_t;
        }
        extern "C" {
            pub fn imaxdiv(__numer: intmax_t, __denom: intmax_t) -> imaxdiv_t;
        }
        extern "C" {
            pub fn strtoimax(
                __nptr: *const ::core::ffi::c_char,
                __endptr: *mut *mut ::core::ffi::c_char,
                __base: ::core::ffi::c_int,
            ) -> intmax_t;
        }
        extern "C" {
            pub fn strtoumax(
                __nptr: *const ::core::ffi::c_char,
                __endptr: *mut *mut ::core::ffi::c_char,
                __base: ::core::ffi::c_int,
            ) -> uintmax_t;
        }
        extern "C" {
            pub fn wcstoimax(
                __nptr: *const __gwchar_t,
                __endptr: *mut *mut __gwchar_t,
                __base: ::core::ffi::c_int,
            ) -> intmax_t;
        }
        extern "C" {
            pub fn wcstoumax(
                __nptr: *const __gwchar_t,
                __endptr: *mut *mut __gwchar_t,
                __base: ::core::ffi::c_int,
            ) -> uintmax_t;
        }
        pub type wchar_t = ::core::ffi::c_int;
        #[repr(C)]
        #[repr(align(16))]
        pub struct max_align_t {
            pub __clang_max_align_nonce1: ::core::ffi::c_longlong,
            pub __bindgen_padding_0: u64,
            pub __clang_max_align_nonce2: u128,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for max_align_t {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "max_align_t",
                    "__clang_max_align_nonce1",
                    &&self.__clang_max_align_nonce1,
                    "__bindgen_padding_0",
                    &&self.__bindgen_padding_0,
                    "__clang_max_align_nonce2",
                    &&self.__clang_max_align_nonce2,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for max_align_t {
            #[inline]
            fn default() -> max_align_t {
                max_align_t {
                    __clang_max_align_nonce1: ::core::default::Default::default(),
                    __bindgen_padding_0: ::core::default::Default::default(),
                    __clang_max_align_nonce2: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for max_align_t {}
        #[automatically_derived]
        impl ::core::clone::Clone for max_align_t {
            #[inline]
            fn clone(&self) -> max_align_t {
                let _: ::core::clone::AssertParamIsClone<::core::ffi::c_longlong>;
                let _: ::core::clone::AssertParamIsClone<u64>;
                let _: ::core::clone::AssertParamIsClone<u128>;
                *self
            }
        }
        pub type output_fn_t = ::core::option::Option<
            unsafe extern "C" fn(ctx: *mut ::core::ffi::c_void, data: *const u8, len: usize),
        >;
        pub type cross_call_fn_t = ::core::option::Option<
            unsafe extern "C" fn(
                call_id: u32,
                data: *const u8,
                len: usize,
                ctx: *mut ::core::ffi::c_void,
                output: output_fn_t,
            ),
        >;
        #[repr(C)]
        pub struct config_t {
            pub is_dylib: ::core::ffi::c_int,
            pub enclaved: ::core::ffi::c_int,
            pub ocall: cross_call_fn_t,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for config_t {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "config_t",
                    "is_dylib",
                    &&self.is_dylib,
                    "enclaved",
                    &&self.enclaved,
                    "ocall",
                    &&self.ocall,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for config_t {
            #[inline]
            fn default() -> config_t {
                config_t {
                    is_dylib: ::core::default::Default::default(),
                    enclaved: ::core::default::Default::default(),
                    ocall: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for config_t {}
        #[automatically_derived]
        impl ::core::clone::Clone for config_t {
            #[inline]
            fn clone(&self) -> config_t {
                let _: ::core::clone::AssertParamIsClone<::core::ffi::c_int>;
                let _: ::core::clone::AssertParamIsClone<::core::ffi::c_int>;
                let _: ::core::clone::AssertParamIsClone<cross_call_fn_t>;
                *self
            }
        }
        #[repr(C)]
        pub struct ecalls_t {
            pub ecall: cross_call_fn_t,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ecalls_t {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "ecalls_t",
                    "ecall",
                    &&self.ecall,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for ecalls_t {
            #[inline]
            fn default() -> ecalls_t {
                ecalls_t {
                    ecall: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for ecalls_t {}
        #[automatically_derived]
        impl ::core::clone::Clone for ecalls_t {
            #[inline]
            fn clone(&self) -> ecalls_t {
                let _: ::core::clone::AssertParamIsClone<cross_call_fn_t>;
                *self
            }
        }
        pub type init_t = ::core::option::Option<
            unsafe extern "C" fn(
                config: *const config_t,
                ecalls: *mut ecalls_t,
            ) -> ::core::ffi::c_int,
        >;
    }
    pub use types::*;
    pub trait CrossCall {
        fn cross_call(&self, id: u32, data: &[u8]) -> Vec<u8>;
        fn cross_call_mut(&mut self, call_id: u32, data: &[u8]) -> Vec<u8>;
    }
    pub trait ECall: CrossCall {}
    pub trait OCall: CrossCall {}
    pub mod ecall {
        use super::ECall;
        use crate::types::{AccountId, Balance, BlockNumber, ExecSideEffects, Hash, Weight};
        use pink_macro::cross_call;
        use scale::{Decode, Encode};
        impl ExecSideEffects {
            pub fn into_query_only_effects(self) -> Self {
                match self {
                    ExecSideEffects::V1 {
                        pink_events,
                        ink_events: _,
                        instantiated: _,
                    } => Self::V1 {
                        pink_events: pink_events
                            .into_iter()
                            .filter(|(_, event)| event.allowed_in_query())
                            .collect(),
                        ink_events: ::alloc::vec::Vec::new(),
                        instantiated: ::alloc::vec::Vec::new(),
                    },
                }
            }
        }
        pub trait EventCallbacks {
            fn emit_log(&self, contract: &AccountId, in_query: bool, level: u8, message: String);
        }
        pub struct TransactionArguments {
            pub origin: AccountId,
            pub now: u64,
            pub block_number: BlockNumber,
            pub transfer: Balance,
            pub gas_limit: Weight,
            pub gas_free: bool,
            pub storage_deposit_limit: Option<Balance>,
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for TransactionArguments {
                fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                    &self,
                    __codec_dest_edqy: &mut __CodecOutputEdqy,
                ) {
                    ::scale::Encode::encode_to(&self.origin, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.now, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.block_number, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.transfer, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.gas_limit, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.gas_free, __codec_dest_edqy);
                    ::scale::Encode::encode_to(&self.storage_deposit_limit, __codec_dest_edqy);
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for TransactionArguments {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for TransactionArguments {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    ::core::result::Result::Ok(TransactionArguments {
                        origin: {
                            let __codec_res_edqy =
                                <AccountId as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `TransactionArguments::origin`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        now: {
                            let __codec_res_edqy =
                                <u64 as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(
                                        e.chain("Could not decode `TransactionArguments::now`"),
                                    )
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        block_number: {
                            let __codec_res_edqy =
                                <BlockNumber as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(e.chain(
                                        "Could not decode `TransactionArguments::block_number`",
                                    ))
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        transfer: {
                            let __codec_res_edqy =
                                <Balance as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(e.chain(
                                        "Could not decode `TransactionArguments::transfer`",
                                    ))
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        gas_limit: {
                            let __codec_res_edqy =
                                <Weight as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(e.chain(
                                        "Could not decode `TransactionArguments::gas_limit`",
                                    ))
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        gas_free: {
                            let __codec_res_edqy =
                                <bool as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy {
                                ::core::result::Result::Err(e) => {
                                    return ::core::result::Result::Err(e.chain(
                                        "Could not decode `TransactionArguments::gas_free`",
                                    ))
                                }
                                ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        },
                        storage_deposit_limit: {
                            let __codec_res_edqy =
                                <Option<Balance> as ::scale::Decode>::decode(__codec_input_edqy);
                            match __codec_res_edqy { :: core :: result :: Result :: Err (e) => return :: core :: result :: Result :: Err (e . chain ("Could not decode `TransactionArguments::storage_deposit_limit`")) , :: core :: result :: Result :: Ok (__codec_res_edqy) => __codec_res_edqy , }
                        },
                    })
                }
            }
        };
        #[automatically_derived]
        impl ::core::clone::Clone for TransactionArguments {
            #[inline]
            fn clone(&self) -> TransactionArguments {
                TransactionArguments {
                    origin: ::core::clone::Clone::clone(&self.origin),
                    now: ::core::clone::Clone::clone(&self.now),
                    block_number: ::core::clone::Clone::clone(&self.block_number),
                    transfer: ::core::clone::Clone::clone(&self.transfer),
                    gas_limit: ::core::clone::Clone::clone(&self.gas_limit),
                    gas_free: ::core::clone::Clone::clone(&self.gas_free),
                    storage_deposit_limit: ::core::clone::Clone::clone(&self.storage_deposit_limit),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TransactionArguments {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "origin",
                    "now",
                    "block_number",
                    "transfer",
                    "gas_limit",
                    "gas_free",
                    "storage_deposit_limit",
                ];
                let values: &[&dyn::core::fmt::Debug] = &[
                    &&self.origin,
                    &&self.now,
                    &&self.block_number,
                    &&self.transfer,
                    &&self.gas_limit,
                    &&self.gas_free,
                    &&self.storage_deposit_limit,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "TransactionArguments",
                    names,
                    values,
                )
            }
        }
        pub trait ECalls {
            fn set_cluster_id(&mut self, cluster_id: Hash);
            fn setup(
                &self,
                gas_price: Balance,
                deposit_per_item: Balance,
                deposit_per_byte: Balance,
                treasury_account: AccountId,
            );
            fn deposit(&self, who: AccountId, value: Balance);
            fn set_key_seed(&self, seed: Vec<u8>);
            fn set_key(&self, key: Vec<u8>);
            fn get_key(&self) -> Vec<u8>;
            fn upload_code(
                &self,
                account: AccountId,
                code: Vec<u8>,
                deterministic: bool,
            ) -> Result<Hash, Vec<u8>>;
            fn upload_sidevm_code(
                &self,
                account: AccountId,
                code: Vec<u8>,
            ) -> Result<Hash, Vec<u8>>;
            fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>>;
            fn set_system_contract(&self, address: AccountId);
            fn system_contract(&self) -> Option<AccountId>;
            fn set_system_contract_code(&self, code_hash: Hash);
            fn root(&self) -> Hash;
            fn free_balance(&self, account: AccountId) -> Balance;
            fn total_balance(&self, account: AccountId) -> Balance;
            fn code_hash(&self, account: AccountId) -> Option<Hash>;
            fn code_exists(&self, code_hash: Hash, sidevm: bool) -> bool;
            fn contract_instantiate(
                &self,
                code_hash: Hash,
                input_data: Vec<u8>,
                salt: Vec<u8>,
                in_query: bool,
                tx_args: TransactionArguments,
            ) -> (Vec<u8>, ExecSideEffects);
            fn contract_call(
                &self,
                contract: AccountId,
                input_data: Vec<u8>,
                in_query: bool,
                tx_args: TransactionArguments,
            ) -> (Vec<u8>, ExecSideEffects);
        }
        impl<T: ECall> ECalls for T {
            fn set_cluster_id(&mut self, cluster_id: Hash) {
                let inputs = (cluster_id);
                let ret = self.cross_call_mut(1, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn setup(
                &self,
                gas_price: Balance,
                deposit_per_item: Balance,
                deposit_per_byte: Balance,
                treasury_account: AccountId,
            ) {
                let inputs = (
                    gas_price,
                    deposit_per_item,
                    deposit_per_byte,
                    treasury_account,
                );
                let ret = self.cross_call(2, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn deposit(&self, who: AccountId, value: Balance) {
                let inputs = (who, value);
                let ret = self.cross_call(3, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn set_key_seed(&self, seed: Vec<u8>) {
                let inputs = (seed);
                let ret = self.cross_call(4, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn set_key(&self, key: Vec<u8>) {
                let inputs = (key);
                let ret = self.cross_call(5, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn get_key(&self) -> Vec<u8> {
                let inputs = ();
                let ret = self.cross_call(6, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn upload_code(
                &self,
                account: AccountId,
                code: Vec<u8>,
                deterministic: bool,
            ) -> Result<Hash, Vec<u8>> {
                let inputs = (account, code, deterministic);
                let ret = self.cross_call(7, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn upload_sidevm_code(
                &self,
                account: AccountId,
                code: Vec<u8>,
            ) -> Result<Hash, Vec<u8>> {
                let inputs = (account, code);
                let ret = self.cross_call(8, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn get_sidevm_code(&self, hash: Hash) -> Option<Vec<u8>> {
                let inputs = (hash);
                let ret = self.cross_call(9, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn set_system_contract(&self, address: AccountId) {
                let inputs = (address);
                let ret = self.cross_call(10, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn system_contract(&self) -> Option<AccountId> {
                let inputs = ();
                let ret = self.cross_call(11, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn set_system_contract_code(&self, code_hash: Hash) {
                let inputs = (code_hash);
                let ret = self.cross_call(12, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn root(&self) -> Hash {
                let inputs = ();
                let ret = self.cross_call(13, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn free_balance(&self, account: AccountId) -> Balance {
                let inputs = (account);
                let ret = self.cross_call(14, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn total_balance(&self, account: AccountId) -> Balance {
                let inputs = (account);
                let ret = self.cross_call(15, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn code_hash(&self, account: AccountId) -> Option<Hash> {
                let inputs = (account);
                let ret = self.cross_call(16, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn code_exists(&self, code_hash: Hash, sidevm: bool) -> bool {
                let inputs = (code_hash, sidevm);
                let ret = self.cross_call(18, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn contract_instantiate(
                &self,
                code_hash: Hash,
                input_data: Vec<u8>,
                salt: Vec<u8>,
                in_query: bool,
                tx_args: TransactionArguments,
            ) -> (Vec<u8>, ExecSideEffects) {
                let inputs = (code_hash, input_data, salt, in_query, tx_args);
                let ret = self.cross_call(19, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn contract_call(
                &self,
                contract: AccountId,
                input_data: Vec<u8>,
                in_query: bool,
                tx_args: TransactionArguments,
            ) -> (Vec<u8>, ExecSideEffects) {
                let inputs = (contract, input_data, in_query, tx_args);
                let ret = self.cross_call(20, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
        }
        pub fn dispatch(env: &mut (impl ECalls + ?Sized), id: u32, input: &[u8]) -> Vec<u8> {
            match id {
                1 => {
                    let (cluster_id) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.set_cluster_id(cluster_id).encode()
                }
                2 => {
                    let (gas_price, deposit_per_item, deposit_per_byte, treasury_account) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.setup(
                        gas_price,
                        deposit_per_item,
                        deposit_per_byte,
                        treasury_account,
                    )
                    .encode()
                }
                3 => {
                    let (who, value) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.deposit(who, value).encode()
                }
                4 => {
                    let (seed) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.set_key_seed(seed).encode()
                }
                5 => {
                    let (key) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.set_key(key).encode()
                }
                6 => {
                    let () = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.get_key().encode()
                }
                7 => {
                    let (account, code, deterministic) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.upload_code(account, code, deterministic).encode()
                }
                8 => {
                    let (account, code) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.upload_sidevm_code(account, code).encode()
                }
                9 => {
                    let (hash) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.get_sidevm_code(hash).encode()
                }
                10 => {
                    let (address) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.set_system_contract(address).encode()
                }
                11 => {
                    let () = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.system_contract().encode()
                }
                12 => {
                    let (code_hash) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.set_system_contract_code(code_hash).encode()
                }
                13 => {
                    let () = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.root().encode()
                }
                14 => {
                    let (account) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.free_balance(account).encode()
                }
                15 => {
                    let (account) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.total_balance(account).encode()
                }
                16 => {
                    let (account) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.code_hash(account).encode()
                }
                18 => {
                    let (code_hash, sidevm) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.code_exists(code_hash, sidevm).encode()
                }
                19 => {
                    let (code_hash, input_data, salt, in_query, tx_args) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.contract_instantiate(code_hash, input_data, salt, in_query, tx_args)
                        .encode()
                }
                20 => {
                    let (contract, input_data, in_query, tx_args) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.contract_call(contract, input_data, in_query, tx_args)
                        .encode()
                }
                _ => ::core::panicking::panic_fmt(format_args!("Unknown call id {0}", id)),
            }
        }
        pub fn id2name(id: u32) -> &'static str {
            match id {
                1u32 => "set_cluster_id",
                2u32 => "setup",
                3u32 => "deposit",
                4u32 => "set_key_seed",
                5u32 => "set_key",
                6u32 => "get_key",
                7u32 => "upload_code",
                8u32 => "upload_sidevm_code",
                9u32 => "get_sidevm_code",
                10u32 => "set_system_contract",
                11u32 => "system_contract",
                12u32 => "set_system_contract_code",
                13u32 => "root",
                14u32 => "free_balance",
                15u32 => "total_balance",
                16u32 => "code_hash",
                18u32 => "code_exists",
                19u32 => "contract_instantiate",
                20u32 => "contract_call",
                _ => "unknown",
            }
        }
    }
    pub mod ocall {
        use super::OCall;
        use crate::types::{AccountId, Hash};
        use pink_macro::cross_call;
        use scale::{Decode, Encode};
        pub trait OCalls {
            fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>>;
            fn storage_commit(&mut self, root: Hash, changes: Vec<(Vec<u8>, (Vec<u8>, i32))>);
            fn is_in_query(&self) -> bool;
            fn emit_log(&self, contract: AccountId, in_query: bool, level: u8, message: String);
        }
        impl<T: OCall> OCalls for T {
            fn storage_get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
                let inputs = (key);
                let ret = self.cross_call(1, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn storage_commit(&mut self, root: Hash, changes: Vec<(Vec<u8>, (Vec<u8>, i32))>) {
                let inputs = (root, changes);
                let ret = self.cross_call_mut(2, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn is_in_query(&self) -> bool {
                let inputs = ();
                let ret = self.cross_call(3, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
            fn emit_log(&self, contract: AccountId, in_query: bool, level: u8, message: String) {
                let inputs = (contract, in_query, level, message);
                let ret = self.cross_call(4, &inputs.encode());
                Decode::decode(&mut &ret[..]).expect("Decode failed")
            }
        }
        pub fn dispatch(env: &mut (impl OCalls + ?Sized), id: u32, input: &[u8]) -> Vec<u8> {
            match id {
                1 => {
                    let (key) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.storage_get(key).encode()
                }
                2 => {
                    let (root, changes) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.storage_commit(root, changes).encode()
                }
                3 => {
                    let () = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.is_in_query().encode()
                }
                4 => {
                    let (contract, in_query, level, message) =
                        Decode::decode(&mut &input[..]).expect("Failed to decode args");
                    env.emit_log(contract, in_query, level, message).encode()
                }
                _ => ::core::panicking::panic_fmt(format_args!("Unknown call id {0}", id)),
            }
        }
        pub fn id2name(id: u32) -> &'static str {
            match id {
                1u32 => "storage_get",
                2u32 => "storage_commit",
                3u32 => "is_in_query",
                4u32 => "emit_log",
                _ => "unknown",
            }
        }
    }
}
pub mod helper {
    pub trait ParamType {
        type T;
        type E;
    }
    impl<T> ParamType for Option<T> {
        type T = T;
        type E = ();
    }
    impl<T, E> ParamType for Result<T, E> {
        type T = T;
        type E = E;
    }
    pub type InnerType<T> = <T as ParamType>::T;
    pub type InnerError<T> = <T as ParamType>::E;
}
