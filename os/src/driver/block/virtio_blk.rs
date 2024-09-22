use super::{BlkConfig, BlockDevice, VIRTIO0};
use crate::mem::{
    frame_alloc, frame_dealloc, kernel_token, FrameTracker, PageTable, PhysAddr, PhysPageNum,
    StepByOne, VirtAddr,
};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;
use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

/// VirtIOBlock device driver structure for virtio_blk device
pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, VirtioHal>>);

lazy_static! {
    static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

impl BlockDevice for VirtIOBlock {
    
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> bool {
        match self.0
            .exclusive_access()
            .read_block(block_id, buf) {
                Ok(_) => true,
                Err(_) => false,
            }
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> bool {
        match self.0
            .exclusive_access()
            .write_block(block_id, buf) {
                Ok(_) => true,
                Err(_) => false,
            }
    }
    
    fn block_count(&self) -> usize {
        let header = unsafe { & *(VIRTIO0 as *mut VirtIOHeader) };
        let config = unsafe { & *(header.config_space() as *const BlkConfig) };
        config.capacity.read() as usize
    }
    
    fn block_size(&self) -> usize {
        let header = unsafe { & *(VIRTIO0 as *mut VirtIOHeader) };
        let config = unsafe { & *(header.config_space() as *const BlkConfig) };
        config.blk_size.read() as usize
    }
    
}

impl VirtIOBlock {
    /// Create a new VirtIOBlock driver with VIRTIO0 base_addr for virtio_blk device
    pub fn new() -> Self {
        unsafe {
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO0 as *mut VirtIOHeader)).unwrap(),
            ))
        }
    }
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let mut ppn_base = PhysPageNum(0);
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: PhysAddr = ppn_base.into();
        pa.0
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base: PhysPageNum = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        PageTable::from_token(kernel_token())
            .translate_va(VirtAddr::from(vaddr))
            .unwrap()
            .0
    }
}
