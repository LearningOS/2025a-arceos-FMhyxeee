#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocError};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: AtomicUsize,
    end: AtomicUsize,
    b_pos: AtomicUsize,  // bytes allocation position (grows forward)
    p_pos: AtomicUsize,  // pages allocation position (grows backward)
    alloc_count: AtomicUsize,  // number of active allocations
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
            b_pos: AtomicUsize::new(0),
            p_pos: AtomicUsize::new(0),
            alloc_count: AtomicUsize::new(0),
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        let end = start + size;
        self.start.store(start, Ordering::Relaxed);
        self.end.store(end, Ordering::Relaxed);
        self.b_pos.store(start, Ordering::Relaxed);
        self.p_pos.store(end, Ordering::Relaxed);
        self.alloc_count.store(0, Ordering::Relaxed);
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        Ok(()) // For simplicity, we don't support adding memory after init
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        let current_b_pos = self.b_pos.load(Ordering::Relaxed);
        let current_p_pos = self.p_pos.load(Ordering::Relaxed);

        // Align the allocation position
        let aligned_pos = (current_b_pos + align - 1) & !(align - 1);
        let new_b_pos = aligned_pos + size;

        if new_b_pos > current_p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos.store(new_b_pos, Ordering::Relaxed);
        self.alloc_count.fetch_add(1, Ordering::Relaxed);

        Ok(unsafe {
            NonNull::new_unchecked(aligned_pos as *mut u8)
        })
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        // Decrement allocation count, but don't actually free memory for simplicity
        let old_count = self.alloc_count.fetch_sub(1, Ordering::Relaxed);
        if old_count == 1 {
            // This was the last allocation, reset bytes position
            let start = self.start.load(Ordering::Relaxed);
            self.b_pos.store(start, Ordering::Relaxed);
        }
    }

    fn total_bytes(&self) -> usize {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);
        end - start
    }

    fn used_bytes(&self) -> usize {
        let start = self.start.load(Ordering::Relaxed);
        let b_pos = self.b_pos.load(Ordering::Relaxed);
        b_pos - start
    }

    fn available_bytes(&self) -> usize {
        let b_pos = self.b_pos.load(Ordering::Relaxed);
        let p_pos = self.p_pos.load(Ordering::Relaxed);
        p_pos - b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let align_mask = (1 << align_pow2) - 1;
        let needed_size = num_pages * SIZE;

        let current_b_pos = self.b_pos.load(Ordering::Relaxed);
        let current_p_pos = self.p_pos.load(Ordering::Relaxed);

        // Align backward (pages grow backward)
        let aligned_pos = (current_p_pos & !align_mask) - needed_size;

        if aligned_pos < current_b_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos.store(aligned_pos, Ordering::Relaxed);
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are never freed in this allocator
    }

    fn total_pages(&self) -> usize {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);
        (end - start) / SIZE
    }

    fn used_pages(&self) -> usize {
        let end = self.end.load(Ordering::Relaxed);
        let p_pos = self.p_pos.load(Ordering::Relaxed);
        (end - p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        let b_pos = self.b_pos.load(Ordering::Relaxed);
        let p_pos = self.p_pos.load(Ordering::Relaxed);
        (p_pos - b_pos) / SIZE
    }
}