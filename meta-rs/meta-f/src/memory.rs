use std::alloc::Layout;

pub(crate) struct Memory {
    memory: *mut u8,
    next: *mut u8,
    limit: *mut u8,
    size: usize,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        unsafe {
            let buf = std::alloc::alloc(Layout::from_size_align(size, 8).unwrap());
            Self {
                memory: buf,
                next: buf,
                limit: buf.add(size),
                size,
            }
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut u64 {
        #[allow(clippy::cast_ptr_alignment)]
        let ptr = self.next as *mut u64;
        self.next = unsafe { self.next.add(size) };
        ptr
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.memory, Layout::from_size_align(self.size, 8).unwrap());
        }
    }
}
