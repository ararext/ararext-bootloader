// CRC verification module
use stm32f4xx_hal::crc::Crc;
use crate::constants::*;

/// Verify CRC of a data buffer
/// 
/// # Arguments
/// * `crc` - CRC peripheral
/// * `data` - Data buffer to verify
/// * `crc_host` - CRC value from host
/// 
/// # Returns
/// VERIFY_CRC_SUCCESS if CRC matches, VERIFY_CRC_FAIL otherwise
pub fn verify_crc(crc: &mut Crc, data: &[u8], crc_host: u32) -> u8 {
    let mut crc_value: u32 = 0xffffffff;
    
    for &byte in data {
        let i_data = byte as u32;
        crc_value = crc.calculate(i_data);
    }
    
    // Reset CRC for next calculation
    drop(crc.reset());
    
    if crc_value == crc_host {
        VERIFY_CRC_SUCCESS
    } else {
        VERIFY_CRC_FAIL
    }
}

/// Calculate CRC of a data buffer
pub fn calculate_crc(crc: &mut Crc, data: &[u8]) -> u32 {
    let mut crc_value: u32 = 0xffffffff;
    
    for &byte in data {
        crc_value = crc.calculate(byte as u32);
    }
    
    drop(crc.reset());
    crc_value
}
