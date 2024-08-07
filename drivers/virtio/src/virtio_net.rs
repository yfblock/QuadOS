use core::cmp;

use alloc::sync::Arc;
use alloc::vec::Vec;
use base::{DAlloc, DeviceType, Driver, NetDriver, NetError};
use lock_api::{Mutex, RawMutex};
use virtio_drivers::device::net::{self, TxBuffer};
use virtio_drivers::transport::Transport;

use super::virtio_impl::HalImpl;

#[allow(dead_code)]
pub struct VirtIONet<T: Transport, R, D: DAlloc> {
    inner: Mutex<R, net::VirtIONet<HalImpl<D>, T, 32>>,
    irqs: Vec<u32>,
}

unsafe impl<T: Transport, R, D: DAlloc> Sync for VirtIONet<T, R, D> {}
unsafe impl<T: Transport, R, D: DAlloc> Send for VirtIONet<T, R, D> {}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> Driver for VirtIONet<T, R, D> {
    fn get_id(&self) -> &str {
        "virtio-blk"
    }

    fn get_device(self: Arc<Self>) -> DeviceType {
        DeviceType::NET(self.clone())
    }
}

impl<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc> NetDriver for VirtIONet<T, R, D> {
    fn recv(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        let packet = self.inner.lock().receive().map_err(|_| NetError::NoData)?;
        let rlen = cmp::min(buf.len(), packet.packet_len());
        buf[..rlen].copy_from_slice(&packet.packet()[..rlen]);
        self.inner
            .lock()
            .recycle_rx_buffer(packet)
            .expect("can't receive data");
        Ok(rlen)
    }

    fn send(&self, buf: &[u8]) -> Result<(), NetError> {
        self.inner
            .lock()
            .send(TxBuffer::from(buf))
            .expect("can't send data");
        Ok(())
    }
}

pub fn init<T: Transport + 'static, R: RawMutex + 'static, D: DAlloc>(
    transport: T,
    irqs: Vec<u32>,
) -> Option<Arc<dyn Driver>> {
    info!("Initialize virtio-net device, irqs: {:?}", irqs);

    let net_device = Arc::new(VirtIONet::<T, R, D> {
        inner: Mutex::new(
            net::VirtIONet::<HalImpl<D>, T, 32>::new(transport, 2048)
                .expect("failed to create blk driver"),
        ),
        irqs,
    });
    Some(net_device)
}
