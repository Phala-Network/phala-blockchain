use core::ops::{Deref, DerefMut};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use scale::{Decode, Encode};
use tinyvec::TinyVec;

pub use args_stack::RetEncode;
pub use ocall_def::*;

mod args_stack;
mod ocall_def;

cfg_if::cfg_if! {
    if #[cfg(any(target_pointer_width = "32", feature = "host"))] {
        pub type IntPtr = i32;
    } else {
        // For unit test
        pub type IntPtr = i64;
    }
}

pub type IntRet = i64;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Encode, Decode)]
pub enum LogLevel {
    None = 0,
    Error,
    Warn,
    Debug,
    Trace,
}

#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OcallError {
    Ok = 0,
    UnknownCallNumber = 1,
    InvalidAddress = 2,
    InvalidParameter = 3,
    InvalidEncoding = 4,
    NoMemory = 5,
    NoReturnValue = 6,
    ResourceNotFound = 7,
    Pending = 8,
    Reserved9 = 9,
    Reserved10 = 10,
    Reserved11 = 11,
    Reserved12 = 12,
    Reserved13 = 13,
    Reserved14 = 14,
    Reserved15 = 15,
    Reserved16 = 16,
    Reserved17 = 17,
    Reserved18 = 18,
    Reserved19 = 19,
    Reserved20 = 20,
    Reserved21 = 21,
    Reserved22 = 22,
    Reserved23 = 23,
    Reserved24 = 24,
    Reserved25 = 25,
    Reserved26 = 26,
    Reserved27 = 27,
    Reserved28 = 28,
    Reserved29 = 29,
    Reserved30 = 30,
    Reserved31 = 31,
    Reserved32 = 32,
    Reserved33 = 33,
    Reserved34 = 34,
    Reserved35 = 35,
    Reserved36 = 36,
    Reserved37 = 37,
    Reserved38 = 38,
    Reserved39 = 39,
    Reserved40 = 40,
    Reserved41 = 41,
    Reserved42 = 42,
    Reserved43 = 43,
    Reserved44 = 44,
    Reserved45 = 45,
    Reserved46 = 46,
    Reserved47 = 47,
    Reserved48 = 48,
    Reserved49 = 49,
    Reserved50 = 50,
    Reserved51 = 51,
    Reserved52 = 52,
    Reserved53 = 53,
    Reserved54 = 54,
    Reserved55 = 55,
    Reserved56 = 56,
    Reserved57 = 57,
    Reserved58 = 58,
    Reserved59 = 59,
    Reserved60 = 60,
    Reserved61 = 61,
    Reserved62 = 62,
    Reserved63 = 63,
    Reserved64 = 64,
    Reserved65 = 65,
    Reserved66 = 66,
    Reserved67 = 67,
    Reserved68 = 68,
    Reserved69 = 69,
    Reserved70 = 70,
    Reserved71 = 71,
    Reserved72 = 72,
    Reserved73 = 73,
    Reserved74 = 74,
    Reserved75 = 75,
    Reserved76 = 76,
    Reserved77 = 77,
    Reserved78 = 78,
    Reserved79 = 79,
    Reserved80 = 80,
    Reserved81 = 81,
    Reserved82 = 82,
    Reserved83 = 83,
    Reserved84 = 84,
    Reserved85 = 85,
    Reserved86 = 86,
    Reserved87 = 87,
    Reserved88 = 88,
    Reserved89 = 89,
    Reserved90 = 90,
    Reserved91 = 91,
    Reserved92 = 92,
    Reserved93 = 93,
    Reserved94 = 94,
    Reserved95 = 95,
    Reserved96 = 96,
    Reserved97 = 97,
    Reserved98 = 98,
    Reserved99 = 99,
    Reserved100 = 100,
    Reserved101 = 101,
    Reserved102 = 102,
    Reserved103 = 103,
    Reserved104 = 104,
    Reserved105 = 105,
    Reserved106 = 106,
    Reserved107 = 107,
    Reserved108 = 108,
    Reserved109 = 109,
    Reserved110 = 110,
    Reserved111 = 111,
    Reserved112 = 112,
    Reserved113 = 113,
    Reserved114 = 114,
    Reserved115 = 115,
    Reserved116 = 116,
    Reserved117 = 117,
    Reserved118 = 118,
    Reserved119 = 119,
    Reserved120 = 120,
    Reserved121 = 121,
    Reserved122 = 122,
    Reserved123 = 123,
    Reserved124 = 124,
    Reserved125 = 125,
    Reserved126 = 126,
    Reserved127 = 127,
    Reserved128 = 128,
    Reserved129 = 129,
    Reserved130 = 130,
    Reserved131 = 131,
    Reserved132 = 132,
    Reserved133 = 133,
    Reserved134 = 134,
    Reserved135 = 135,
    Reserved136 = 136,
    Reserved137 = 137,
    Reserved138 = 138,
    Reserved139 = 139,
    Reserved140 = 140,
    Reserved141 = 141,
    Reserved142 = 142,
    Reserved143 = 143,
    Reserved144 = 144,
    Reserved145 = 145,
    Reserved146 = 146,
    Reserved147 = 147,
    Reserved148 = 148,
    Reserved149 = 149,
    Reserved150 = 150,
    Reserved151 = 151,
    Reserved152 = 152,
    Reserved153 = 153,
    Reserved154 = 154,
    Reserved155 = 155,
    Reserved156 = 156,
    Reserved157 = 157,
    Reserved158 = 158,
    Reserved159 = 159,
    Reserved160 = 160,
    Reserved161 = 161,
    Reserved162 = 162,
    Reserved163 = 163,
    Reserved164 = 164,
    Reserved165 = 165,
    Reserved166 = 166,
    Reserved167 = 167,
    Reserved168 = 168,
    Reserved169 = 169,
    Reserved170 = 170,
    Reserved171 = 171,
    Reserved172 = 172,
    Reserved173 = 173,
    Reserved174 = 174,
    Reserved175 = 175,
    Reserved176 = 176,
    Reserved177 = 177,
    Reserved178 = 178,
    Reserved179 = 179,
    Reserved180 = 180,
    Reserved181 = 181,
    Reserved182 = 182,
    Reserved183 = 183,
    Reserved184 = 184,
    Reserved185 = 185,
    Reserved186 = 186,
    Reserved187 = 187,
    Reserved188 = 188,
    Reserved189 = 189,
    Reserved190 = 190,
    Reserved191 = 191,
    Reserved192 = 192,
    Reserved193 = 193,
    Reserved194 = 194,
    Reserved195 = 195,
    Reserved196 = 196,
    Reserved197 = 197,
    Reserved198 = 198,
    Reserved199 = 199,
    Reserved200 = 200,
    Reserved201 = 201,
    Reserved202 = 202,
    Reserved203 = 203,
    Reserved204 = 204,
    Reserved205 = 205,
    Reserved206 = 206,
    Reserved207 = 207,
    Reserved208 = 208,
    Reserved209 = 209,
    Reserved210 = 210,
    Reserved211 = 211,
    Reserved212 = 212,
    Reserved213 = 213,
    Reserved214 = 214,
    Reserved215 = 215,
    Reserved216 = 216,
    Reserved217 = 217,
    Reserved218 = 218,
    Reserved219 = 219,
    Reserved220 = 220,
    Reserved221 = 221,
    Reserved222 = 222,
    Reserved223 = 223,
    Reserved224 = 224,
    Reserved225 = 225,
    Reserved226 = 226,
    Reserved227 = 227,
    Reserved228 = 228,
    Reserved229 = 229,
    Reserved230 = 230,
    Reserved231 = 231,
    Reserved232 = 232,
    Reserved233 = 233,
    Reserved234 = 234,
    Reserved235 = 235,
    Reserved236 = 236,
    Reserved237 = 237,
    Reserved238 = 238,
    Reserved239 = 239,
    Reserved240 = 240,
    Reserved241 = 241,
    Reserved242 = 242,
    Reserved243 = 243,
    Reserved244 = 244,
    Reserved245 = 245,
    Reserved246 = 246,
    Reserved247 = 247,
    Reserved248 = 248,
    Reserved249 = 249,
    Reserved250 = 250,
    Reserved251 = 251,
    Reserved252 = 252,
    Reserved253 = 253,
    Reserved254 = 254,
    Reserved255 = 255,
}

pub type Result<T, E = OcallError> = core::result::Result<T, E>;
pub trait OcallEnv {
    fn put_return(&mut self, rv: Vec<u8>) -> usize;
    fn take_return(&mut self) -> Option<Vec<u8>>;
    fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()>;
    fn slice_from_vm(&self, ptr: IntPtr, len: IntPtr) -> Result<&[u8]>;
    fn slice_from_vm_mut(&self, ptr: IntPtr, len: IntPtr) -> Result<&mut [u8]>;
}

extern "C" {
    pub fn sidevm_ocall(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntRet;
    pub fn sidevm_ocall_fast_return(
        task_id: i32,
        func_id: i32,
        p0: IntPtr,
        p1: IntPtr,
        p2: IntPtr,
        p3: IntPtr,
    ) -> IntRet;
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

fn empty_buffer() -> Buffer {
    Default::default()
}

fn alloc_buffer(size: usize) -> Buffer {
    let mut buf = Buffer::default();
    buf.0.resize(size, 0_u8);
    buf
}

thread_local! {
    static CURRENT_TASK: std::cell::Cell<i32>  = Default::default();
}

pub fn current_task() -> i32 {
    CURRENT_TASK.with(|id| id.get())
}

pub fn set_current_task(task_id: i32) {
    CURRENT_TASK.with(|id| id.set(task_id))
}
#[cfg(test)]
mod tests;
