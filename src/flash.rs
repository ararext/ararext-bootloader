// Flash memory operations module
use stm32f4xx_hal::flash::Flash;
use crate::constants::*;

/// Flash sector configuration for STM32F407xx
#[derive(Debug, Clone, Copy)]
pub struct FlashSector {
    pub number: u8,
    pub base_address: u32,
    pub size: u32,
}

impl FlashSector {
    /// Get sector information for STM32F407xx
    /// Sectors 0-7 for STM32F407xx
    pub fn get_sector_info(sector_num: u8) -> Option<FlashSector> {
        match sector_num {
            0 => Some(FlashSector { number: 0, base_address: 0x08000000, size: 16 * 1024 }),
            1 => Some(FlashSector { number: 1, base_address: 0x08004000, size: 16 * 1024 }),
            2 => Some(FlashSector { number: 2, base_address: 0x08008000, size: 16 * 1024 }),
            3 => Some(FlashSector { number: 3, base_address: 0x0800C000, size: 16 * 1024 }),
            4 => Some(FlashSector { number: 4, base_address: 0x08010000, size: 64 * 1024 }),
            5 => Some(FlashSector { number: 5, base_address: 0x08020000, size: 128 * 1024 }),
            6 => Some(FlashSector { number: 6, base_address: 0x08040000, size: 128 * 1024 }),
            7 => Some(FlashSector { number: 7, base_address: 0x08060000, size: 128 * 1024 }),
            _ => None,
        }
    }
}

/// Erase flash sectors
/// 
/// # Arguments
/// * `flash` - Flash controller
/// * `sector_number` - Starting sector (0-7), or 0xFF for mass erase
/// * `number_of_sectors` - Number of sectors to erase
/// 
/// # Returns
/// Ok(()) on success, Err on failure
pub fn execute_flash_erase(
    flash: &mut Flash,
    sector_number: u8,
    number_of_sectors: u8,
) -> Result<(), &'static str> {
    if number_of_sectors > 8 {
        return Err("Invalid number of sectors");
    }
    
    if sector_number == 0xFF {
        // Mass erase - erase all user sectors
        flash.clear_all_err_flags();
        
        for i in 0..8 {
            flash.erase(i).map_err(|_| "Mass erase failed")?;
        }
        
        Ok(())
    } else if sector_number <= 7 {
        // Erase specific sectors
        flash.clear_all_err_flags();
        
        let remaining_sectors = 8 - sector_number;
        let sectors_to_erase = if number_of_sectors > remaining_sectors {
            remaining_sectors
        } else {
            number_of_sectors
        };
        
        for i in 0..sectors_to_erase {
            flash.erase(sector_number + i)
                .map_err(|_| "Sector erase failed")?;
        }
        
        Ok(())
    } else {
        Err("Invalid sector number")
    }
}

/// Write data to flash memory byte-by-byte
/// 
/// # Arguments
/// * `flash` - Flash controller
/// * `address` - Target address in flash
/// * `data` - Data to write
/// 
/// # Returns
/// Ok(()) on success, Err on failure
pub fn execute_mem_write(
    flash: &mut Flash,
    address: u32,
    data: &[u8],
) -> Result<(), &'static str> {
    flash.clear_all_err_flags();
    
    for (offset, &byte) in data.iter().enumerate() {
        let write_addr = address + offset as u32;
        
        flash.program(write_addr, &[byte], false)
            .map_err(|_| "Write failed")?;
    }
    
    Ok(())
}

/// Configure flash sector read/write protection
/// 
/// # Arguments
/// * `sector_details` - Bitmap of sectors to protect
/// * `protection_mode` - 1 for write protect, 2 for read/write protect
/// * `disable` - true to disable protection
pub fn configure_flash_sector_rw_protection(
    sector_details: u8,
    protection_mode: u8,
    disable: bool,
) -> Result<(), &'static str> {
    const FLASH_OPTCR: u32 = 0x40023C14;
    let optcr_ptr = FLASH_OPTCR as *mut u32;
    
    unsafe {
        if disable {
            // Unlock option bytes
            let optcr_val = core::ptr::read_volatile(optcr_ptr);
            
            // Clear bit 31 (disable protection)
            let new_val = optcr_val & !(1 << 31);
            // Set all protection bits (no protection)
            let new_val = new_val | (0xFF << 16);
            // Set OPTSTRT bit
            let new_val = new_val | (1 << 1);
            
            core::ptr::write_volatile(optcr_ptr, new_val);
            
            // Wait for operation to complete
            while (core::ptr::read_volatile(optcr_ptr) & 1) != 0 {}
        } else if protection_mode == 1 {
            // Write protection only
            let optcr_val = core::ptr::read_volatile(optcr_ptr);
            let new_val = optcr_val & !(1 << 31);
            let new_val = new_val & !(((sector_details as u32) << 16));
            let new_val = new_val | (1 << 1);
            
            core::ptr::write_volatile(optcr_ptr, new_val);
            while (core::ptr::read_volatile(optcr_ptr) & 1) != 0 {}
        } else if protection_mode == 2 {
            // Read/write protection
            let optcr_val = core::ptr::read_volatile(optcr_ptr);
            let new_val = optcr_val | (1 << 31);
            let new_val = new_val & !(0xFF << 16);
            let new_val = new_val | (((sector_details as u32) << 16));
            let new_val = new_val | (1 << 1);
            
            core::ptr::write_volatile(optcr_ptr, new_val);
            while (core::ptr::read_volatile(optcr_ptr) & 1) != 0 {}
        }
    }
    
    Ok(())
}

/// Read option byte read/write protection status
pub fn read_ob_rw_protection_status() -> u16 {
    const OB_ADDR: u32 = 0x1FFFC008;
    let ob_ptr = OB_ADDR as *const u16;
    unsafe { core::ptr::read_volatile(ob_ptr) }
}
