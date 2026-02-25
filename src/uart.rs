// UART communication module
use embedded_hal::serial::{Read, Write};
use crate::constants::*;

/// UART communication wrapper
pub struct UartComm {
    rx_buffer: [u8; BL_RX_LEN],
    rx_count: usize,
}

impl UartComm {
    pub fn new() -> Self {
        UartComm {
            rx_buffer: [0; BL_RX_LEN],
            rx_count: 0,
        }
    }
    
    /// Read a single byte from UART
    pub fn read_byte<R>(serial: &mut R) -> Option<u8>
    where
        R: Read<u8>,
    {
        match nb::block!(serial.read()) {
            Ok(byte) => Some(byte),
            Err(_) => None,
        }
    }
    
    /// Write a single byte to UART
    pub fn write_byte<W>(byte: u8, serial: &mut W)
    where
        W: Write<u8>,
    {
        nb::block!(serial.write(byte)).ok();
    }
    
    /// Write a buffer to UART
    pub fn write_buffer<W>(buffer: &[u8], serial: &mut W)
    where
        W: Write<u8>,
    {
        for &byte in buffer {
            nb::block!(serial.write(byte)).ok();
        }
    }
    
    /// Send ACK response
    pub fn send_ack<W>(command_code: u8, follow_len: u8, serial: &mut W)
    where
        W: Write<u8>,
    {
        Self::write_byte(BL_ACK, serial);
        Self::write_byte(command_code, serial);
        Self::write_byte(follow_len, serial);
    }
    
    /// Send NACK response
    pub fn send_nack<W>(serial: &mut W)
    where
        W: Write<u8>,
    {
        Self::write_byte(BL_NACK, serial);
    }
    
    /// Get reference to RX buffer
    pub fn rx_buffer(&self) -> &[u8] {
        &self.rx_buffer
    }
    
    /// Get mutable reference to RX buffer
    pub fn rx_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.rx_buffer
    }
    
    /// Clear RX buffer
    pub fn clear_rx_buffer(&mut self) {
        self.rx_buffer = [0; BL_RX_LEN];
        self.rx_count = 0;
    }
}

/// Parse command packet
/// 
/// Frame structure:
/// [Length][Command][Payload...][CRC32(4 bytes)]
#[derive(Debug, Clone)]
pub struct CommandPacket {
    pub length: u8,
    pub command: u8,
    pub payload: [u8; BL_RX_LEN - 3],
    pub payload_len: usize,
    pub crc: u32,
}

impl CommandPacket {
    pub fn parse(frame: &[u8]) -> Option<Self> {
        if frame.len() < 6 {
            return None;
        }

        let length = frame[0] as usize;
        if (length + 1) != frame.len() {
            return None;
        }

        // Length includes [Command][Payload][CRC32]
        // Minimum legal value is 5 => 1 command + 4 CRC bytes.
        if length < 5 {
            return None;
        }

        let command = frame[1];
        let crc_offset = frame.len() - 4;
        let crc = u32::from_le_bytes([
            frame[crc_offset],
            frame[crc_offset + 1],
            frame[crc_offset + 2],
            frame[crc_offset + 3],
        ]);

        let payload_len = length - 1 - 4; // - command and CRC
        if payload_len > (BL_RX_LEN - 3) {
            return None;
        }

        let mut payload = [0u8; BL_RX_LEN - 3];
        if payload_len > 0 {
            payload[..payload_len].copy_from_slice(&frame[2..2 + payload_len]);
        }

        Some(CommandPacket {
            length: length as u8,
            command,
            payload,
            payload_len,
            crc,
        })
    }
}
