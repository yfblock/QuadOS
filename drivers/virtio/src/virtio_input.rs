use alloc::sync::Arc;
use alloc::vec::Vec;
use drivers_base::{DAlloc, DeviceType, Driver, InputDriver};
use lock_api::{Mutex, RawMutex};
use virtio_drivers::device::input::VirtIOInput as VirtIOInputWrapper;
use virtio_drivers::transport::Transport;

use super::virtio_impl::HalImpl;

pub struct VirtIOInput<T: Transport, R: RawMutex, D: DAlloc> {
    _inner: Mutex<R, VirtIOInputWrapper<HalImpl<D>, T>>,
    interrupts: Vec<u32>,
}

unsafe impl<T: Transport, R: RawMutex, D: DAlloc> Sync for VirtIOInput<T, R, D> {}
unsafe impl<T: Transport, R: RawMutex, D: DAlloc> Send for VirtIOInput<T, R, D> {}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> Driver for VirtIOInput<T, R, D> {
    fn get_id(&self) -> &str {
        "virtio-input"
    }

    fn interrupts(&self) -> &[u32] {
        &self.interrupts
    }

    fn get_device(self: Arc<Self>) -> DeviceType {
        DeviceType::INPUT(self.clone())
    }
}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> InputDriver
    for VirtIOInput<T, R, D>
{
    fn read_event(&self) -> u64 {
        todo!()
    }

    fn handle_irq(&self) {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }
}

pub fn init<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc>(
    transport: T,
    irqs: Vec<u32>,
) -> Option<Arc<dyn Driver>> {
    info!("Initialize virtio-iput device");

    let input_device = Arc::new(VirtIOInput::<T, R, D> {
        _inner: Mutex::new(
            VirtIOInputWrapper::<HalImpl<D>, T>::new(transport)
                .expect("failed to create blk driver"),
        ),
        interrupts: irqs,
    });
    Some(input_device)
}
