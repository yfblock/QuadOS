use alloc::sync::Arc;
use alloc::vec::Vec;
use drivers_base::{BlkDriver, DAlloc, DeviceType, Driver};
use lock_api::{Mutex, RawMutex};
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::Transport;

use super::virtio_impl::HalImpl;

pub struct VirtIOBlock<T: Transport, R, D: DAlloc> {
    inner: Mutex<R, VirtIOBlk<HalImpl<D>, T>>,
    irqs: Vec<u32>,
}

unsafe impl<T: Transport, R, D: DAlloc> Sync for VirtIOBlock<T, R, D> {}
unsafe impl<T: Transport, R, D: DAlloc> Send for VirtIOBlock<T, R, D> {}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> Driver for VirtIOBlock<T, R, D> {
    fn interrupts(&self) -> &[u32] {
        &self.irqs
    }

    fn get_id(&self) -> &str {
        "virtio-blk"
    }

    fn get_device(self: Arc<Self>) -> DeviceType {
        DeviceType::BLOCK(self.clone())
    }
}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> BlkDriver for VirtIOBlock<T, R, D> {
    fn read_blocks(&self, block_id: usize, buf: &mut [u8]) {
        self.inner
            .lock()
            .read_blocks(block_id, buf)
            .expect("can't read block by virtio block");
    }

    fn write_blocks(&self, block_id: usize, buf: &[u8]) {
        self.inner
            .lock()
            .write_blocks(block_id, buf)
            .expect("can't write block by virtio block");
    }

    fn capacity(&self) -> usize {
        self.inner.lock().capacity() as usize * 0x200
    }
}

pub fn init<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc>(
    transport: T,
    irqs: Vec<u32>,
) -> Option<Arc<dyn Driver>> {
    info!("Initialize virtio-block device");

    let blk_device = Arc::new(VirtIOBlock::<T, R, D> {
        inner: Mutex::new(
            VirtIOBlk::<HalImpl<D>, T>::new(transport).expect("failed to create blk driver"),
        ),
        irqs,
    });

    Some(blk_device)
}
