// snap-os silverback — no_std microkernel core
// Tests run with std; the kernel binary target will be no_std.
#![cfg_attr(not(test), no_std)]

pub mod capability;
pub mod memory;
