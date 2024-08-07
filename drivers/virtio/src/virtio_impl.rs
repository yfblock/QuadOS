use base::DAlloc;
use core::{marker::PhantomData, ptr::NonNull};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct HalImpl<T: DAlloc>(PhantomData<T>);

unsafe impl<D: DAlloc> Hal for HalImpl<D> {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let paddr = D::alloc(pages);
        let vaddr = D::phys_to_virt(paddr);
        (paddr, NonNull::new(vaddr as _).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        D::dealloc(paddr, pages)
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(D::phys_to_virt(paddr) as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        D::virt_to_phys(buffer.as_ptr() as *const u8 as _)
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}
