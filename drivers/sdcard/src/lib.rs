#![no_std]

use core::{
    cmp,
    marker::PhantomData,
    mem::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};

use drivers_base::DAlloc;
use regs::{
    BlkCnt, Capability, Capability2,
    ClkCtl::{self, TOUT_CNT},
    CommandType,
    ErrInt::{self, XFER_CMPL},
    PresentStatus, Register, XferCmd, ADMA2_DT, PWRLVL,
};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    registers::InMemoryRegister,
};

mod regs;

pub const SUPPORT_PCI_DEVICE: &[(u32, u32)] = &[(0x1b36, 0x0007)];

pub struct SDCard<D: DAlloc> {
    regs: &'static Register,
    rsa: u32,
    dma: bool,
    adma2_dt: (AtomicUsize, AtomicUsize),
    phantom: PhantomData<D>,
}

impl<D: DAlloc> SDCard<D> {
    pub fn new(addr: usize, dma: bool) -> Self {
        let mut sd = Self {
            regs: unsafe {
                (addr as *const Register)
                    .as_ref()
                    .expect("Can't create SDCard")
            },
            dma,
            rsa: 0,
            adma2_dt: Default::default(),
            phantom: PhantomData::default(),
        };
        sd.init_sd();
        if dma {
            let paddr = D::alloc(1);
            let vaddr = D::phys_to_virt(paddr);
            sd.adma2_dt.0.store(paddr, Ordering::SeqCst);
            sd.adma2_dt.1.store(vaddr, Ordering::SeqCst);
        }
        if sd.check_sd() {
            // TODO: ReSet Configuration
            sd.reset_config();
            sd.set_clk(4);

            // Setting Power
            // sd.regs
            //     .pwr_bg
            //     .write(PWRLVL::PWR_EN::SET + PWRLVL::VOL_SEL::V33);

            sd.test_transfer();
        }
        sd
    }

    /// Get the base address of the sdcard register block.
    pub fn addr(&self) -> usize {
        self.regs as *const _ as usize
    }

    /// Set the clock for the sdcard.
    pub fn set_clk(&self, clk: u32) {
        self.regs
            .clk_ctl
            .modify(ClkCtl::SD_CLK_EN::CLEAR + ClkCtl::FREQ_SEL.val(clk) + ClkCtl::INT_CLK_EN::SET);

        loop {
            if self.regs.clk_ctl.is_set(ClkCtl::INT_CLK_STABLE) {
                break;
            }
        }
        self.regs.clk_ctl.modify(ClkCtl::SD_CLK_EN::SET);
    }

    /// Reset SDIO Configuration
    #[inline]
    pub fn reset_config(&self) {
        log::info!("cmd status: {:#x}", self.regs.err_int.get());

        // Cleat All Bits
        self.regs.pwr_bg.write(PWRLVL::VOL_SEL::CLEAR);
        self.regs
            .clk_ctl
            .modify_no_read(self.regs.clk_ctl.extract(), ClkCtl::SOFT_RST_ALL::CLEAR);

        self.regs
            .pwr_bg
            .write(PWRLVL::PWR_EN::SET + PWRLVL::VOL_SEL::V18);

        match self.dma {
            true => self.regs.pwr_bg.modify(PWRLVL::DMA_SEL::ADMA2),
            false => self.regs.pwr_bg.modify(PWRLVL::HS_EN::SET),
        }
    }

    /// check the sdcard that was inserted
    #[inline]
    pub fn check_sd(&self) -> bool {
        log::trace!("SD Status: {:#x}", self.regs.status.get());
        self.regs.status.is_set(PresentStatus::PRESENT)
    }

    pub fn init_sd(&mut self) {
        log::debug!(
            "V18    Support: {:#x?}",
            self.regs.cap1.is_set(Capability::V18_SUPPORT)
        );
        log::debug!(
            "V30    Support: {:#x?}",
            self.regs.cap1.is_set(Capability::V30_SUPPORT)
        );
        log::debug!(
            "V33    Support: {:#x?}",
            self.regs.cap1.is_set(Capability::V33_SUPPORT)
        );

        log::debug!(
            "ADMA2  Support: {:#x?}",
            self.regs.cap1.is_set(Capability::ADMA2_SUPPORT)
        );
        log::debug!(
            "SDMA   Support: {:#x?}",
            self.regs.cap1.is_set(Capability::SDMA_SUPPORT)
        );
        log::debug!(
            "HS     Support: {:#x?}",
            self.regs.cap1.is_set(Capability::HS_SUPPORT)
        );

        log::debug!(
            "SDR50  Support: {:#x?}",
            self.regs.cap2.is_set(Capability2::SDR50_SUPPORT)
        );
        log::debug!(
            "SDR104 Support: {:#x?}",
            self.regs.cap2.is_set(Capability2::SDR104_SUPPORT)
        );
        log::debug!(
            "DDR50  Support: {:#x?}",
            self.regs.cap2.is_set(Capability2::DDR50_SUPPORT)
        );
        log::debug!(
            "CLK_MULTIPLIER: {:#x?}",
            self.regs.cap2.read(Capability2::CLK_MULTIPLIER)
        );

        self.cmd_transfer(CommandType::CMD(0), 0, 0);
        self.cmd_transfer(CommandType::CMD(8), 0x1aa, 0);

        // wait for initialization to end.
        const XSDPS_ACMD41_HCS: u32 = 0x4000_0000;
        const XSDPS_ACMD41_3V3: u32 = 0x0030_0000;
        loop {
            self.cmd_transfer(CommandType::CMD(55), 0, 0);
            self.cmd_transfer(
                CommandType::ACMD(41),
                XSDPS_ACMD41_HCS | XSDPS_ACMD41_3V3,
                0,
            );

            if self.regs.resp[0].get() >> 31 == 1 {
                break;
            }
        }
        // Verify the sdcard inserted, and get CID information.
        self.cmd_transfer(CommandType::CMD(2), 0, 0);
        log::info!(
            "OEM: {}{} DEVICE: {}{}{}{}{} MDT: {}/{}",
            self.regs.resp[3].get().to_le_bytes()[2] as char,
            self.regs.resp[3].get().to_le_bytes()[1] as char,
            self.regs.resp[3].get().to_le_bytes()[0] as char,
            self.regs.resp[2].get().to_le_bytes()[3] as char,
            self.regs.resp[2].get().to_le_bytes()[2] as char,
            self.regs.resp[2].get().to_le_bytes()[1] as char,
            self.regs.resp[2].get().to_le_bytes()[0] as char,
            (self.regs.resp[0].get() >> 12) & 0xff,
            (self.regs.resp[0].get() >> 8) & 0xf,
        );
        // Broadcast rsa of the sdcard.
        self.cmd_transfer(CommandType::CMD(3), 0, 0);
        self.rsa = self.regs.resp[0].get();
    }

    pub fn test_transfer(&self) {
        // Read CSD Register
        let res = self.cmd_transfer(CommandType::CMD(9), self.rsa, 0);
        log::info!("version: {}", self.regs.resp[3].get() >> 30);
        log::info!("version: {}", (self.regs.resp[1].get() >> 24) & 0x3);
        let resp = ((res[3] as u128) << 96)
            | ((res[2] as u128) << 64)
            | ((res[1] as u128) << 32)
            | res[0] as u128;
        let c_size = (resp >> 62) & 0xfff;
        let size_multi = (resp >> 47) & 0x7;
        let sector_size = (resp >> 80) & 0xf;
        log::info!(
            "c_size: {c_size:#x}  multi: {size_multi}  sec_size: {sector_size}  nr: {:#x}",
            (c_size + 1) * (1 << (2 + size_multi))
        );
        log::info!("sectore size: {:#x}", (1 << sector_size));
        log::info!(
            "sdcard size: {} MB",
            (c_size + 1) << (size_multi + 2 + sector_size - 10 - 10)
        );
        // Select SD Card, if 0 cancel all selected
        self.cmd_transfer(CommandType::CMD(7), self.rsa, 0);

        // self.cmd_transfer(CommandType::CMD(55), rsa, 0);
        // // Read SCR Register
        // self.cmd_transfer(CommandType::ACMD(51), 0, 0);

        // self.cmd_transfer(CommandType::CMD(55), rsa, 0);
        // self.cmd_transfer(CommandType::ACMD(6), 0, 0);

        // Test Read
        let mut buffer = [0u8; 0x1000];
        self.read_block(0, &mut buffer);
        for i in 0..10 {
            log::info!("data: {:#x}", buffer[i]);
        }
        for i in 0..10 {
            log::info!("data: {:#x}", buffer[0x200 + i]);
        }

        log::info!(
            "pwr: {:#x?}",
            self.regs
                .pwr_bg
                .read_as_enum::<PWRLVL::VOL_SEL::Value>(PWRLVL::VOL_SEL)
        );
        loop {}
    }

    /// Read Block From SD Card, IO Mode Transfer, Not DMA.
    ///
    /// Block size is 0x200, 512 Bytes. Make sure that size of buffer aligned with 0x200.
    /// Read size relying on the buffer.
    pub fn read_block(&self, blk_off: u32, buffer: &mut [u8]) {
        assert!(buffer.len() % 0x200 == 0);
        let blk_cnt = buffer.len() / 0x200;

        // If Read Mode is DMA.
        // TODO: Check ADMA Support and using SDMA When ADMA is not available
        if self.dma {
            let adma_dt = self.adma2_dt();
            let mut idx = 0;
            let mut buffer_vaddr = buffer.as_ptr() as usize;
            let mut last = buffer.len();
            loop {
                adma_dt[idx].write(
                    ADMA2_DT::VALID::SET
                        + ADMA2_DT::INT::SET
                        + ADMA2_DT::ACT::Tran
                        + ADMA2_DT::ADDR.val(D::virt_to_phys(buffer_vaddr) as _),
                );
                let len = match buffer_vaddr % 0x100 != 0 {
                    true => cmp::min(0x1000 - (buffer_vaddr % 0x100), last),
                    false => cmp::min(0x1000, last),
                };
                adma_dt[idx].modify(ADMA2_DT::LEN.val(len as _));
                buffer_vaddr += len;
                last -= len;
                idx += 1;
                if last == 0 {
                    break;
                }
            }
            adma_dt[buffer.len() / 0x1000].modify(ADMA2_DT::END::SET);
            self.regs
                .adma_addr
                .set(self.adma2_dt.0.load(Ordering::SeqCst) as _);
        }

        // Send Write CMD, IO Mode, Not DMA Mode
        self.cmd_transfer(CommandType::CMD(18), blk_off, blk_cnt as u32);

        // Read Transfer Data.
        for idx in 0..blk_cnt {
            assert!(!self.regs.err_int.is_set(ErrInt::ERR_INT));
            while !self
                .regs
                .err_int
                .any_matching_bits_set(ErrInt::BUF_RR::SET + ErrInt::XFER_CMPL::SET)
            {}

            self.regs
                .err_int
                .write(ErrInt::BUF_RR::SET + ErrInt::XFER_CMPL::SET);

            if self.dma {
                return;
            }

            for off in 0..(buffer.len() / core::mem::size_of::<u32>()) {
                unsafe {
                    transmute::<*mut u8, *mut u32>(buffer.as_mut_ptr())
                        .add(idx * 0x200 + off)
                        .write_volatile(self.regs.bf_data.get());
                }
            }

            if self.regs.err_int.is_set(XFER_CMPL) {
                self.regs.err_int.write(XFER_CMPL::SET);
                break;
            }
        }
    }

    /// Transfer a command.
    /// 
    /// If you have additional operations, you should complete before the transfer
    /// cmd is what command to transfer
    /// args is the argument of the transfer
    /// blk_cnt is the transfer block_count. (Read, Write Block only)
    pub fn cmd_transfer(&self, cmd: CommandType, arg: u32, blk_cnt: u32) -> [u32; 4] {
        log::info!("send cmd: {:?}", cmd);
        while self.regs.status.is_set(PresentStatus::INHIBIT_DAT) {}

        let mut flags = XferCmd::CMD_IDX.val(cmd.num() as u32);

        if blk_cnt > 0 {
            // set blk size and blk count
            self.regs
                .cnt
                .modify(BlkCnt::BLK_CNT.val(blk_cnt) + BlkCnt::XFER_BLK_SIZE.val(0x200));
            flags += XferCmd::BLK_CNT_EN::SET;
        }
        if blk_cnt > 1 {
            flags += XferCmd::MULTI_BLK_EN::SET;
        }

        log::info!("flags: {:#x}", flags.value);

        flags += match cmd {
            CommandType::CMD(17) | CommandType::CMD(18) | CommandType::ACMD(51) => {
                XferCmd::DATA_PRESENT::SET + XferCmd::DAT_XFER_READ::SET
            }
            CommandType::CMD(24) => XferCmd::DATA_PRESENT::SET,
            _ => XferCmd::DMA_EN::CLEAR,
        };

        flags += match cmd {
            // R1
            CommandType::ACMD(6)
            | CommandType::ACMD(42)
            | CommandType::ACMD(51)
            | CommandType::CMD(17)
            | CommandType::CMD(18)
            | CommandType::CMD(24)
            | CommandType::CMD(8)
            | CommandType::CMD(16)
            | CommandType::CMD(7) => XferCmd::RESP_TYPE_SEL::L48 + XferCmd::CMD_CRC_CHK_EN::SET,
            CommandType::CMD(2) | CommandType::CMD(9) => {
                XferCmd::RESP_TYPE_SEL::L136 + XferCmd::CMD_CRC_CHK_EN::SET
            }
            // R3
            CommandType::ACMD(41) | CommandType::CMD(58) => XferCmd::RESP_TYPE_SEL::L48,
            // R6
            CommandType::CMD(3) => {
                XferCmd::RESP_TYPE_SEL::L48
                    + XferCmd::CMD_CRC_CHK_EN::SET
                    + XferCmd::CMD_IDX_CHK_EN::SET
            }
            _ => XferCmd::DMA_EN::CLEAR,
        };

        if self.dma {
            flags += XferCmd::DMA_EN::SET;
        }

        // set blk cnt
        // self.regs.cnt.modify(BLK_CNT.val(0));
        // set timeout time
        self.regs.clk_ctl.modify(TOUT_CNT.val(0x3));
        // Clear Err Int Register
        self.regs.err_int.set(0xF3FFFFFF);

        self.regs.arg1.set(arg);
        self.regs.cmd.write(flags);

        // Wait for command complete
        self.wait_for_cmd();

        let res = [
            self.regs.resp[0].get(),
            self.regs.resp[1].get(),
            self.regs.resp[2].get(),
            self.regs.resp[3].get(),
        ];

        // this is used to print result and consume ptr.
        // There needs to read the resp regs after cmd.
        log::trace!(
            "resp: {:#x} {:#x} {:#x} {:#x}",
            res[0],
            res[1],
            res[2],
            res[3]
        );

        res
    }

    pub fn adma2_dt(&self) -> &'static mut [InMemoryRegister<u64, ADMA2_DT::Register>] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.adma2_dt.1.load(Ordering::SeqCst)
                    as *mut InMemoryRegister<u64, ADMA2_DT::Register>,
                0x200,
            )
        }
    }

    pub fn wait_for_cmd(&self) {
        loop {
            assert!(!self.regs.err_int.is_set(ErrInt::ERR_INT));
            if self.regs.err_int.is_set(ErrInt::CMD_CMPL) {
                self.regs.err_int.write(ErrInt::CMD_CMPL::SET);
                break;
            }
        }
    }
}
