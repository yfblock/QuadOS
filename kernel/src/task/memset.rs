use alloc::collections::btree_map::BTreeMap;
use polyhal::{
    pagetable::PAGE_SIZE, MappingFlags, MappingSize, PageTable, PhysAddr, PhysPage, VirtAddr,
    VirtPage,
};

use crate::mem::frames::{alloc_page, FrameTracker};

/// User task memory manager.
pub struct MemSet(pub BTreeMap<VirtPage, FrameTracker>);

impl MemSet {
    /// Create a new memset.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Map a page to the specified virtual address for the given page table.
    pub fn map_page(&mut self, pt: PageTable, vpn: VirtPage) -> PhysPage {
        let tracker = alloc_page();
        let ppn = tracker.0;

        pt.map_page(vpn, tracker.0, MappingFlags::URWX, MappingSize::Page4KB);

        self.0.insert(vpn, tracker);
        ppn
    }

    /// Get the physical address for the given virtual page
    #[inline]
    pub fn vpn_to_ppn(&self, vpn: VirtPage) -> PhysPage {
        self.0[&vpn].0
    }

    /// Get the physical address for the given virtual address
    #[inline]
    pub fn vaddr_to_paddr(&self, vaddr: VirtAddr) -> PhysAddr {
        PhysAddr::new(self.0[&vaddr.into()].0.to_addr() + vaddr.addr() % PAGE_SIZE)
    }
}
