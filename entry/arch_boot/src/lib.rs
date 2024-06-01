//! The boot entry of the whole kernel, which will initialize the kernel and start the first user process.
//!
//! To ensure the one-way dependence on the calling relationship, the boot module is moved to the top level of the project.
#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]
#[cfg(feature = "alloc")]
mod alloc;

mod platform;

#[cfg(feature = "smp")]
mod mp;
