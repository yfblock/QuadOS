#![no_std]

pub fn reg_transfer<T>(addr: usize, offset: usize) -> &'static mut T {
    unsafe { ((addr + offset) as *mut T).as_mut().unwrap() }
}

/*
pub struct PresentState(u32) {
    reserved_25: u7,
    cmd_line_state: u1,
    dat_3_0_state: u4,
    card_wp_state: u1,
    card_cd_state: u1,
    card_stable: u1,
    card_inserted: u1,
    reserved_12: u4,
    buf_rd_enable: u1,
    buf_wr_enable: u1,
    rd_xfer_active: u1,
    wr_xfer_active: u1,
    reserved_4: u4,
    re_tune_req: u1,
    dat_line_active: u1,
    cmd_inhibit_dat: bool,
    cmd_inhibit: bool,
 } */

pub const SUPPORT_PCI_DEVICE: &[(u32, u32)] = &[
    (0x1b36, 0x0007)
];

pub struct SDCard {
    addr: usize,
}

impl SDCard {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    /// check the sdcard that was inserted
    pub fn check_sd(&self) -> bool {
        let present_state = reg_transfer::<usize>(self.addr, 0x24);
        // present_state.card_inserted().get() == u1!(1)
        log::debug!("sdcard: {:#x}", present_state);
        todo!("check sd")
        // Initialize sd card gpio
        // if check_sd() {
        //     pad_settings();
        //     reset_config();

        //     power_config(PowerLevel::V18);
        //     set_clock(4);

        //     // sdcard initialize.
        //     cmd_transfer(CommandType::CMD(0), 0, 0)?;
        //     cmd_transfer(CommandType::CMD(8), 0x1aa, 0)?;
        //     // wait for initialization to end.
        //     loop {
        //         cmd_transfer(CommandType::CMD(55), 0, 0)?;
        //         cmd_transfer(
        //             CommandType::ACMD(41),
        //             0x4000_0000 | 0x0030_0000 | (0x1FF << 15),
        //             0,
        //         )?;

        //         if *reg_transfer::<u32>(0x10) >> 31 == 1 {
        //             break;
        //         }
        //         for _ in 0..0x100_0000 {
        //             unsafe { asm!("nop") }
        //         }
        //     }
        //     log::debug!("init finished");
        //     // // get card and select
        //     cmd_transfer(CommandType::CMD(2), 0, 0)?;
        //     cmd_transfer(CommandType::CMD(3), 0, 0)?;
        //     log::debug!("start to read scd");
        //     let rsa = *reg_transfer::<u32>(0x10) & 0xffff0000;
        //     cmd_transfer(CommandType::CMD(9), rsa, 0)?; // get scd reg
        //     log::debug!("start to select card");
        //     cmd_transfer(CommandType::CMD(7), rsa, 0)?; // select card

        //     log::debug!("start to switch to 4 bit bus");
        //     // support 4 bit bus width.
        //     cmd_transfer(CommandType::CMD(55), rsa, 0)?;
        //     cmd_transfer(CommandType::ACMD(6), 2, 0)?;
        //     unsafe {
        //         *((SD_DRIVER_ADDR + 0x28) as *mut u8) |= 2;
        //     }
        //     clk_en(false);
        // }
    }
}
