#[derive(Debug, Clone, Copy)]
pub struct Stride {
    stride: usize,
    priority: usize,
    #[allow(unused)]
    big_stride: usize
}

use core::cmp::{PartialEq, Ordering};

use super::TaskControlBlock;

impl PartialEq for Stride {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Await for check.
        match PartialOrd::partial_cmp(&self.stride, &other.stride) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            Some(Ordering::Less) => Some(Ordering::Greater),
            Some(Ordering::Greater) => Some(Ordering::Less),
            None => None
        }
    }
}

impl Ord for Stride {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match PartialOrd::partial_cmp(&self.stride, &other.stride) {
            Some(t) => t,
            None => Ordering::Equal
        }
    }
}

impl Eq for Stride {
    
}

impl Stride {
    pub fn new() -> Self {
        Self {
            stride: 0,
            priority: 16,
            big_stride: 0x1000
        }
    }
    pub fn set_priority(&mut self, p: usize) {
        self.priority = p;
    }
    pub fn accumulate(&mut self) {
        // Potential BUG here.
        self.stride += self.big_stride / self.priority
    }
}

impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.stride, &other.stride)
    }
}

impl PartialEq for TaskControlBlock {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match PartialOrd::partial_cmp(&self.stride, &other.stride) {
            Some(t) => t,
            None => Ordering::Equal
        }
    }
}

impl Eq for TaskControlBlock {
    
}
