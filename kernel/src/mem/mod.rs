//! QuadOS memory settings mod.
//!
//! Includes rust [GlobalAllocator], os frameallocator. etc.

mod allocator;
pub mod frames;

/// Initialize the memory mod.
pub fn init() {
    frames::init_frames();
}
