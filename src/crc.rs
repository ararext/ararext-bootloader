// CRC verification module
//
// Frame format:
// [Length][Command][Payload...][CRC32 (LE)]

const CRC_INIT: u32 = 0xFFFF_FFFF;
const CRC_POLY: u32 = 0x04C11DB7;

/// Verify the CRC of a full protocol frame.
/// Returns false for malformed frames.
pub fn verify_frame_crc(frame: &[u8]) -> bool {
    if frame.len() < 6 {
        return false;
    }

    let data_len = frame.len() - 4;
    let expected = u32::from_le_bytes([
        frame[data_len],
        frame[data_len + 1],
        frame[data_len + 2],
        frame[data_len + 3],
    ]);

    calculate_crc(&frame[..data_len]) == expected
}

/// Calculate CRC over a byte slice using the same byte-fed 32-bit update model
/// as the original implementation.
pub fn calculate_crc(data: &[u8]) -> u32 {
    let mut crc_value = CRC_INIT;

    for &byte in data {
        crc_value = accumulate_word_crc(crc_value, byte as u32);
    }

    crc_value
}

fn accumulate_word_crc(mut crc: u32, data: u32) -> u32 {
    crc ^= data;

    for _ in 0..32 {
        if (crc & 0x8000_0000) != 0 {
            crc = (crc << 1) ^ CRC_POLY;
        } else {
            crc <<= 1;
        }
    }

    crc
}
