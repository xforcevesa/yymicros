extern crate alloc;
use core::{alloc::Layout, ptr::NonNull};

use memory_addr::VirtAddr;

pub(crate) struct TaskStack {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl TaskStack {
    pub fn alloc(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).unwrap();
        Self {
            ptr: NonNull::new(unsafe { alloc::alloc::alloc(layout) }).unwrap(),
            layout,
        }
    }

    pub const fn top(&self) -> VirtAddr {
        unsafe { core::mem::transmute(self.ptr.as_ptr().add(self.layout.size())) }
    }

    // #[cfg(feature = "monolithic")]
    // /// 获取内核栈第一个压入的trap上下文，防止出现内核trap嵌套
    // pub fn get_first_trap_frame(&self) -> *mut TrapFrame {
    //     (self.top().as_usize() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame
    // }
}

impl Drop for TaskStack {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}
