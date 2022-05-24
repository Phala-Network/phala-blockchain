//! Tools for writing Sidevm programs.

// #![warn(missing_docs)]

use core::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task,
};
use std::cell::RefCell;

use crate::ocall_funcs_guest as ocall;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use scale::{Decode, Encode};
use tinyvec::TinyVec;

pub use args_stack::RetEncode;
pub use ocall_def::*;
pub use pink_sidevm_macro::main;
pub use tasks::{spawn, TaskHandle};

mod args_stack;
mod ocall_def;
mod tasks;

cfg_if::cfg_if! {
    if #[cfg(all(not(test), any(target_pointer_width = "32", feature = "host")))] {
        pub type IntPtr = i32;
    } else {
        // For unit test
        pub type IntPtr = i64;
    }
}

pub type IntRet = i64;

#[derive(Clone, Copy, Debug, derive_more::Display, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OcallError {
    Ok = 0,
    UnknownCallNumber = 1,
    InvalidAddress = 2,
    InvalidParameter = 3,
    InvalidEncoding = 4,
    NoMemory = 5,
    NoReturnValue = 6,
    NotFound = 7,
    UnsupportedOperation = 8,
    IoError = 9,
    ResourceLimited = 10,
    Pending = 11,
    EndOfFile = 12,
    /// Reserved for future use
    Reserved13 = 13,
    /// Reserved for future use
    Reserved14 = 14,
    /// Reserved for future use
    Reserved15 = 15,
    /// Reserved for future use
    Reserved16 = 16,
    /// Reserved for future use
    Reserved17 = 17,
    /// Reserved for future use
    Reserved18 = 18,
    /// Reserved for future use
    Reserved19 = 19,
    /// Reserved for future use
    Reserved20 = 20,
    /// Reserved for future use
    Reserved21 = 21,
    /// Reserved for future use
    Reserved22 = 22,
    /// Reserved for future use
    Reserved23 = 23,
    /// Reserved for future use
    Reserved24 = 24,
    /// Reserved for future use
    Reserved25 = 25,
    /// Reserved for future use
    Reserved26 = 26,
    /// Reserved for future use
    Reserved27 = 27,
    /// Reserved for future use
    Reserved28 = 28,
    /// Reserved for future use
    Reserved29 = 29,
    /// Reserved for future use
    Reserved30 = 30,
    /// Reserved for future use
    Reserved31 = 31,
    /// Reserved for future use
    Reserved32 = 32,
    /// Reserved for future use
    Reserved33 = 33,
    /// Reserved for future use
    Reserved34 = 34,
    /// Reserved for future use
    Reserved35 = 35,
    /// Reserved for future use
    Reserved36 = 36,
    /// Reserved for future use
    Reserved37 = 37,
    /// Reserved for future use
    Reserved38 = 38,
    /// Reserved for future use
    Reserved39 = 39,
    /// Reserved for future use
    Reserved40 = 40,
    /// Reserved for future use
    Reserved41 = 41,
    /// Reserved for future use
    Reserved42 = 42,
    /// Reserved for future use
    Reserved43 = 43,
    /// Reserved for future use
    Reserved44 = 44,
    /// Reserved for future use
    Reserved45 = 45,
    /// Reserved for future use
    Reserved46 = 46,
    /// Reserved for future use
    Reserved47 = 47,
    /// Reserved for future use
    Reserved48 = 48,
    /// Reserved for future use
    Reserved49 = 49,
    /// Reserved for future use
    Reserved50 = 50,
    /// Reserved for future use
    Reserved51 = 51,
    /// Reserved for future use
    Reserved52 = 52,
    /// Reserved for future use
    Reserved53 = 53,
    /// Reserved for future use
    Reserved54 = 54,
    /// Reserved for future use
    Reserved55 = 55,
    /// Reserved for future use
    Reserved56 = 56,
    /// Reserved for future use
    Reserved57 = 57,
    /// Reserved for future use
    Reserved58 = 58,
    /// Reserved for future use
    Reserved59 = 59,
    /// Reserved for future use
    Reserved60 = 60,
    /// Reserved for future use
    Reserved61 = 61,
    /// Reserved for future use
    Reserved62 = 62,
    /// Reserved for future use
    Reserved63 = 63,
    /// Reserved for future use
    Reserved64 = 64,
    /// Reserved for future use
    Reserved65 = 65,
    /// Reserved for future use
    Reserved66 = 66,
    /// Reserved for future use
    Reserved67 = 67,
    /// Reserved for future use
    Reserved68 = 68,
    /// Reserved for future use
    Reserved69 = 69,
    /// Reserved for future use
    Reserved70 = 70,
    /// Reserved for future use
    Reserved71 = 71,
    /// Reserved for future use
    Reserved72 = 72,
    /// Reserved for future use
    Reserved73 = 73,
    /// Reserved for future use
    Reserved74 = 74,
    /// Reserved for future use
    Reserved75 = 75,
    /// Reserved for future use
    Reserved76 = 76,
    /// Reserved for future use
    Reserved77 = 77,
    /// Reserved for future use
    Reserved78 = 78,
    /// Reserved for future use
    Reserved79 = 79,
    /// Reserved for future use
    Reserved80 = 80,
    /// Reserved for future use
    Reserved81 = 81,
    /// Reserved for future use
    Reserved82 = 82,
    /// Reserved for future use
    Reserved83 = 83,
    /// Reserved for future use
    Reserved84 = 84,
    /// Reserved for future use
    Reserved85 = 85,
    /// Reserved for future use
    Reserved86 = 86,
    /// Reserved for future use
    Reserved87 = 87,
    /// Reserved for future use
    Reserved88 = 88,
    /// Reserved for future use
    Reserved89 = 89,
    /// Reserved for future use
    Reserved90 = 90,
    /// Reserved for future use
    Reserved91 = 91,
    /// Reserved for future use
    Reserved92 = 92,
    /// Reserved for future use
    Reserved93 = 93,
    /// Reserved for future use
    Reserved94 = 94,
    /// Reserved for future use
    Reserved95 = 95,
    /// Reserved for future use
    Reserved96 = 96,
    /// Reserved for future use
    Reserved97 = 97,
    /// Reserved for future use
    Reserved98 = 98,
    /// Reserved for future use
    Reserved99 = 99,
    /// Reserved for future use
    Reserved100 = 100,
    /// Reserved for future use
    Reserved101 = 101,
    /// Reserved for future use
    Reserved102 = 102,
    /// Reserved for future use
    Reserved103 = 103,
    /// Reserved for future use
    Reserved104 = 104,
    /// Reserved for future use
    Reserved105 = 105,
    /// Reserved for future use
    Reserved106 = 106,
    /// Reserved for future use
    Reserved107 = 107,
    /// Reserved for future use
    Reserved108 = 108,
    /// Reserved for future use
    Reserved109 = 109,
    /// Reserved for future use
    Reserved110 = 110,
    /// Reserved for future use
    Reserved111 = 111,
    /// Reserved for future use
    Reserved112 = 112,
    /// Reserved for future use
    Reserved113 = 113,
    /// Reserved for future use
    Reserved114 = 114,
    /// Reserved for future use
    Reserved115 = 115,
    /// Reserved for future use
    Reserved116 = 116,
    /// Reserved for future use
    Reserved117 = 117,
    /// Reserved for future use
    Reserved118 = 118,
    /// Reserved for future use
    Reserved119 = 119,
    /// Reserved for future use
    Reserved120 = 120,
    /// Reserved for future use
    Reserved121 = 121,
    /// Reserved for future use
    Reserved122 = 122,
    /// Reserved for future use
    Reserved123 = 123,
    /// Reserved for future use
    Reserved124 = 124,
    /// Reserved for future use
    Reserved125 = 125,
    /// Reserved for future use
    Reserved126 = 126,
    /// Reserved for future use
    Reserved127 = 127,
    /// Reserved for future use
    Reserved128 = 128,
    /// Reserved for future use
    Reserved129 = 129,
    /// Reserved for future use
    Reserved130 = 130,
    /// Reserved for future use
    Reserved131 = 131,
    /// Reserved for future use
    Reserved132 = 132,
    /// Reserved for future use
    Reserved133 = 133,
    /// Reserved for future use
    Reserved134 = 134,
    /// Reserved for future use
    Reserved135 = 135,
    /// Reserved for future use
    Reserved136 = 136,
    /// Reserved for future use
    Reserved137 = 137,
    /// Reserved for future use
    Reserved138 = 138,
    /// Reserved for future use
    Reserved139 = 139,
    /// Reserved for future use
    Reserved140 = 140,
    /// Reserved for future use
    Reserved141 = 141,
    /// Reserved for future use
    Reserved142 = 142,
    /// Reserved for future use
    Reserved143 = 143,
    /// Reserved for future use
    Reserved144 = 144,
    /// Reserved for future use
    Reserved145 = 145,
    /// Reserved for future use
    Reserved146 = 146,
    /// Reserved for future use
    Reserved147 = 147,
    /// Reserved for future use
    Reserved148 = 148,
    /// Reserved for future use
    Reserved149 = 149,
    /// Reserved for future use
    Reserved150 = 150,
    /// Reserved for future use
    Reserved151 = 151,
    /// Reserved for future use
    Reserved152 = 152,
    /// Reserved for future use
    Reserved153 = 153,
    /// Reserved for future use
    Reserved154 = 154,
    /// Reserved for future use
    Reserved155 = 155,
    /// Reserved for future use
    Reserved156 = 156,
    /// Reserved for future use
    Reserved157 = 157,
    /// Reserved for future use
    Reserved158 = 158,
    /// Reserved for future use
    Reserved159 = 159,
    /// Reserved for future use
    Reserved160 = 160,
    /// Reserved for future use
    Reserved161 = 161,
    /// Reserved for future use
    Reserved162 = 162,
    /// Reserved for future use
    Reserved163 = 163,
    /// Reserved for future use
    Reserved164 = 164,
    /// Reserved for future use
    Reserved165 = 165,
    /// Reserved for future use
    Reserved166 = 166,
    /// Reserved for future use
    Reserved167 = 167,
    /// Reserved for future use
    Reserved168 = 168,
    /// Reserved for future use
    Reserved169 = 169,
    /// Reserved for future use
    Reserved170 = 170,
    /// Reserved for future use
    Reserved171 = 171,
    /// Reserved for future use
    Reserved172 = 172,
    /// Reserved for future use
    Reserved173 = 173,
    /// Reserved for future use
    Reserved174 = 174,
    /// Reserved for future use
    Reserved175 = 175,
    /// Reserved for future use
    Reserved176 = 176,
    /// Reserved for future use
    Reserved177 = 177,
    /// Reserved for future use
    Reserved178 = 178,
    /// Reserved for future use
    Reserved179 = 179,
    /// Reserved for future use
    Reserved180 = 180,
    /// Reserved for future use
    Reserved181 = 181,
    /// Reserved for future use
    Reserved182 = 182,
    /// Reserved for future use
    Reserved183 = 183,
    /// Reserved for future use
    Reserved184 = 184,
    /// Reserved for future use
    Reserved185 = 185,
    /// Reserved for future use
    Reserved186 = 186,
    /// Reserved for future use
    Reserved187 = 187,
    /// Reserved for future use
    Reserved188 = 188,
    /// Reserved for future use
    Reserved189 = 189,
    /// Reserved for future use
    Reserved190 = 190,
    /// Reserved for future use
    Reserved191 = 191,
    /// Reserved for future use
    Reserved192 = 192,
    /// Reserved for future use
    Reserved193 = 193,
    /// Reserved for future use
    Reserved194 = 194,
    /// Reserved for future use
    Reserved195 = 195,
    /// Reserved for future use
    Reserved196 = 196,
    /// Reserved for future use
    Reserved197 = 197,
    /// Reserved for future use
    Reserved198 = 198,
    /// Reserved for future use
    Reserved199 = 199,
    /// Reserved for future use
    Reserved200 = 200,
    /// Reserved for future use
    Reserved201 = 201,
    /// Reserved for future use
    Reserved202 = 202,
    /// Reserved for future use
    Reserved203 = 203,
    /// Reserved for future use
    Reserved204 = 204,
    /// Reserved for future use
    Reserved205 = 205,
    /// Reserved for future use
    Reserved206 = 206,
    /// Reserved for future use
    Reserved207 = 207,
    /// Reserved for future use
    Reserved208 = 208,
    /// Reserved for future use
    Reserved209 = 209,
    /// Reserved for future use
    Reserved210 = 210,
    /// Reserved for future use
    Reserved211 = 211,
    /// Reserved for future use
    Reserved212 = 212,
    /// Reserved for future use
    Reserved213 = 213,
    /// Reserved for future use
    Reserved214 = 214,
    /// Reserved for future use
    Reserved215 = 215,
    /// Reserved for future use
    Reserved216 = 216,
    /// Reserved for future use
    Reserved217 = 217,
    /// Reserved for future use
    Reserved218 = 218,
    /// Reserved for future use
    Reserved219 = 219,
    /// Reserved for future use
    Reserved220 = 220,
    /// Reserved for future use
    Reserved221 = 221,
    /// Reserved for future use
    Reserved222 = 222,
    /// Reserved for future use
    Reserved223 = 223,
    /// Reserved for future use
    Reserved224 = 224,
    /// Reserved for future use
    Reserved225 = 225,
    /// Reserved for future use
    Reserved226 = 226,
    /// Reserved for future use
    Reserved227 = 227,
    /// Reserved for future use
    Reserved228 = 228,
    /// Reserved for future use
    Reserved229 = 229,
    /// Reserved for future use
    Reserved230 = 230,
    /// Reserved for future use
    Reserved231 = 231,
    /// Reserved for future use
    Reserved232 = 232,
    /// Reserved for future use
    Reserved233 = 233,
    /// Reserved for future use
    Reserved234 = 234,
    /// Reserved for future use
    Reserved235 = 235,
    /// Reserved for future use
    Reserved236 = 236,
    /// Reserved for future use
    Reserved237 = 237,
    /// Reserved for future use
    Reserved238 = 238,
    /// Reserved for future use
    Reserved239 = 239,
    /// Reserved for future use
    Reserved240 = 240,
    /// Reserved for future use
    Reserved241 = 241,
    /// Reserved for future use
    Reserved242 = 242,
    /// Reserved for future use
    Reserved243 = 243,
    /// Reserved for future use
    Reserved244 = 244,
    /// Reserved for future use
    Reserved245 = 245,
    /// Reserved for future use
    Reserved246 = 246,
    /// Reserved for future use
    Reserved247 = 247,
    /// Reserved for future use
    Reserved248 = 248,
    /// Reserved for future use
    Reserved249 = 249,
    /// Reserved for future use
    Reserved250 = 250,
    /// Reserved for future use
    Reserved251 = 251,
    /// Reserved for future use
    Reserved252 = 252,
    /// Reserved for future use
    Reserved253 = 253,
    /// Reserved for future use
    Reserved254 = 254,
    /// Reserved for future use
    Reserved255 = 255,
}

impl std::error::Error for OcallError {}

pub type Result<T, E = OcallError> = core::result::Result<T, E>;
pub trait OcallEnv {
    fn put_return(&mut self, rv: Vec<u8>) -> usize;
    fn take_return(&mut self) -> Option<Vec<u8>>;
}

pub trait VmMemory {
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()>;
    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]>;
    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]>;
}

extern "C" {
    fn sidevm_ocall(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntRet;
    fn sidevm_ocall_fast_return(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntRet;
}

unsafe fn do_ocall(func_id: i32, p0: IntPtr, p1: IntPtr, p2: IntPtr, p3: IntPtr) -> IntRet {
    sidevm_ocall(tasks::current_task(), func_id, p0, p1, p2, p3)
}

unsafe fn do_ocall_fast_return(
    func_id: i32,
    p0: IntPtr,
    p1: IntPtr,
    p2: IntPtr,
    p3: IntPtr,
) -> IntRet {
    sidevm_ocall_fast_return(tasks::current_task(), func_id, p0, p1, p2, p3)
}

#[derive(Default)]
struct Buffer(TinyVec<[u8; 128]>);

impl Deref for Buffer {
    type Target = TinyVec<[u8; 128]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl scale::Output for Buffer {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes)
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

fn alloc_buffer(size: usize) -> Buffer {
    let mut buf = Buffer::default();
    buf.0.resize(size, 0_u8);
    buf
}

#[cfg(test)]
mod tests;
