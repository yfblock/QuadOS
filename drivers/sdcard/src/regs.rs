use tock_registers::{
    fields::FieldValue, register_bitfields, register_structs, registers::{ReadOnly, ReadWrite}
};

register_structs! {
    pub(crate) Register {
        (0x00 => pub addr: ReadWrite<u32>),
        (0x04 => pub cnt: ReadWrite<u32, BlkCnt::Register>),
        (0x08 => pub arg1: ReadWrite<u32>),
        (0x0c => pub cmd: ReadWrite<u32, XferCmd::Register>),
        (0x10 => pub resp: [ReadWrite<u32>; 4]),
        (0x20 => pub bf_data: ReadWrite<u32>),
        (0x24 => pub status: ReadOnly<u32, PresentStatus::Register>),
        (0x28 => pub pwr_bg: ReadWrite<u32, PWRLVL::Register>),
        (0x2c => pub clk_ctl: ReadWrite<u32, ClkCtl::Register>),
        (0x30 => pub err_int: ReadWrite<u32, ErrInt::Register>),
        (0x34 => _reserved),
        (0x40 => pub cap1: ReadOnly<u32, Capability::Register>),
        (0x44 => pub cap2: ReadOnly<u32, Capability2::Register>),
        (0x48 => _reserved2),
        (0x58 => pub adma_addr: ReadWrite<u64>),
        (0x60 => @END),
    }
}

register_bitfields! [
    u32,
    pub BlkCnt [
        XFER_BLK_SIZE OFFSET(0) NUMBITS(12),
        BLK_CNT OFFSET(16) NUMBITS(16),
    ],
    pub XferCmd [
        DMA_EN OFFSET(0) NUMBITS(1) [],
        BLK_CNT_EN OFFSET(1) NUMBITS(1) [],
        AUTO_CMD_EN OFFSET(2) NUMBITS(2) [
            DISABLE = 0,
            CMD12 = 1,
            CMD23 = 2
        ],
        DAT_XFER_READ OFFSET(4) NUMBITS(1) [],
        MULTI_BLK_EN OFFSET(5) NUMBITS(1) [],
        RESP_TYPE OFFSET(6) NUMBITS(1) [
            // (Memory)
            R1 = 0,
            // (SDIO)
            R5 = 1
        ],
        RESP_ERR_CHK_EN OFFSET(7) NUMBITS(1) [],
        RESP_INT_DISABLE OFFSET(8) NUMBITS(1) [],
        // Response type
        RESP_TYPE_SEL OFFSET(16) NUMBITS(2) [
            NO_RESP = 0,
            // Response Length 136
            L136 = 1,
            // Response Length 48
            L48 = 2,
            // Response Length 48 with busy
            L48_BUSY = 3
        ],
        SUB_CMD_FLAG OFFSET(18) NUMBITS(1) [
            MAIN_CMD = 0,
            SUB_CMD = 1
        ],
        CMD_CRC_CHK_EN OFFSET(19) NUMBITS(1) [],
        CMD_IDX_CHK_EN OFFSET(20) NUMBITS(1) [],
        DATA_PRESENT OFFSET(21) NUMBITS(1) [],
        CMD_TYPE OFFSET(22) NUMBITS(2) [
            NORMAL = 0,
            // CMD52 for writing "Bus Suspend" in CCCR
            SUSPEND = 1,
            // CMD52 for writing "Function Select" in CCCR
            SELECT = 2,
            // CMD12, CMD52 for writing "I/O Abort" in CCCR
            ABORT = 3,
        ],
        CMD_IDX OFFSET(24) NUMBITS(6) []
    ],
    pub PresentStatus [
        INHIBIT OFFSET(0) NUMBITS(1) [],
        INHIBIT_DAT OFFSET(1) NUMBITS(1) [],

        BUF_WR_ENABLE OFFSET(10) NUMBITS(1) [],
        BUF_RD_ENABLE OFFSET(11) NUMBITS(1) [],
        PRESENT OFFSET(16) NUMBITS(1) [],
        STABLE  OFFSET(17) NUMBITS(1) [],

        DAT_SIG OFFSET(20) NUMBITS(4) [],
        CMD_SIG OFFSET(24) NUMBITS(1) [],
    ],
    pub PWRLVL [
        HS_EN OFFSET(2) NUMBITS(1) [],
        DMA_SEL OFFSET(3) NUMBITS(2) [
            SDMA = 0,
            ADMA2 = 2,
            ADMA2_3 = 3,
        ],
        PWR_EN OFFSET(8) NUMBITS(1) [],
        VOL_SEL OFFSET(9) NUMBITS(3) [
            V33 = 0b111,
            V30 = 0b110,
            V18 = 0b101
        ],
    ],
    pub ClkCtl [
        INT_CLK_EN OFFSET(0) NUMBITS(1) [],
        INT_CLK_STABLE OFFSET(1) NUMBITS(1) [],
        SD_CLK_EN OFFSET(2) NUMBITS(1) [],
        PLL_EN OFFSET(3) NUMBITS(1) [],

        UP_FREQ_SEL OFFSET(6) NUMBITS(2) [],
        FREQ_SEL OFFSET(8) NUMBITS(8) [],

        TOUT_CNT OFFSET(16) NUMBITS(4) [
            // TODO: CNT Select
        ],
        SOFT_RST_ALL OFFSET(24) NUMBITS(1) [],
        SOFT_RST_CMD OFFSET(25) NUMBITS(1) [],
        SOFT_RST_DAT OFFSET(26) NUMBITS(1) [],
    ],
    pub ErrInt [
        CMD_CMPL 0,
        XFER_CMPL 1,
        BUF_WR 4,
        BUF_RR 5,
        ERR_INT 15,
    ],
    pub Capability [
        TOUT_CLK_FREQ OFFSET(0) NUMBITS(1) [],
        MAX_BLK_LEN OFFSET(16) NUMBITS(2) [
            B512 = 0,
            B1024 = 1,
            B2048 = 2
        ],
        EMBEDDED_8BIT OFFSET(18) NUMBITS(1) [],
        ADMA2_SUPPORT OFFSET(19) NUMBITS(1) [],
        HS_SUPPORT OFFSET(21) NUMBITS(1) [],
        SDMA_SUPPORT OFFSET(22) NUMBITS(1) [],
        SUSP_RES_SUPPORT OFFSET(23) NUMBITS(1) [],
        V33_SUPPORT OFFSET(24) NUMBITS(1) [],
        V30_SUPPORT OFFSET(25) NUMBITS(1) [],
        V18_SUPPORT OFFSET(26) NUMBITS(1) [],
        BUTS64_SUPPORT OFFSET(28) NUMBITS(1) [],
        ASYNC_INT_SUPPORT OFFSET(29) NUMBITS(1) [],
        SLOT_TYPE OFFSET(30) NUMBITS(2) [
            RemovableCard = 0,
            EmbeddedSlot = 1,
            SharedBusSlot = 2
        ]
    ],
    pub Capability2 [
        SDR50_SUPPORT OFFSET(0) NUMBITS(1) [],
        SDR104_SUPPORT OFFSET(1) NUMBITS(1) [],
        DDR50_SUPPORT OFFSET(2) NUMBITS(1) [],
        DRV_A_SUPPORT OFFSET(4) NUMBITS(1) [],
        DRV_C_SUPPORT OFFSET(5) NUMBITS(1) [],
        DRV_D_SUPPORT OFFSET(6) NUMBITS(1) [],
        TUNE_SDR50 OFFSET(13) NUMBITS(1) [],
        RETUNE_MODE OFFSET(14) NUMBITS(2) [],
        CLK_MULTIPLIER OFFSET(16) NUMBITS(8) []
    ]
];

register_bitfields! [
    u64,
    // AMDA2 Descriptor table
    pub(crate) ADMA2_DT [
        VALID OFFSET(0) NUMBITS(1) [],
        END OFFSET(1) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) [],
        ACT OFFSET(4) NUMBITS(2) [
            NoOP = 0,
            Tran = 2,
            Link = 3
        ],
        LEN OFFSET(16) NUMBITS(16) [],
        ADDR OFFSET(32) NUMBITS(32) []
    ]
];

#[derive(Debug)]
pub enum CommandType {
    CMD(u8),
    ACMD(u8),
}

impl CommandType {
    pub fn num(&self) -> u8 {
        match self {
            CommandType::CMD(t) => *t,
            CommandType::ACMD(t) => *t,
        }
    }

    pub fn flags(&self) -> FieldValue<u32, XferCmd::Register> {
        match self {
            CommandType::CMD(17) => {
                    XferCmd::DATA_PRESENT::SET
                    + XferCmd::DAT_XFER_READ::SET
            }
            CommandType::CMD(18) => {
                XferCmd::BLK_CNT_EN::SET
                    + XferCmd::MULTI_BLK_EN::SET
                    + XferCmd::DATA_PRESENT::SET
                    + XferCmd::DAT_XFER_READ::SET
            }
            _ => XferCmd::CMD_IDX::CLEAR,
        }
    }
}
