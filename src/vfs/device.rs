use core::any::Any;
/// Trait for block devices
/// which reads and writes data in the unit of blocks
pub trait BlockDevice: Send + Sync + Any {
    /// Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> bool;
    /// Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]) -> bool;
    /// Get the total number of blocks in the device
    fn block_count(&self) -> usize;
    /// Get the block size in bytes
    fn block_size(&self) -> usize;
}
