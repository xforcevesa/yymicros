pub fn init_allocator() {
    use axhal::mem::{memory_regions, phys_to_virt, MemRegionFlags};
    axlog::info!("Initialize global memory allocator...");
    axlog::info!("  use {} allocator.", axalloc::global_allocator().name());

    let mut max_region_size = 0;
    let mut max_region_paddr = 0.into();
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.size > max_region_size {
            max_region_size = r.size;
            max_region_paddr = r.paddr;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
            axlog::ax_println!("init_allocator: [{:#x}, {:#x})", r.paddr, r.paddr + r.size);
            axalloc::global_init(phys_to_virt(r.paddr).as_usize(), r.size);
            break;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr != max_region_paddr {
            axlog::ax_println!("add_heap_memory: [{:#x}, {:#x})", r.paddr, r.paddr + r.size);
            axalloc::global_add_memory(phys_to_virt(r.paddr).as_usize(), r.size)
                .expect("add heap memory region failed");
        }
    }
}
