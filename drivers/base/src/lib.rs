#![no_std]

extern crate alloc;

use core::fmt::Debug;

use alloc::sync::Arc;

/// Device Type Enumerator
pub enum DeviceType {
    RTC(Arc<dyn RtcDriver>),
    BLOCK(Arc<dyn BlkDriver>),
    NET(Arc<dyn NetDriver>),
    INPUT(Arc<dyn InputDriver>),
    INT(Arc<dyn IntDriver>),
    UART(Arc<dyn UartDriver>),
    None,
}

/// Driver Trait
pub trait Driver: Send + Sync {
    /// Get driver name
    fn get_id(&self) -> &str;

    /// Get interrupt numbers associated with the driver
    fn interrupts(&self) -> &[u32] {
        &[]
    }

    /// Trying to handle an interrupt
    fn try_handle_interrupt(&self, _irq: u32) -> bool {
        false
    }

    /// Get device type and self pointer.
    fn get_device(self: Arc<Self>) -> DeviceType;
}

impl Debug for dyn Driver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "device: {} irq: {:?}", self.get_id(), self.interrupts())
    }
}

/// Real time controller driver.
pub trait RtcDriver: Driver {
    fn read_timestamp(&self) -> u64;
    fn read(&self) -> u64;
}

/// Block device driver trait.
pub trait BlkDriver: Driver {
    fn read_blocks(&self, block_id: usize, buf: &mut [u8]);
    fn write_blocks(&self, block_id: usize, buf: &[u8]);
    fn capacity(&self) -> usize {
        0
    }
}

#[derive(Debug)]
pub enum NetError {
    NoData,
}

/// Net Interface Card
pub trait NetDriver: Driver {
    fn recv(&self, buf: &mut [u8]) -> Result<usize, NetError>;
    fn send(&self, buf: &[u8]) -> Result<(), NetError>;
}

/// Interrupt controller driver trait.
pub trait IntDriver: Driver {
    fn register_irq(&self, irq: u32, driver: Arc<dyn Driver>);
}

/// Input driver Trait.
pub trait InputDriver: Driver {
    fn read_event(&self) -> u64;
    fn handle_irq(&self);
    fn is_empty(&self) -> bool;
}

/// Uart driver trait.
pub trait UartDriver: Driver {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
}

/// Default implementation for the driver trait.
pub struct UnsupportedDriver;

/// Implement the default driver implementation
impl Driver for UnsupportedDriver {
    fn get_id(&self) -> &str {
        "unsupported-driver"
    }

    fn get_device(self: Arc<Self>) -> DeviceType {
        DeviceType::None
    }
}

/// Driver page alloc interface.
pub trait DAlloc: 'static {
    /// This function allocate the memory, and the return value is physical memory.
    ///
    fn alloc(pages: usize) -> usize;

    /// Deallocate the memory
    fn dealloc(paddr: usize, pages: usize) -> i32;

    /// Convert physical address to virtual address.
    fn phys_to_virt(paddr: usize) -> usize;

    /// Convert virtual address to physical address.
    fn virt_to_phys(vaddr: usize) -> usize;
}
