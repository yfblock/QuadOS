#![no_std]
#![feature(used_with_arg)]
#![feature(strict_provenance)]

extern crate alloc;
#[macro_use]
extern crate log;

pub mod virtio_blk;
pub mod virtio_impl;
pub mod virtio_input;
pub mod virtio_net;

use core::ptr::NonNull;

use alloc::{sync::Arc, vec::Vec};
use base::{DAlloc, Driver};
use lock_api::RawMutex;
use virtio_drivers::transport::{
    mmio::{MmioTransport, VirtIOHeader},
    DeviceType, Transport,
};

pub fn probe<R: RawMutex + 'static, D: DAlloc>(
    addr: usize,
    irqs: Vec<u32>,
) -> Option<Arc<dyn Driver>> {
    let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
    if let Ok(transport) = unsafe { MmioTransport::new(header) } {
        info!(
            "Detected virtio MMIO device with
                vendor id {:#X}
                device type {:?}
                version {:?} 
                addr @ {:#X} 
                interrupt: {:?}",
            transport.vendor_id(),
            transport.device_type(),
            transport.version(),
            addr,
            irqs
        );
        match transport.device_type() {
            DeviceType::Block => virtio_blk::init::<MmioTransport, R, D>(transport, irqs),
            DeviceType::Input => virtio_input::init::<MmioTransport, R, D>(transport, irqs),
            DeviceType::Network => virtio_net::init::<MmioTransport, R, D>(transport, irqs),
            device_type => {
                warn!("Unrecognized virtio device: {:?}", device_type);
                None
            }
        }
    } else {
        None
    }
}
