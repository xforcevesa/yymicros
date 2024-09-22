//! virtio_blk device driver
mod virtio_gpu;
mod virtio_blk;

#[allow(unused)]
pub use virtio_blk::VirtIOBlock;

pub use virtio_gpu::gpu_test;

use alloc::sync::Arc;
use crate::vfs::BlockDevice;
use lazy_static::*;
use volatile::Volatile;

type BlockDeviceImpl = virtio_blk::VirtIOBlock;

/// The base address of control registers in Virtio_Block device
#[allow(unused)]
const VIRTIO0: usize = 0x10001000;
#[allow(unused)]
const VIRTIO1: usize = 0x10002000;
#[allow(unused)]
const VIRTIO2: usize = 0x10003000;
#[allow(unused)]
const VIRTIO3: usize = 0x10004000;
#[allow(unused)]
const VIRTIO4: usize = 0x10005000;

#[repr(C)]
#[derive(Debug)]
struct BlkConfig {
    /// Number of 512 Bytes sectors
    capacity: Volatile<u64>,
    size_max: Volatile<u32>,
    seg_max: Volatile<u32>,
    cylinders: Volatile<u16>,
    heads: Volatile<u8>,
    sectors: Volatile<u8>,
    blk_size: Volatile<u32>,
    physical_block_exp: Volatile<u8>,
    alignment_offset: Volatile<u8>,
    min_io_size: Volatile<u16>,
    opt_io_size: Volatile<u32>,
    // ... ignored
}

lazy_static! {
    /// The global block device driver instance: BLOCK_DEVICE with BlockDevice trait
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
/// Test the block device
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    let block_count = block_device.block_count();
    let block_size = block_device.block_size();
    println!("Block Count: {}, Block Size: {}", block_count, block_size);
    for i in 0..block_count {
        assert!(block_device.read_block(i as usize, &mut read_buffer));
        if i % 40 == 0 {
            println!("Test block {}, read data: {:x?}", i, &read_buffer[0..8]);
        }
        for index in 0..512 {
            write_buffer[index] = read_buffer[index];
        }
        assert!(block_device.write_block(i as usize, &write_buffer));
    }
    println!("block device test passed!");
}
