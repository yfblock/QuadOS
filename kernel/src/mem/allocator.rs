//! Rust Global Allocator implement.
//!
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use buddy_system_allocator::LockedHeap;
use polyhal::consts::VIRT_ADDR_START;

use crate::config::{DEFAULT_HEAP_SIZE, PAGE_SIZE};

use super::frames::alloc_pages_raw;

/// Rust Global Allocator implement.
#[global_allocator]
static GLOBAL_ALLOCATOR: HeapAllocator = HeapAllocator {
    data: [0u8; DEFAULT_HEAP_SIZE],
    heap: LockedHeap::new(),
};

/// Heap Allocator for QuadOS.
#[repr(align(4096))]
struct HeapAllocator {
    data: [u8; DEFAULT_HEAP_SIZE],
    heap: LockedHeap<32>,
}

/// Implement GlobalAlloc for HeapAllocator.
unsafe impl GlobalAlloc for HeapAllocator {
    /// Allocate the memory from the allocator.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get heap usage
        let (total, actual) = {
            let heap = self.heap.lock();
            (heap.stats_total_bytes(), heap.stats_alloc_actual())
        };

        // Supply heap allocator's memory
        if total == 0 {
            let mm_start = GLOBAL_ALLOCATOR.data.as_ptr() as usize;
            self.heap
                .lock()
                .add_to_heap(mm_start, mm_start + DEFAULT_HEAP_SIZE);
        } else if total - actual < layout.size() + PAGE_SIZE {
            let allocate_pages = (layout.size() + 10 * PAGE_SIZE - 1) / PAGE_SIZE;
            let mm_start = alloc_pages_raw(allocate_pages).to_addr() | VIRT_ADDR_START;
            log::debug!(
                "Allocate {:#x} - {:#x} from FRAMES for HEAP_Allocator",
                mm_start,
                mm_start + allocate_pages * PAGE_SIZE
            );
            self.heap
                .lock()
                .add_to_heap(mm_start, mm_start + allocate_pages * PAGE_SIZE);
        }

        // Allocate memory
        self.heap
            .lock()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    /// DeAllocate the memory from the allocator.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap
            .lock()
            .dealloc(NonNull::new_unchecked(ptr), layout)
    }
}
