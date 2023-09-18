//! This file is copied from crate sgx_isa which is part of Fortanix, we can not directly use
//! sgx_isa because it requires compilation target=fortanix-sgx.

#![allow(dead_code)]
#![allow(clippy::enum_variant_names)]

use bitflags::bitflags;
use core::arch::asm;
use core::mem::MaybeUninit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
enum Keyname {
    Einittoken = 0,
    Provision = 1,
    ProvisionSeal = 2,
    Report = 3,
    Seal = 4,
}

bitflags! {
    #[repr(C)]
    struct Keypolicy: u16 {
        const MRENCLAVE = 0b0000_0001;
        const MRSIGNER  = 0b0000_0010;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
enum Enclu {
    EReport = 0,
    EGetkey = 1,
    EEnter = 2,
    EResume = 3,
    EExit = 4,
    EAccept = 5,
    EModpe = 6,
    EAcceptcopy = 7,
}

#[repr(align(16))]
struct Align16<T>(T);

#[repr(align(512))]
struct Align512<T>(T);

/// Call the `EGETKEY` instruction to obtain a 128-bit secret key.
fn egetkey(request: &Align512<[u8; 512]>) -> Result<Align16<[u8; 16]>, u32> {
    unsafe {
        let mut out = MaybeUninit::uninit();
        let error;

        asm!(
            // rbx is reserved by LLVM
            "xchg %rbx, {0}",
            "enclu",
            "mov {0}, %rbx",
            inout(reg) request => _,
            inlateout("eax") Enclu::EGetkey as u32 => error,
            in("rcx") out.as_mut_ptr(),
            options(att_syntax, nostack),
        );

        match error {
            0 => Ok(out.assume_init()),
            err => Err(err),
        }
    }
}

#[repr(C, align(512))]
struct Keyrequest {
    keyname: u16,
    keypolicy: Keypolicy,
    isvsvn: u16,
    _reserved1: u16,
    cpusvn: [u8; 16],
    attributemask: [u64; 2],
    keyid: [u8; 32],
    miscmask: u32,
    _reserved2: [u8; 436],
}

impl Keyrequest {
    fn copy(&self) -> [u8; 512] {
        unsafe { *(self as *const Keyrequest as *const [u8; 512]) }
    }

    fn egetkey(&self) -> Result<Align16<[u8; 16]>, u32> {
        egetkey(&Align512(self.copy()))
    }
}

/// Derive a MRENCLAVE sealing key with the given `isvsvn`, `cpusvn` and `keyid`.
pub fn get_mrenclave_sealing_key(
    isvsvn: u16,
    cpusvn: [u8; 16],
    keyid: [u8; 32],
) -> Result<[u8; 16], u32> {
    let request = Keyrequest {
        keyname: Keyname::Seal as u16,
        keypolicy: Keypolicy::MRENCLAVE,
        isvsvn,
        _reserved1: 0,
        cpusvn,
        // Less mask to prevent the derived key changes during system upgrade.
        attributemask: [0x03, 0x00],
        keyid,
        miscmask: 0,
        _reserved2: [0; 436],
    };
    let key = request.egetkey()?;
    Ok(key.0)
}
