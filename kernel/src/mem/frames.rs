//! Frame Allocator mod.
//!
//!

use alloc::vec::Vec;
use buddy_system_allocator::FrameAllocator;
use polyhal::{
    common::get_mem_areas,
    utils::{LazyInit, MutexNoIrq},
    PhysPage,
};

use crate::{
    config::{PAGE_SIZE, VIRT_ADDR_START},
    sym_addr,
};

static FRAME_ALLOCATOR: LazyInit<MutexNoIrq<FrameAllocator>> = LazyInit::new();

/// Init the [FRAME_ALLOCATOR].
pub(super) fn init_frames() {
    FRAME_ALLOCATOR.init_by(MutexNoIrq::new(FrameAllocator::new()));
    get_mem_areas()
        .into_iter()
        .for_each(|(mut start, mut size)| {
            // Align up end symbol's address with PAGE_SIZE.
            let phys_end = (sym_addr!(end) + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

            // Get the new start address.
            start = match phys_end >= start && phys_end <= start + size {
                true => {
                    size -= phys_end - start;
                    phys_end - VIRT_ADDR_START
                }
                false => start - VIRT_ADDR_START,
            };

            // Ensure that all memory is zeroed.
            unsafe {
                let per_len = core::mem::size_of::<u128>();
                core::slice::from_raw_parts_mut(start as *mut u128, size / per_len).fill(0);
            }

            FRAME_ALLOCATOR
                .lock()
                .add_frame(start / PAGE_SIZE, (start + size) / PAGE_SIZE);
            log::debug!("frame memory {:#010x} - {:#010x}", start, start + size);
        });
}

/// Allocate a Physical Page from the [FRAME_ALLOCATOR].
///
/// WARN: You should release the [PhysPage] manually.
pub unsafe fn alloc_page_raw() -> PhysPage {
    FRAME_ALLOCATOR.lock().alloc(1).map(PhysPage::new).unwrap()
}

/// Allocate a Physical Page from the [FRAME_ALLOCATOR].
///
/// WARN: You should release the [PhysPage] manually.
pub unsafe fn alloc_pages_raw(count: usize) -> PhysPage {
    FRAME_ALLOCATOR
        .lock()
        .alloc(count)
        .map(PhysPage::new)
        .unwrap()
}

/// Deallocate a physical page from the [FRAME_ALLOCATOR].
pub unsafe fn dealloc_pages_raw(start: PhysPage, count: usize) {
    // Ensure the allocate page is clean
    (0..count).for_each(|i| (start + i).drop_clear());
    FRAME_ALLOCATOR.lock().dealloc(start.as_num(), count)
}

/// Allocate a page from the [FRAME_ALLOCATOR].
pub fn alloc_page() -> FrameTracker {
    FRAME_ALLOCATOR
        .lock()
        .alloc(1)
        .map(PhysPage::new)
        .map(FrameTracker)
        .unwrap()
}

/// Allocate count pages from the [FRAME_ALLOCATOR].
#[allow(dead_code)]
pub fn alloc_pages(count: usize) -> Vec<FrameTracker> {
    // Start page of the [FRAME_ALLOCATOR]
    let start = FRAME_ALLOCATOR.lock().alloc(count).unwrap();

    (start..start + count)
        .into_iter()
        .map(PhysPage::new)
        .map(FrameTracker)
        .collect()
}

#[derive(Debug)]
pub struct FrameTracker(pub PhysPage);

/// Implement the [Drop] trait for [FrameTracker]
impl Drop for FrameTracker {
    fn drop(&mut self) {
        self.0.drop_clear();
        FRAME_ALLOCATOR.lock().dealloc(self.0.as_num(), 1)
    }
}
