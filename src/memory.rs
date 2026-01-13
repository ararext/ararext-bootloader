// Memory and address validation module
use crate::constants::*;

/// Verify if an address is valid for jumping
/// 
/// Valid regions:
/// - SRAM1 (main RAM)
/// - SRAM2 (secondary RAM)
/// - FLASH (program memory)
/// - Backup SRAM
/// 
/// # Arguments
/// * `address` - Address to validate
/// 
/// # Returns
/// ADDR_VALID if address is in a valid region, ADDR_INVALID otherwise
pub fn verify_address(address: u32) -> u8 {
    if (address >= SRAM1_BASE && address <= SRAM1_END) ||
       (address >= SRAM2_BASE && address <= SRAM2_END) ||
       (address >= FLASH_BASE && address <= FLASH_END) ||
       (address >= BKPSRAM_BASE && address <= BKPSRAM_END) {
        ADDR_VALID
    } else {
        ADDR_INVALID
    }
}

/// Get MCU chip ID
/// 
/// Reads from DBGMCU->IDCODE register
pub fn get_mcu_chip_id() -> u16 {
    // DBGMCU base address for STM32F4
    const DBGMCU_BASE: u32 = 0xE0042000;
    const DBGMCU_IDCODE_OFFSET: u32 = 0x00;
    
    let idcode_ptr = (DBGMCU_BASE + DBGMCU_IDCODE_OFFSET) as *const u32;
    let idcode = unsafe { core::ptr::read_volatile(idcode_ptr) };
    
    // Extract chip ID (bits [11:0])
    (idcode & 0x0FFF) as u16
}

/// Get Flash Read Protection (RDP) level
/// 
/// Reads from option bytes at 0x1FFFC000
pub fn get_flash_rdp_level() -> u8 {
    const OB_ADDR: u32 = 0x1FFFC000;
    let ob_ptr = OB_ADDR as *const u32;
    let ob_value = unsafe { core::ptr::read_volatile(ob_ptr) };
    
    // RDP level is at bits [15:8]
    ((ob_value >> 8) & 0xFF) as u8
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryRegion {
    SRAM1,
    SRAM2,
    Flash,
    BackupSram,
    Unknown,
}

/// Identify which memory region an address belongs to
pub fn identify_memory_region(address: u32) -> MemoryRegion {
    match address {
        addr if addr >= SRAM1_BASE && addr <= SRAM1_END => MemoryRegion::SRAM1,
        addr if addr >= SRAM2_BASE && addr <= SRAM2_END => MemoryRegion::SRAM2,
        addr if addr >= FLASH_BASE && addr <= FLASH_END => MemoryRegion::Flash,
        addr if addr >= BKPSRAM_BASE && addr <= BKPSRAM_END => MemoryRegion::BackupSram,
        _ => MemoryRegion::Unknown,
    }
}
