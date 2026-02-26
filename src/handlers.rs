// Bootloader command handlers
use crate::constants::*;
use crate::uart::UartComm;
use crate::memory;
use embedded_hal::serial::Write;

pub trait CommandHandler {
    fn handle_get_version(&self);
    fn handle_get_help(&self);
    fn handle_get_cid(&self);
    fn handle_get_rdp_status(&self);
    fn handle_go_to_address(&self, address: u32);
    fn handle_flash_erase(&self, sector: u8, count: u8);
    fn handle_mem_write(&self, address: u32, data: &[u8]);
    fn handle_mem_read(&self, address: u32, length: u32);
    fn handle_enable_rw_protect(&self, sectors: u8, mode: u8);
    fn handle_disable_rw_protect(&self);
    fn handle_read_sector_protection(&self);
    fn handle_read_otp(&self);
}

/// Get bootloader version
pub fn get_bootloader_version() -> u8 {
    BL_VERSION
}

/// Handle BL_GET_VER command
pub fn handle_getver_cmd<W: Write<u8>>(_packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    let version = get_bootloader_version();
    
    UartComm::send_ack(BL_GET_VER, 1, serial);
    UartComm::write_byte(version, serial);
}

/// Handle BL_GET_HELP command
pub fn handle_gethelp_cmd<W: Write<u8>>(_uart: &mut UartComm, serial: &mut W) {
    let num_commands = SUPPORTED_COMMANDS.len() as u8;
    
    UartComm::send_ack(BL_GET_HELP, num_commands, serial);
    UartComm::write_buffer(SUPPORTED_COMMANDS, serial);
}

/// Handle BL_GET_CID command
pub fn handle_getcid_cmd<W: Write<u8>>(_uart: &mut UartComm, serial: &mut W) {
    let chip_id = memory::get_mcu_chip_id();
    let cid_bytes = chip_id.to_le_bytes();
    
    UartComm::send_ack(BL_GET_CID, 2, serial);
    UartComm::write_buffer(&cid_bytes, serial);
}

/// Handle BL_GET_RDP_STATUS command
pub fn handle_getrdp_cmd<W: Write<u8>>(_uart: &mut UartComm, serial: &mut W) {
    let rdp_level = memory::get_flash_rdp_level();
    
    UartComm::send_ack(BL_GET_RDP_STATUS, 1, serial);
    UartComm::write_byte(rdp_level, serial);
}

/// Handle BL_GO_TO_ADDR command
pub fn handle_go_cmd<W: Write<u8>>(packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    if packet.len() < 4 {
        UartComm::send_nack(serial);
        return;
    }
    
    let address = u32::from_le_bytes([packet[0], packet[1], packet[2], packet[3]]);
    
    if memory::verify_address(address) == ADDR_VALID {
        UartComm::send_ack(BL_GO_TO_ADDR, 0, serial);
        
        // Small delay to ensure ACK is transmitted
        cortex_m::asm::delay(1000);
        
        // Jump to address
        jump_to_address(address);
    } else {
        UartComm::send_nack(serial);
    }
}

/// Handle BL_FLASH_ERASE command
pub fn handle_flash_erase_cmd<W: Write<u8>>(packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    if packet.len() < 2 {
        UartComm::send_nack(serial);
        return;
    }
    
    let _sector_number = packet[0];
    let _number_of_sectors = packet[1];

    // Command currently not wired to flash controller in main context.
    UartComm::send_nack(serial);
}

/// Handle BL_MEM_WRITE command
pub fn handle_mem_write_cmd<W: Write<u8>>(packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    if packet.len() < 6 {
        UartComm::send_nack(serial);
        return;
    }
    
    let address = u32::from_le_bytes([packet[0], packet[1], packet[2], packet[3]]);
    let write_len = packet[4];
    
    if (5 + write_len as usize) > packet.len() {
        UartComm::send_nack(serial);
        return;
    }
    
    let _data = &packet[5..5 + write_len as usize];
    
    // Verify address first, but this command is currently not wired to flash controller.
    if memory::verify_address(address) == ADDR_VALID {
        UartComm::send_nack(serial);
    } else {
        UartComm::send_nack(serial);
    }
}

/// Handle BL_MEM_READ command
pub fn handle_mem_read_cmd<W: Write<u8>>(packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    if packet.len() < 6 {
        UartComm::send_nack(serial);
        return;
    }
    
    let address = u32::from_le_bytes([packet[0], packet[1], packet[2], packet[3]]);
    let read_len = packet[4];
    
    if memory::verify_address(address) == ADDR_VALID {
        UartComm::send_ack(BL_MEM_READ, read_len, serial);
        
        // Read and send data
        for i in 0..read_len {
            let addr = (address + i as u32) as *const u8;
            let byte = unsafe { core::ptr::read_volatile(addr) };
            UartComm::write_byte(byte, serial);
        }
    } else {
        UartComm::send_nack(serial);
    }
}

/// Handle BL_EN_RW_PROTECT command
pub fn handle_en_rw_protect_cmd<W: Write<u8>>(packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    if packet.len() < 2 {
        UartComm::send_nack(serial);
        return;
    }
    
    UartComm::send_nack(serial);
}

/// Handle BL_DIS_R_W_PROTECT command
pub fn handle_dis_rw_protect_cmd<W: Write<u8>>(_packet: &[u8], _uart: &mut UartComm, serial: &mut W) {
    UartComm::send_nack(serial);
}

/// Handle BL_READ_SECTOR_P_STATUS command
pub fn handle_read_sector_protection_cmd<W: Write<u8>>(_uart: &mut UartComm, serial: &mut W) {
    let protection_status = crate::flash::read_ob_rw_protection_status();
    let status_bytes = protection_status.to_le_bytes();
    
    UartComm::send_ack(BL_READ_SECTOR_P_STATUS, 2, serial);
    UartComm::write_buffer(&status_bytes, serial);
}

/// Handle BL_OTP_READ command
pub fn handle_read_otp_cmd<W: Write<u8>>(_uart: &mut UartComm, serial: &mut W) {
    // OTP read - stub for now
    UartComm::send_nack(serial);
}

/// Jump to application code
/// 
/// This function:
/// 1. Sets the MSP (Main Stack Pointer) from the app reset vector
/// 2. Jumps to the app reset handler
#[inline(never)]
fn jump_to_address(address: u32) -> ! {
    unsafe {
        // Configure MSP from app base address
        let msp = core::ptr::read_volatile(address as *const u32);
        cortex_m::register::msp::write(msp);
        
        // Fetch reset handler address (at address + 4)
        let reset_handler = core::ptr::read_volatile((address + 4) as *const u32);
        
        // Create function pointer and jump
        let jump: extern "C" fn() -> ! = core::mem::transmute(reset_handler);
        jump();
    }
}
