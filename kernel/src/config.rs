/// Default Heap Size.
pub const DEFAULT_HEAP_SIZE: usize = 0x20_0000;
/// Address offset of the higher half kernel.
pub use polyhal::consts::VIRT_ADDR_START;
/// The size of the last level page.
pub use polyhal::pagetable::PAGE_SIZE;

/// Need to align with 0x1000.
pub const DEFAULT_USER_STACK_SIZE: usize = 0x8000;

/// Default stack top address for the user stack.
pub const DEFAULT_USER_STACK_TOP: usize = 0x1_0000_0000;

/// Default alignment for the user stack and memory.
pub const ALIGN_SIZE: usize = core::mem::size_of::<usize>();
