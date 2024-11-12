//! Intel Corporation USB Register Documentation
//!
//! # Intel Corporation USB Register Documentation PDF
//! https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/ehci-specification-for-usb.pdf
//!

use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_structs! {
    pub(crate) CapRegister {
        /// Capability Register Length
        (0x00 => pub caplength: ReadOnly<u8>),
        (0x01 => _reserved: ReadOnly<u8>),
        /// Interface Version Number
        (0x02 => pub hci_version: ReadOnly<u16>),
        /// Structural Parameters
        (0x04 => pub hcs_params: ReadOnly<u32, HCSPARAMS::Register>),
        /// Capability Parameters
        (0x08 => pub hcc_params: ReadOnly<u32, HCCPARAMS::Register>),
        /// Companion Port Route Description
        (0x0c => pub hcsp_portroute: [ReadOnly<u8>; 8]),
        (0x14 => @END),
    }
}

register_structs! {
    pub(crate) OpRegister {
        /// USB Command Register
        (0x00 => pub cmd: ReadWrite<u32, USBCMD::Register>),
        /// USB Status Register
        (0x04 => pub sts: ReadWrite<u32, USBSTS::Register>),
        /// USB Interrupt Enable Register
        (0x08 => pub intr: ReadWrite<u32, USBINTR::Register>),
        /// Frame Index Register
        (0x0c => pub fridx: ReadWrite<u32>),
        /// Control Data Structure Segment Register
        (0x10 => pub ctr: ReadWrite<u32>),
        /// Periodic Frame List Base Address Register
        (0x14 => pub pfladdr: ReadWrite<u32>),
        /// Current Asynchronous List Address Register
        (0x18 => pub calar: ReadWrite<u32>),
        (0x1c => _reserved),
        /// Configure Flag Register
        (0x40 => pub cfr: ReadWrite<u32, USBCFR::Register>),
        /// Port Status and Control Register
        (0x44 => pub psc: [ReadWrite<u32, USBPSC::Register>; 15]),
        (0x80 => @END),
    }
}

register_bitfields! [
    u32,
    pub HCSPARAMS [
        N_PORTS OFFSET(0) NUMBITS(4),
        /// Port Power Control
        PPC OFFSET(4) NUMBITS(1),
        /// Port Routing Rules
        PRR OFFSET(5) NUMBITS(1)
        // TODO: Another bitfields for other parameters
    ],
    pub HCCPARAMS [
        /// 64-bit Addressing Capability1 .
        /// This field documents the addressing range capability of this implementation
        ADDR_64 OFFSET(0) NUMBITS(1),
        /// Programmable Frame List Flag
        ///
        PRL_FLAGS OFFSET(1) NUMBITS(1),
        /// Isochronous Scheduling Threshold
        ///
        IST OFFSET(4) NUMBITS(4),
        /// EHCI Extended Capabilities Pointer (EECP)
        EECP OFFSET(8) NUMBITS(8)
    ],
    pub USBCMD [
        /// Run/Stop (RS)
        RUN OFFSET(0) NUMBITS(1) [],
        /// Host Controller Reset (HCRESET)
        HCRST OFFSET(1) NUMBITS(1) [],
        /// Frame List Size, DEFAULT 00
        FLS OFFSET(2) NUMBITS(2) [    
            BYTE1024 = 0b00,
            BYTE512 = 0b01,
            BYTE256 = 0b10,
            BYTE128 = 0b11
        ],
        /// Periodic Schedule Enable
        PS_EN OFFSET(4) NUMBITS(1) [],
        /// Asynchronous Schedule Enable
        AS_EN OFFSET(5) NUMBITS(1) [],
        /// Interrupt on Async Advance Doorbell
        IAAD OFFSET(6) NUMBITS(1) [],
        /// Light Host Controller Reset (OPTIONAL)
        LHCR OFFSET(7) NUMBITS(1) [],
        /// Asynchronous Schedule Park Mode Count (OPTIONAL)
        ASPM_CNT OFFSET(8) NUMBITS(2) [],
        /// Asynchronous Schedule Park Mode Enable (OPTIONAL)
        ASPM_EN OFFSET(10) NUMBITS(1) [],
        /// Interrupt Threshold Control
        ITC OFFSET(16) NUMBITS(8) [],
    ],
    pub USBSTS [
        /// USB Interrupt
        INT     0,
        /// USB Error Interrupt
        ERR_INT 1,
        /// Port Change Detect
        PCDT    2,
        /// Frame List Rollover 
        FLR     3,
        /// Host System Error
        HS_ERR  4,
        /// Interrupt on Async Advance
        IAA     5,
        /// HCHalted, Default is 1
        /// This bit is a zero whenever the Run/Stop bit is a one
        HCH     12,
        /// Reclamation
        REC     13,
        /// Periodic Schedule Status
        PSS     14,
        /// Asynchronous Schedule Status
        ASS     15,
    ],
    pub USBINTR [
        /// USB Interrupt
        INT_EN     0,
        /// USB Error Interrupt
        ERR_INT_EN 1,
        /// Port Change Interrupt Enable.
        PCI_EN    2,
        /// Frame List Rollover Enable.
        FLR_EN     3,
        /// Host System Error Enable
        HS_ERR_EN  4,
        /// Interrupt on Async Advance Enable
        IAA_EN     5,
    ],
    pub USBCFR [
        /// Configure Flag for Port
        CF OFFSET(0) NUMBITS(1) [],
    ],
    pub USBPSC [
        /// Current Connect Status
        CONN_STATUS OFFSET(0) NUMBITS(1) [],
        /// Connect Status Change
        CONN_CHANGE OFFSET(1) NUMBITS(1) [],
        /// Port Enabled/Disabled
        PORT_EN     OFFSET(2) NUMBITS(1) [],
        /// Port Enable/Disable Change
        PORT_EN_CHANGE OFFSET(3) NUMBITS(1) [],
        /// Over-current Active
        OCA       OFFSET(4) NUMBITS(1) [],
        /// Over-current Detected
        OCD       OFFSET(5) NUMBITS(1) [],
        /// Force Port Resume
        FPR       OFFSET(6) NUMBITS(1) [],
        /// SUSPEND      1: SUSPENDED
        SUSPEND   OFFSET(7) NUMBITS(1) [],
        /// Port Reset   1: IN RESET   
        RESET     OFFSET(8) NUMBITS(1) [],
        /// Line Status
        LINE_STATUS OFFSET(10) NUMBITS(2) [
            /// Not Low-speed device, perform EHCI reset.
            IDLE = 3,
            /// Low-speed device, release ownership of port
            J = 2,
            /// Not Low-speed device, perform EHCI reset
            K = 1,
            /// Not Low-speed device, perform EHCI reset
            SE0 = 0
        ],
        /// Port Power
        POWER     OFFSET(12) NUMBITS(1) [],
        /// Porn Owner
        OWNER     OFFSET(13) NUMBITS(1) [],
        /// Port Indicator Control.
        PIC       OFFSET(14) NUMBITS(2) [
            OFF = 0,
            AMBER = 1,
            GREEN = 2,
            UNDEF = 3
        ],
        /// Port Test Control
        /// When this field is zero, the port is NOT operating in a test mode. 
        PTC       OFFSET(16) NUMBITS(4) [
            DISABLE = 0,
            J_STATE = 1,
            K_STATE = 2,
            SE0_NAK = 3,
            PACKAGE = 4,
            FORCE_ENABLE = 5,
        ],
        /// Wake on Connect Enable (WKCNNT_E)
        WKCNNT_E  OFFSET(20) NUMBITS(1) [],
        /// Wake on Disconnect Enable (WKDSCNNT_E)
        WKDSCNNT_E OFFSET(21) NUMBITS(1) [],
        /// Wake on Over-current Enable (WKOC_E)
        WKOC_E    OFFSET(22) NUMBITS(1) [],
    ]
];

register_bitfields! [
    u32,
    /// Horizontal Link Pointer
    pub HLP [
        /// Set if this is the last Queue Head in a Periodic List. 
        /// Not used for Asynchronous List. 
        TERMINATE OFFSET(0) NUMBITS(1) [],
        /// Next Queue Type
        QUEUE_TYPE OFFSET(1) NUMBITS(2) [
            Isochronous = 0,
            QUEUE_HEAD = 1,
            /// Split Transaction Isochronous TD
            SPLIT_ISOCHRONOUS = 2,
            /// Frame Span Traversal Node
            Node = 3
        ],
        QUEUE_HEAD OFFSET(5) NUMBITS(27) [],
    ],
    /// Endpoint Characteristics
    pub EP_CHAR [
        /// Device Address
        DEV_ADDR OFFSET(0) NUMBITS(7) [],
        /// Inactivate
        /// Only used in Periodic List 
        INACTIVATE OFFSET(7) NUMBITS(1) [],
        /// Endpoint Number
        EP_NUM OFFSET(8) NUMBITS(4) [],
        /// Endpoint Speed
        EP_SPEED OFFSET(12) NUMBITS(2) [
            FULL_SPEED = 0,
            LOW_SPEED = 1,
            HIGH_SPEED = 2
        ],
        /// Data Toggle Control
        /// Set if data toggle should use value from TD
        DTC OFFSET(14) NUMBITS(1) [],
        /// Head of Reclamation List
        /// Set if this is the first Queue Head in an Asynchronous List 
        HRL OFFSET(15) NUMBITS(1) [],
        /// Maximum Packet Length
        MAX_PAC_LEN OFFSET(16) NUMBITS(11) [],
        /// Control Endpoint
        CE OFFSET(27) NUMBITS(1) [],
        /// NAK Reload
        NAK_RELOAD OFFSET(28) NUMBITS(4) [],
    ],
    /// Endpoint Capabilities
    pub EP_CAP [
        /// Interrupt Schedule Mask
        /// Used for split transactions 
        ISM OFFSET(0) NUMBITS(8) [],
        /// Split Completion Mask
        /// Used for split transactions 
        SCM OFFSET(8) NUMBITS(8) [],
        /// Hub Address
        /// Used for split transactions 
        HA  OFFSET(16) NUMBITS(7) [],
        /// Port Number
        /// Used for split transactions 
        PN  OFFSET(23) NUMBITS(7) [],
        /// High Bandwidth Pipe Multiplier
        /// Must be greater than zero 
        HBPM OFFSET(30) NUMBITS(2) [],
    ]
];

#[repr(C)]
/// EHCI Queue Head (QH) structure
pub struct EHCIQH {
    /// Horizontal Link Pointer
    pub(crate) hlp: u32,
    /// Endpoint Characteristics
    pub(crate) ep_char: u32,
    /// Endpoint Capabilities
    pub(crate) ep_cap: u32,
    /// Current TD Address
    pub(crate) cur_link: u32,


    pub(crate) nxt_link: u32,
    pub(crate) alt_link: u32,
    pub(crate) token: u32,
    pub(crate) buffer: [u32; 5],
    pub(crate) ext_buffer: [u32; 5],
}

#[repr(C)]
pub struct EHCICmd {
    pub(crate) b_request_t: u8,
    pub(crate) b_request: u8,
    pub(crate) w_value: u16,
    pub(crate) w_index: u16,
    pub(crate) w_length: u16,
}

#[repr(C)]
pub struct EHCITD {
    pub(crate) nxt_link: u32,
    pub(crate) alt_link: u32,
    pub(crate) token: u32,
    pub(crate) buffer: [u32; 5],
    pub(crate) ext_buffer: [u32; 5],
}
