/// A PCI Configuration Access Mechanism.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cam {
    /// The PCI memory-mapped Configuration Access Mechanism.
    ///
    /// This provides access to 256 bytes of configuration space per device function.
    MmioCam,
    /// The PCIe memory-mapped Enhanced Configuration Access Mechanism.
    ///
    /// This provides access to 4 KiB of configuration space per device function.
    Ecam,
}

pub struct PCIIterator<'a> {
    root: &'a PCIRoot,
    bus: u8,
    slot: u8,
    function: u8,
}

const INVALID_FUNCTION: u32 = 0xffffffff;
const MAX_FUNCTIONS: u8 = 8;
const MAX_SLOTS: u8 = 32;

impl<'a> Iterator for PCIIterator<'a> {
    type Item = (*mut u8, u8, u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        while self.slot <= MAX_SLOTS {
            let addr = self.root.get_addr(self.bus, self.slot, self.function);

            self.function += 1;
            if self.function >= MAX_FUNCTIONS {
                self.function = 0;
                self.slot += 1;
            }

            unsafe {
                if (addr as *const u32).read_volatile() != INVALID_FUNCTION {
                    return Some((addr as _, self.bus, self.slot, self.function));
                }
            }
        }
        None
    }
}

pub struct PCIRoot {
    base: *mut u8,
    cam: Cam,
}

impl PCIRoot {
    pub const fn new(mmconfig_base: *mut u8, cam: Cam) -> Self {
        Self {
            base: mmconfig_base,
            cam,
        }
    }

    pub const fn enumerate_bus(&self, bus: u8) -> PCIIterator {
        PCIIterator {
            root: self,
            bus,
            slot: 0,
            function: 0,
        }
    }

    pub fn get_addr(&self, bus: u8, slot: u8, function: u8) -> *mut u8 {
        let mut dfaddr = 0;
        dfaddr += (bus as usize) << 8;
        dfaddr += (slot as usize) << 3;
        dfaddr += (function as usize) << 0;

        let mut addr = self.base as usize;
        addr += dfaddr
            << match self.cam {
                Cam::MmioCam => 8,
                Cam::Ecam => 12,
            };

        addr as _
    }

    #[inline]
    pub fn get_device(&self, bus: u8, slot: u8, function: u8) -> PCIDevice {
        PCIDevice {
            addr: self.get_addr(bus, slot, function) as _,
            bus,
            slot,
            function,
        }
    }
}

pub struct PCIDevice {
    addr: usize,
    bus: u8,
    slot: u8,
    function: u8,
}

impl PCIDevice {
    pub fn from_raw(mm_base: *mut u8, cam: Cam, bus: u8, slot: u8, function: u8) -> Self {
        let pci_root = PCIRoot::new(mm_base as _, cam);
        pci_root.get_device(bus, slot, function)
    }

    #[inline]
    pub fn get_reg(&self, offset: usize) -> u32 {
        unsafe {
            let reg_addr = self.addr + offset;
            (reg_addr as *const u32).read_volatile()
        }
    }

    #[inline]
    pub fn set_reg(&self, offset: usize, value: u32) {
        unsafe {
            let reg_addr = self.addr + offset;
            (reg_addr as *mut u32).write_volatile(value)
        }
    }
}
