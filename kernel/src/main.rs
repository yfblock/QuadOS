#![no_std]
#![no_main]

extern crate alloc;

use mem::frames::{self, FrameTracker};
use polyhal::common::PageAlloc;

mod config;
mod lang_items;
mod mem;
mod utils;

struct PageAllocator;

impl PageAlloc for PageAllocator {
    /// Allocate a Physical Page.
    fn alloc(&self) -> polyhal::PhysPage {
        unsafe { frames::alloc_page_raw() }
    }

    /// Deallocate a Physical Page.
    fn dealloc(&self, ppn: polyhal::PhysPage) {
        unsafe {
            drop(FrameTracker::new(ppn));
        }
    }
}

/// Kernel Entry Point
#[polyhal::arch_entry]
fn main(hart_id: usize) {
    log::info!("hart_id: {}", hart_id);

    polyhal::common::init(&PageAllocator);
    mem::init();
}
