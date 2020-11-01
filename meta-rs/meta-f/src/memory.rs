use std::alloc::Layout;
use std::mem::{align_of, size_of};

use crate::vm::Value;

pub(crate) struct Memory {
    memory: *mut Value,
    next: *mut Value,
    limit: *mut Value,
    count: usize,
}

impl Memory {
    pub fn new(count: usize) -> Self {
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            let buf = std::alloc::alloc(Self::layout(count)) as *mut Value;
            Self {
                memory: buf,
                next: buf,
                limit: buf.add(count),
                count,
            }
        }
    }

    pub fn allocate_cells(&mut self, n_cells: u64) -> *mut Value {
        let ptr = self.next;
        self.next = unsafe { self.next.add(n_cells as usize) };
        ptr
    }

    fn layout(count: usize) -> Layout {
        unsafe {
            // Layout::repeat() is nightly-only
            Layout::from_size_align_unchecked(size_of::<Value>() * count, align_of::<Value>())
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.memory as *mut u8, Self::layout(self.count));
        }
    }
}
