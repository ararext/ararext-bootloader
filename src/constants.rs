// Bootloader command codes and constants
#![allow(dead_code)]

/// Bootloader version
pub const BL_VERSION: u8 = 0x10;

/// Command Codes
pub const BL_GET_VER: u8 = 0x51;
pub const BL_GET_HELP: u8 = 0x52;
pub const BL_GET_CID: u8 = 0x53;
pub const BL_GET_RDP_STATUS: u8 = 0x54;
pub const BL_GO_TO_ADDR: u8 = 0x55;
pub const BL_FLASH_ERASE: u8 = 0x56;
pub const BL_MEM_WRITE: u8 = 0x57;
pub const BL_EN_RW_PROTECT: u8 = 0x58;
pub const BL_MEM_READ: u8 = 0x59;
pub const BL_READ_SECTOR_P_STATUS: u8 = 0x5A;
pub const BL_OTP_READ: u8 = 0x5B;
pub const BL_DIS_R_W_PROTECT: u8 = 0x5C;

/// Response codes
pub const BL_ACK: u8 = 0xA5;
pub const BL_NACK: u8 = 0x7F;

/// CRC verification results
pub const VERIFY_CRC_SUCCESS: u8 = 0;
pub const VERIFY_CRC_FAIL: u8 = 1;

/// Address validation results
pub const ADDR_VALID: u8 = 0x00;
pub const ADDR_INVALID: u8 = 0x01;

/// Sector errors
pub const INVALID_SECTOR: u8 = 0x04;

/// Memory addresses for STM32F407xx
pub const SRAM1_BASE: u32 = 0x20000000;
pub const SRAM1_SIZE: u32 = 112 * 1024;
pub const SRAM1_END: u32 = SRAM1_BASE + SRAM1_SIZE;

pub const SRAM2_BASE: u32 = 0x2001C000;
pub const SRAM2_SIZE: u32 = 16 * 1024;
pub const SRAM2_END: u32 = SRAM2_BASE + SRAM2_SIZE;

pub const FLASH_BASE: u32 = 0x08000000;
pub const FLASH_SIZE: u32 = 512 * 1024;
pub const FLASH_END: u32 = FLASH_BASE + FLASH_SIZE;

pub const BKPSRAM_BASE: u32 = 0x40024000;
pub const BKPSRAM_SIZE: u32 = 4 * 1024;
pub const BKPSRAM_END: u32 = BKPSRAM_BASE + BKPSRAM_SIZE;

/// User application flash sector
pub const FLASH_SECTOR2_BASE_ADDRESS: u32 = 0x08008000;

/// Maximum receive buffer length
pub const BL_RX_LEN: usize = 200;

/// Supported commands list
pub const SUPPORTED_COMMANDS: &[u8] = &[
    BL_GET_VER,
    BL_GET_HELP,
    BL_GET_CID,
    BL_GET_RDP_STATUS,
    BL_GO_TO_ADDR,
    BL_FLASH_ERASE,
    BL_MEM_WRITE,
    BL_EN_RW_PROTECT,
    BL_MEM_READ,
    BL_READ_SECTOR_P_STATUS,
    BL_OTP_READ,
    BL_DIS_R_W_PROTECT,
];
