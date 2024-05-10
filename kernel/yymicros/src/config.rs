// Linear mapping
pub const PHYSICAL_MEMORY_OFFSET: usize = 0xFFFF_FFFF_4000_0000;

pub const KERNEL_OFFSET: usize = 0xFFFF_FFFF_C000_0000;

pub const KERNEL_HEAP_SIZE: usize = 0x0080_0000;

pub const MEMORY_OFFSET: usize = 0x8000_0000;
// TODO: get memory end from device tree
pub const MEMORY_END: usize = 0x8800_0000;

// TODO: rv64 `sh` and `ls` will crash if stack top > 0x80000000 ???
pub const USER_STACK_OFFSET: usize = 0x40000000 - USER_STACK_SIZE;
pub const USER_STACK_SIZE: usize = 0x10000;

pub const KSEG2_START: usize = 0xffff_fe80_0000_0000;

pub const MAX_DTB_SIZE: usize = 0x2000;

pub const ARCH: &'static str = "riscv64";