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
    get_mem_areas().into_iter().for_each(|(start, mut size)| {
        // Align up end symbol's address with PAGE_SIZE.
        let phys_end = (sym_addr!(end) + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;

        let frame_start = if phys_end >= start && phys_end <= size {
            (start - VIRT_ADDR_START) / PAGE_SIZE
        } else {
            size -= phys_end - start;
            (phys_end - VIRT_ADDR_START) / PAGE_SIZE
        };

        FRAME_ALLOCATOR
            .lock()
            .add_frame(frame_start, frame_start + size / PAGE_SIZE);
        log::debug!("frame memory {:#x} - {:#x}", start, start + size);
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
        FRAME_ALLOCATOR.lock().dealloc(self.0.as_num(), 1)
    }
}
