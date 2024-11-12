#![no_std]

mod regs;

use core::{marker::PhantomData, ptr::slice_from_raw_parts_mut, slice};

use drivers_base::{
    pci::PCIDevice,
    DAlloc,
};
use regs::{CapRegister, EHCICmd, OpRegister, EHCIQH, EHCITD, HCCPARAMS, HCSPARAMS, USBCFR, USBCMD, USBPSC, USBSTS};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

pub const SUPPORT_PCI_DEVICE: &[(u32, u32)] = &[(0x1b36, 0x0007)];
const EHCI_HCI_VER: u16 = 0x100;

pub struct EHCIHost<D: DAlloc> {
    regs: &'static CapRegister,
    op_reg: &'static OpRegister,
    pci_dev: PCIDevice,
    phantom: PhantomData<D>,
}

impl<D: DAlloc> EHCIHost<D> {
    pub fn new(addr: usize, pci_dev: PCIDevice) -> Self {
        let cap_reg = unsafe {
            (addr as *const CapRegister)
                .as_ref()
                .expect("can't unwrap for EHCI registers")
        };
        let ehci = Self {
            regs: &cap_reg,
            op_reg: unsafe {
                ((addr + cap_reg.caplength.get() as usize) as *const OpRegister)
                    .as_ref()
                    .expect("Can't unwrap for OP registers")
            },
            pci_dev,
            phantom: PhantomData,
        };
        ehci.check();
        ehci.init();
        ehci.probe_ports();
        ehci
    }

    pub fn init(&self) {
        let eecp = self.regs.hcc_params.read(HCCPARAMS::EECP) as usize;
        let mut cap_reg = eecp;
        let mut cap_id = self.pci_dev.get_reg(cap_reg);

        loop {
            if cap_id & 0xff == 0x1 {
                if cap_id & 0x10000 != 0 {
                    cap_id &= !0x10000;
                    cap_id |= 0x1000000;
                    self.pci_dev.set_reg(cap_reg, cap_id);
                }
            }
            let pos_next = (cap_id >> 8) & 0xff;
            if pos_next != 0 {
                cap_reg += pos_next as usize;
            } else {
                break;
            }
        }
        // TODO: Set Interrupts for usbint.

        // Stop usb if already running.
        if self.op_reg.cmd.is_set(USBCMD::RUN) {
            self.op_reg.cmd.modify(USBCMD::RUN::CLEAR);
            while self.op_reg.cmd.is_set(USBCMD::RUN) {
                // TODO: Sleep for a while
            }
        }

        // Reset the host controller.
        self.op_reg.cmd.modify(USBCMD::HCRST::SET);
        while self.op_reg.cmd.is_set(USBCMD::HCRST) {
            // TODO: Sleep for a while
        }
        // Display Information.
        log::debug!("[EHCI] USBCMD: {:#x}", self.op_reg.cmd.get());
        log::debug!("[EHCI] USBSTS: {:#x}", self.op_reg.sts.get());
        log::debug!("[EHCI] USBINTR: {:#x}", self.op_reg.intr.get());
        log::debug!("[EHCI] USBFRIDX: {:#x}", self.op_reg.fridx.get());
        log::debug!("[EHCI] USBCTRLDSS: {:#x}", self.op_reg.ctr.get());
        log::debug!("[EHCI] USBCFR: {:#x}", self.op_reg.cfr.get());

        let periodic_paddr = D::alloc(1);
        let periodic_list = unsafe {
            slice::from_raw_parts_mut(D::phys_to_virt(periodic_paddr) as *mut u32, 1024)
        };
        for i in 0..1024 {
            periodic_list[i] |= 1;
        }

        self.op_reg.ctr.set(0);
        // Enable ALL USB Interrupts.
        self.op_reg.intr.set(0);

        self.op_reg.pfladdr.set(periodic_paddr as _);

        // self.op_reg.cmd.write(
        //     USBCMD::RUN::SET + USBCMD::ITC.val(0x48)
        // );
        self.op_reg.cmd.modify(USBCMD::ITC.val(0x40));

        self.op_reg.cmd.modify(USBCMD::RUN::SET);

        self.op_reg.cfr.modify(USBCFR::CF::SET);
    }

    pub fn check(&self) {
        assert_eq!(
            self.regs.hci_version.get(),
            EHCI_HCI_VER,
            "[EHCI] This driver is not compliant with current EHCIprotocol version"
        );
        log::debug!("64-bit Version {:#x}", self.regs.hcc_params.get());

    }

    pub fn init_port(&self, port: usize) {
        log::debug!("[EHCI] Init port [{}]", port);
        // reset port
        self.op_reg.psc[port].modify(USBPSC::RESET::SET);

        self.op_reg.psc[port].modify(USBPSC::RESET::CLEAR);

        if !self.op_reg.psc[port as usize].is_set(USBPSC::PORT_EN) {
            return;
        }

        // if self.op_reg.psc[port as usize].get() & 0x3 != 0x3 {
        //     return;
        // }

        // FIXME: Remove Magic Number 1
        self.ehci_set_device_address(2);
    }

    pub fn ehci_set_device_address(&self, addr: u8) {
        let paddr = D::alloc(1);
        let paddr1 = D::alloc(1);
        
        unsafe {
            let cmd = (D::phys_to_virt(paddr) as *mut EHCICmd).as_mut().unwrap();
            cmd.b_request_t = 0 << 5;
            cmd.b_request = 0x5;        // 0x5 is set addr
            cmd.w_index = 0;
            cmd.w_value = addr as _;
            cmd.w_length = 0;

            let command = (D::phys_to_virt(paddr1) as *mut EHCITD).as_mut().unwrap();
            let status = (D::phys_to_virt(paddr1 + 0x400) as *mut EHCITD).as_mut().unwrap();
            let head1 = (D::phys_to_virt(paddr1 + 0x800) as *mut EHCIQH).as_mut().unwrap();
            let head2 = (D::phys_to_virt(paddr1 + 0xc00) as *mut EHCIQH).as_mut().unwrap();
            
            command.nxt_link = D::virt_to_phys(status as *mut _ as _) as u32;
            command.alt_link = 1;
            command.token |= 8 << 16;   // setup size
            command.token |= 1 << 7;    // actief
            command.token |= 0x2 << 8;  // type is setup
            command.token |= 0x3 << 10; // maxerror
            command.buffer[0] = D::virt_to_phys(cmd as *mut _ as _) as u32;

            status.nxt_link = 1;
            status.alt_link = 1;
            status.token |= 1 << 8;     // PID instellen
            status.token |= 1 << 31;    // toggle
            status.token |= 1 << 7;     // actief
            status.token |= 0x3 << 10;  // maxerror

            head2.alt_link = 1;
            head2.nxt_link = D::virt_to_phys(command as *mut _ as _ )as u32; // qdts2
            head2.hlp = (D::virt_to_phys(head1 as *mut _ as _) as u32) | 2;
            head2.cur_link = 0;         // qdts1
            head2.ep_char |= 1 << 14;   // dtc
            head2.ep_char |= 64 << 16;  // mplen
            head2.ep_char |= 2 << 12;   // eps
            head2.ep_cap = 0x40000000;

            head1.alt_link = 1;
            head1.nxt_link = 1;
            head1.hlp = (D::virt_to_phys(head2 as *mut _ as _)as u32) | 2;
            head1.cur_link = 0;
            head1.ep_char = 1 << 15; // T
            head1.token = 0x40;
            log::info!("set request list");

            self.op_reg.calar.set(D::virt_to_phys(head1 as *mut _ as _) as u32);
            self.op_reg.cmd.modify(USBCMD::ASPM_EN::SET);
            // unsigned char lstatus = ehci_wait_for_completion(status);
            
            let mut i = 0;
            let ptr = &status.token as *const u32;
            log::info!("ready to read status");
            loop {
                i+=1;
                if ptr.read_volatile() & (1 << 7) == 0 {
                    break;
                }
                if i > 0x100000 {
                    log::debug!("status: {:#x} {:#b}", ptr.read_volatile(), ptr.read_volatile());
                    log::debug!("status: {:#x}", self.op_reg.sts.get());
                    // todo!("check result");
                }
            }

            self.op_reg.cmd.modify(USBCMD::ASPM_EN::CLEAR);
            self.op_reg.calar.set(1);
        }

    }

    pub fn probe_ports(&self) {
        let ports = self.regs.hcs_params.read(HCSPARAMS::N_PORTS);
        for i in 0..ports {
            self.init_port(i as _);
        }
        todo!("Return the EHCI host")
    }

    pub fn wait_for_complete(&self) -> Result<(), ()> {
        // loop {
        //     if self.op_reg.sts.is_set(USBSTS::HS_ERR) {
        //         return Err(());
        //     }

        // }

        // void *buffer = requestPage();
        // EhciCMD *command = ehci_generate_command_structure(USB2_REQUEST_GET_DESCRIPTOR,0,4,0,0,size,(type << 8) | index);                                                          // OK
        // EhciTD *status = ehci_generate_transfer_descriptor(1,0,0,1,0);                                                                      // OK
        // EhciTD *transfercommand = ehci_generate_transfer_descriptor((uint32_t)(upointer_t)status,1,size,1,(uint32_t)(upointer_t)buffer);    // OK
        // EhciTD *td = ehci_generate_transfer_descriptor((uint32_t)(upointer_t)transfercommand,2,8,0,(uint32_t)(upointer_t)command);       // OK 8
        // EhciQH *head1 = ehci_generate_queue_head(1,0,0,1,0,0,0,0x40,0);
        // EhciQH *head2 = ehci_generate_queue_head((uint32_t)(upointer_t)td,2,1,0,64,address,0x40000000,0,0);
        // head1->horizontal_link_pointer = ((uint32_t)(upointer_t)head2) | 2;
        // head2->horizontal_link_pointer = ((uint32_t)(upointer_t)head1) | 2;
        // uint8_t res = ehci_offer_queuehead_to_ring((uint32_t)(upointer_t)head1,status);
        todo!("Wait And Check USB Status")
    }

    pub fn build_queue(&self) {

    }
}
