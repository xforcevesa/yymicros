//! Slab allocator for `no_std` systems. It uses multiple slabs with blocks of
//! different sizes and a [buddy_system_allocator] for blocks larger than 4096
//! bytes.
//!
//! It's based on <https://github.com/weclaw1/slab_allocator>.
//!
//! [buddy_system_allocator]: https://docs.rs/buddy_system_allocator/latest/buddy_system_allocator/

#![feature(allocator_api)]
#![no_std]

extern crate alloc;
extern crate buddy_system_allocator;

use alloc::alloc::{AllocError, Layout};
use allocator::{BaseAllocator, ByteAllocator};
use core::ptr::NonNull;

#[cfg(test)]
mod tests;
mod tlsf;
mod slab;
use slab::Slab;

const SET_SIZE: usize = 2;
const MIN_HEAP_SIZE: usize = 0x1;

enum HeapAllocator {
    Slab128Bytes,
    Slab256Bytes,
    Slab512Bytes,
    Slab1024Bytes,
    Slab2048Bytes,
    Slab4096Bytes,
    TlsfAllocator,
}

/// A fixed size heap backed by multiple slabs with blocks of different sizes.
/// Allocations over 4096 bytes are served by linked list allocator.
pub struct Heap {
    slab_128_bytes: Slab<128>,
    slab_256_bytes: Slab<256>,
    slab_512_bytes: Slab<512>,
    slab_1024_bytes: Slab<1024>,
    slab_2048_bytes: Slab<2048>,
    slab_4096_bytes: Slab<4096>,
    tlsf_allocator: tlsf::TlsfByteAllocator,
}

impl Heap {
    /// Creates a new heap with the given `heap_start_addr` and `heap_size`. The start address must be valid
    /// and the memory in the `[heap_start_addr, heap_start_addr + heap_size)` range must not be used for
    /// anything else.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn new(heap_start_addr: usize, heap_size: usize) -> Heap {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size >= MIN_HEAP_SIZE,
            "Heap size should be greater or equal to minimum heap size"
        );
        assert!(
            heap_size % MIN_HEAP_SIZE == 0,
            "Heap size should be a multiple of minimum heap size"
        );
        Heap {
            slab_128_bytes: Slab::<128>::new(0, 0),
            slab_256_bytes: Slab::<256>::new(0, 0),
            slab_512_bytes: Slab::<512>::new(0, 0),
            slab_1024_bytes: Slab::<1024>::new(0, 0),
            slab_2048_bytes: Slab::<2048>::new(0, 0),
            slab_4096_bytes: Slab::<4096>::new(0, 0),
            tlsf_allocator: tlsf::TlsfByteAllocator::new(),
        }
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn add_memory(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0,
            "Add Heap size should be a multiple of page size"
        );
        let _ = self.tlsf_allocator.add_memory(heap_start_addr, heap_size);
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    unsafe fn _grow(&mut self, mem_start_addr: usize, mem_size: usize, slab: HeapAllocator) {
        match slab {
            HeapAllocator::Slab128Bytes => self.slab_128_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab256Bytes => self.slab_256_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab512Bytes => self.slab_512_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab1024Bytes => self.slab_1024_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab2048Bytes => self.slab_2048_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::Slab4096Bytes => self.slab_4096_bytes.grow(mem_start_addr, mem_size),
            HeapAllocator::TlsfAllocator => self
                .add_memory(mem_start_addr, mem_size),
        }
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    /// This function finds the slab of lowest size which can still accommodate the given chunk.
    /// The runtime is in `O(1)` for chunks of size <= 4096, and `O(n)` when chunk size is > 4096,
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab128Bytes => self
                .slab_128_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::Slab256Bytes => self
                .slab_256_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::Slab512Bytes => self
                .slab_512_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::Slab1024Bytes => self
                .slab_1024_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::Slab2048Bytes => self
                .slab_2048_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::Slab4096Bytes => self
                .slab_4096_bytes
                .allocate(layout, &mut self.tlsf_allocator),
            HeapAllocator::TlsfAllocator => self
                .tlsf_allocator
                .alloc(layout)
                .map(|ptr| ptr.as_ptr() as usize)
                .map_err(|_| AllocError),
        }
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// This function finds the slab which contains address of `ptr` and adds the blocks beginning
    /// with `ptr` address to the list of free blocks.
    /// This operation is in `O(1)` for blocks <= 4096 bytes and `O(n)` for blocks > 4096 bytes.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn deallocate(&mut self, ptr: usize, layout: Layout) {
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab128Bytes => self.slab_128_bytes.deallocate(ptr),
            HeapAllocator::Slab256Bytes => self.slab_256_bytes.deallocate(ptr),
            HeapAllocator::Slab512Bytes => self.slab_512_bytes.deallocate(ptr),
            HeapAllocator::Slab1024Bytes => self.slab_1024_bytes.deallocate(ptr),
            HeapAllocator::Slab2048Bytes => self.slab_2048_bytes.deallocate(ptr),
            HeapAllocator::Slab4096Bytes => self.slab_4096_bytes.deallocate(ptr),
            HeapAllocator::TlsfAllocator => self
                .tlsf_allocator
                .dealloc(NonNull::new(ptr as *mut u8).unwrap(), layout),
        }
    }

    /// Returns bounds on the guaranteed usable size of a successful
    /// allocation created with the specified `layout`.
    pub fn usable_size(&self, layout: Layout) -> (usize, usize) {
        match Heap::layout_to_allocator(&layout) {
            HeapAllocator::Slab128Bytes => (layout.size(), 128),
            HeapAllocator::Slab256Bytes => (layout.size(), 256),
            HeapAllocator::Slab512Bytes => (layout.size(), 512),
            HeapAllocator::Slab1024Bytes => (layout.size(), 1024),
            HeapAllocator::Slab2048Bytes => (layout.size(), 2048),
            HeapAllocator::Slab4096Bytes => (layout.size(), 4096),
            HeapAllocator::TlsfAllocator => (layout.size(), layout.size()),
        }
    }

    /// Finds allocator to use based on layout size and alignment
    fn layout_to_allocator(layout: &Layout) -> HeapAllocator {
        if layout.size() > 4096 {
            HeapAllocator::TlsfAllocator
        } else if layout.size() <= 128 && layout.align() <= 128 {
            HeapAllocator::Slab128Bytes
        } else if layout.size() <= 256 && layout.align() <= 256 {
            HeapAllocator::Slab256Bytes
        } else if layout.size() <= 512 && layout.align() <= 512 {
            HeapAllocator::Slab512Bytes
        } else if layout.size() <= 1024 && layout.align() <= 1024 {
            HeapAllocator::Slab1024Bytes
        } else if layout.size() <= 2048 && layout.align() <= 2048 {
            HeapAllocator::Slab2048Bytes
        } else {
            HeapAllocator::Slab4096Bytes
        }
    }

    /// Returns total memory size in bytes of the heap.
    pub fn total_bytes(&self) -> usize {
        self.slab_128_bytes.total_blocks() * 128
            + self.slab_256_bytes.total_blocks() * 256
            + self.slab_512_bytes.total_blocks() * 512
            + self.slab_1024_bytes.total_blocks() * 1024
            + self.slab_2048_bytes.total_blocks() * 2048
            + self.slab_4096_bytes.total_blocks() * 4096
            + self.tlsf_allocator.total_bytes()
    }

    /// Returns allocated memory size in bytes.
    pub fn used_bytes(&self) -> usize {
        self.slab_128_bytes.used_blocks() * 128
            + self.slab_256_bytes.used_blocks() * 256
            + self.slab_512_bytes.used_blocks() * 512
            + self.slab_1024_bytes.used_blocks() * 1024
            + self.slab_2048_bytes.used_blocks() * 2048
            + self.slab_4096_bytes.used_blocks() * 4096
            + self.tlsf_allocator.used_bytes()
    }

    /// Returns available memory size in bytes.
    pub fn available_bytes(&self) -> usize {
        self.total_bytes() - self.used_bytes()
    }
}
