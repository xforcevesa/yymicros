use core::any::Any;
use core::cmp::min;
use crate::driver::block::BLOCK_DEVICE;
use crate::sync::UPSafeCell;
use crate::vfs::err::{DevResult, DevError};
use alloc::sync::Arc;
use lazy_static::*;
use alloc::vec::Vec;
use alloc::vec;

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

#[derive(Clone)]
/// A disk device with a cursor.
pub struct Disk {
    // Vulnerability: mutable reference to a trait object
    dev: &'static dyn BlockDevice,
    info: UPSafeCell<Info>
}

#[derive(Clone)]
struct Info {
    block_id: u64,
    offset: usize,
    block_size: usize,
    block_count: usize
}

#[allow(unused)]
impl Disk {
    /// Create a new disk.
    pub fn new(dev: &'static dyn BlockDevice) -> Self {
        Self {
            dev: dev,
            info: unsafe {
                UPSafeCell::new(Info {
                    block_id: 0,
                    offset: 0,
                    block_size: dev.block_size(),
                    block_count: dev.block_count()
                })
            }
        }
    }

    /// Get the size of the disk.
    pub fn size(self: &Arc<Self>) -> u64 {
        let info = self.info.exclusive_access();
        (info.block_count * info.block_size) as u64
    }

    /// Get the size of the disk.
    pub fn size_self(&self) -> u64 {
        let info = self.info.exclusive_access();
        (info.block_count * info.block_size) as u64
    }

    /// Get the size of the disk.
    pub fn block_size(self: &Arc<Self>) -> u64 {
        let info = self.info.exclusive_access();
        info.block_size as u64
    }

    /// Get the position of the cursor.
    pub fn position(self: &Arc<Self>) -> u64 {
        let info = self.info.exclusive_access();
        info.block_id * info.block_size as u64 + info.offset as u64
    }

    /// Get the position of the cursor.
    pub fn position_self(&self) -> u64 {
        let info = self.info.exclusive_access();
        info.block_id * info.block_size as u64 + info.offset as u64
    }

    /// Set the position of the cursor.
    pub fn set_position(self: &Arc<Self>, pos: u64) {
        let mut info = self.info.exclusive_access();
        info.block_id = pos / info.block_size as u64;
        info.offset = pos as usize % info.block_size;
    }

    /// Set the position of the cursor.
    pub fn set_position_self(&self, pos: u64) {
        let mut info = self.info.exclusive_access();
        info.block_id = pos / info.block_size as u64;
        info.offset = pos as usize % info.block_size;
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(self: &mut Arc<Self>, buf: &mut [u8]) -> DevResult<usize> {
        let mut info = self.info.exclusive_access();
        let position = info.block_id * info.block_size as u64 + info.offset as u64;
        let size = (info.block_count * info.block_size) as u64;
        let buf = if position as usize + buf.len() >= size as usize {
            &mut buf[0..(size - position) as usize]
        } else {
            buf
        };
        let read_size = if info.offset == 0 && buf.len() >= info.block_size {
            // whole block
            if !self.dev
                .read_block(info.block_id as usize, &mut buf[0..info.block_size]) {
                    return Err(DevError::ReadError)
                }
            info.block_id += 1;
            info.block_size
        } else {
            // partial block
            let mut data = vec![0u8; info.block_size];
            let start = info.offset;
            let count = buf.len().min(info.block_size - info.offset);
            if start > info.block_size {
                info!("block size: {} start {}", info.block_size, start);
            }

            if !self.dev.read_block(info.block_id as usize, &mut data.as_mut_slice()) {
                return Err(DevError::ReadError)
            }
            buf[..count].copy_from_slice(&data[start..start + count]);

            info.offset += count;
            if info.offset >= info.block_size {
                info.block_id += 1;
                info.offset -= info.block_size;
            }
            count
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(self: &mut Arc<Self>, buf: &[u8]) -> DevResult<usize> {
        let mut info = self.info.exclusive_access();
        let write_size = if info.offset == 0 && buf.len() >= info.block_size {
            // whole block
            if !self.dev.write_block(info.block_id as usize, &buf[0..info.block_size]) {
                return Err(DevError::WriteError)
            }
            info.block_id += 1;
            info.block_size
        } else {
            // partial block
            let mut data = vec![0u8; info.block_size];
            let start = info.offset;
            let count = buf.len().min(info.block_size - info.offset);

            if !self.dev.read_block(info.block_id as usize, data.as_mut_slice()) {
                return Err(DevError::ReadError)
            }
            data[start..start + count].copy_from_slice(&buf[..count]);
            if !self.dev.write_block(info.block_id as usize, data.as_slice()) {
                return Err(DevError::WriteError)
            }

            info.offset += count;
            if info.offset >= info.block_size {
                info.block_id += 1;
                info.offset -= info.block_size;
            }
            count
        };
        Ok(write_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one_self(&mut self, buf: &[u8]) -> DevResult<usize> {
        let mut info = self.info.exclusive_access();
        let write_size = if info.offset == 0 && buf.len() >= info.block_size {
            // whole block
            if !self.dev.write_block(info.block_id as usize, &buf[0..info.block_size]) {
                return Err(DevError::WriteError)
            }
            info.block_id += 1;
            info.block_size
        } else {
            // partial block
            let mut data = vec![0u8; info.block_size];
            let start = info.offset;
            let count = buf.len().min(info.block_size - info.offset);

            if !self.dev.read_block(info.block_id as usize, data.as_mut_slice()) {
                return Err(DevError::ReadError)
            }
            data[start..start + count].copy_from_slice(&buf[..count]);
            if !self.dev.write_block(info.block_id as usize, data.as_slice()) {
                return Err(DevError::WriteError)
            }

            info.offset += count;
            if info.offset >= info.block_size {
                info.block_id += 1;
                info.offset -= info.block_size;
            }
            count
        };
        Ok(write_size)
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one_self(&mut self, buf: &mut [u8]) -> DevResult<usize> {
        let mut info = self.info.exclusive_access();
        let position = info.block_id * info.block_size as u64 + info.offset as u64;
        let size = (info.block_count * info.block_size) as u64;
        let buf = if position as usize + buf.len() >= size as usize {
            &mut buf[0..(size - position) as usize]
        } else {
            buf
        };
        let read_size = if info.offset == 0 && buf.len() >= info.block_size {
            // whole block
            if !self.dev
                .read_block(info.block_id as usize, &mut buf[0..info.block_size]) {
                    return Err(DevError::ReadError)
                }
            info.block_id += 1;
            info.block_size
        } else {
            // partial block
            let mut data = vec![0u8; info.block_size];
            let start = info.offset;
            let count = buf.len().min(info.block_size - info.offset);
            if start > info.block_size {
                info!("block size: {} start {}", info.block_size, start);
            }

            if !self.dev.read_block(info.block_id as usize, &mut data.as_mut_slice()) {
                return Err(DevError::ReadError)
            }
            buf[..count].copy_from_slice(&data[start..start + count]);

            info.offset += count;
            if info.offset >= info.block_size {
                info.block_id += 1;
                info.offset -= info.block_size;
            }
            count
        };
        Ok(read_size)
    }

    /// Read a single block starting from the specified offset.
    #[allow(unused)]
    pub fn read_offset(&mut self, offset: usize) -> Vec<u8> {
        let info = self.info.exclusive_access();
        let block_id = offset / info.block_size;
        let mut block_data = vec![0u8; info.block_size];
        assert!(self.dev
            .read_block(block_id as usize, &mut block_data));
        block_data
    }

    /// Write single block starting from the specified offset.
    #[allow(unused)]
    pub fn write_offset(&mut self, offset: usize, buf: &[u8]) -> DevResult<usize> {
        let info = self.info.exclusive_access();
        assert!(
            buf.len() == info.block_size,
            "Buffer length must be equal to BLOCK_SIZE"
        );
        assert!(offset % info.block_size == 0);
        let block_id = offset / info.block_size;
        self.dev.write_block(block_id as usize, buf);
        Ok(buf.len())
    }
}

lazy_static! {
    /// The global block device driver instance: BLOCK_DEVICE with BlockDevice trait
    pub static ref DISK_DEVICE: Arc<Disk> = Arc::new(Disk::new(BLOCK_DEVICE.as_ref()));
}

/// Test the block device
pub fn disk_device_test() {
    let mut disk_device = DISK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    let disk_size = disk_device.size();
    let block_size = disk_device.block_size();
    
    assert_ne!(disk_size, 0);
    assert_ne!(block_size, 0);
    println!("Disk Size: {}, Block Size: {}", disk_size, block_size);
    disk_device.set_position(0);
    let mut read_size: usize = 0;
    let target = min(16384, (disk_size / 4) as usize);
    while read_size < target {
        let position = disk_device.position();
        match disk_device.read_one(&mut read_buffer) {
            Ok(size) => {
                read_size += size;
            },
            Err(_) => panic!("Read Error in disk_device_test: {}", position)
        }
        
        if position % 1400 < read_buffer.len() as u64 {
            println!("Test position: {}, read data: {:x?}", position, &read_buffer[0..8]);
        }
        for index in 0..512 {
            write_buffer[index] = read_buffer[index];
        }
        disk_device.set_position(position);
        match disk_device.write_one(&mut read_buffer) {
            Ok(_) => {},
            Err(_) => panic!("Write Error in disk_device_test: {}", position)
        }
    }
    println!("block device test passed!");
    disk_device.set_position(0);
}

