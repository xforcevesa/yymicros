//! virtio_blk device driver

mod virtio_blk;

#[allow(unused)]
pub use virtio_blk::VirtIOBlock;

use alloc::sync::Arc;
use crate::vfs::BlockDevice;
use lazy_static::*;

type BlockDeviceImpl = virtio_blk::VirtIOBlock;

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
