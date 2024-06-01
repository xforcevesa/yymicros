//! Platform-specific operations.

cfg_if::cfg_if! {
if #[cfg(all(target_arch = "x86_64", platform_family = "x86-pc"))] {
    mod x86_pc;
    pub use x86_pc::*;
} else if #[cfg(all(target_arch = "riscv64", platform_family = "riscv64-qemu-virt"))] {
    mod riscv64_qemu_virt;
    pub use riscv64_qemu_virt::*;
} else if #[cfg(all(target_arch = "aarch64", any(
    platform_family = "aarch64-qemu-virt",
    platform_family = "aarch64-raspi",
    platform_family = "aarch64-bsta1000b",
    platform_family = "aarch64-rk3588j")))]
{
    mod aarch64_common;
    pub use self::aarch64_common::*;
} else {
    mod dummy;
    pub use self::dummy::*;
}
}
