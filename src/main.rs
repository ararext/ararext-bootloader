// ararext Bootloader for STM32F407xx
// A high-performance bootloader implementation in Rust

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::stm32;

mod constants;
mod crc;
mod flash;
mod memory;
mod uart;
mod handlers;

use constants::*;
use uart::{UartComm, CommandPacket};
use handlers::*;

/// System initialization and main bootloader loop
#[entry]
fn main() -> ! {
    // Get peripherals
    let dp = stm32::Peripherals::take().unwrap();
    
    // Setup clocks
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(84.mhz())
        .pclk1(42.mhz())
        .pclk2(84.mhz())
        .freeze();
    
    // Setup GPIO
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    
    // Button on PA0 (B1)
    let button = gpioa.pa0.into_pull_up_input();
    
    // LED on PA5 (LD2)
    let mut led = gpioa.pa5.into_push_pull_output();
    
    // Setup USART2 (Command/Control UART)
    // TX: PA2, RX: PA3
    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7();
    let serial = stm32f4xx_hal::serial::Serial::usart2(
        dp.USART2,
        (tx, rx),
        Config::default().baudrate(115_200.bps()),
        clocks,
    ).unwrap();
    
    let (mut tx, mut rx) = serial.split();
    
    // Setup USART3 (Debug output UART)
    // TX: PB10, RX: PB11
    let tx_debug = gpiob.pb10.into_alternate_af7();
    let rx_debug = gpiob.pb11.into_alternate_af7();
    let _serial_debug = stm32f4xx_hal::serial::Serial::usart3(
        dp.USART3,
        (tx_debug, rx_debug),
        Config::default().baudrate(115_200.bps()),
        clocks,
    ).unwrap();
    
    // Brief startup sequence
    for _ in 0..3 {
        led.set_high();
        cortex_m::asm::delay(8_400_000); // ~0.1s at 84MHz
        led.set_low();
        cortex_m::asm::delay(8_400_000);
    }
    
    // Check button to decide: bootloader or jump to app
    if button.is_low().unwrap_or(false) {
        // Button pressed - enter bootloader mode
        led.set_high();
        bootloader_loop(&mut rx, &mut tx);
    } else {
        // Button not pressed - jump to user application
        led.set_low();
        jump_to_user_app();
    }
}

/// Main bootloader command loop
fn bootloader_loop(
    rx: &mut stm32f4xx_hal::serial::Rx<stm32::USART2>,
    tx: &mut stm32f4xx_hal::serial::Tx<stm32::USART2>,
) -> ! {
    use embedded_hal::serial::Read;
    
    let mut uart = UartComm::new();
    
    'boot: loop {
        // Read command length (first byte)
        let length = match nb::block!(rx.read()) {
            Ok(byte) => byte,
            Err(_) => {
                UartComm::send_nack(tx);
                continue;
            }
        };

        // Frame length includes the first length byte.
        let frame_len = (length as usize) + 1;
        if frame_len > BL_RX_LEN || length < 5 {
            UartComm::send_nack(tx);
            continue;
        }
        
        // Read command packet
        let mut buffer = [0u8; BL_RX_LEN];
        buffer[0] = length;
        
        for i in 1..frame_len {
            match nb::block!(rx.read()) {
                Ok(byte) => buffer[i] = byte,
                Err(_) => {
                    UartComm::send_nack(tx);
                    continue 'boot;
                }
            }
        }

        let frame = &buffer[..frame_len];

        if !crc::verify_frame_crc(frame) {
            UartComm::send_nack(tx);
            continue;
        }
        
        // Parse command packet
        if let Some(packet) = CommandPacket::parse(frame) {
            match packet.command {
                BL_GET_VER => {
                    handle_getver_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_GET_HELP => {
                    handle_gethelp_cmd(&mut uart, tx);
                }
                BL_GET_CID => {
                    handle_getcid_cmd(&mut uart, tx);
                }
                BL_GET_RDP_STATUS => {
                    handle_getrdp_cmd(&mut uart, tx);
                }
                BL_GO_TO_ADDR => {
                    handle_go_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_FLASH_ERASE => {
                    handle_flash_erase_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_MEM_WRITE => {
                    handle_mem_write_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_EN_RW_PROTECT => {
                    handle_en_rw_protect_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_MEM_READ => {
                    handle_mem_read_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                BL_READ_SECTOR_P_STATUS => {
                    handle_read_sector_protection_cmd(&mut uart, tx);
                }
                BL_OTP_READ => {
                    handle_read_otp_cmd(&mut uart, tx);
                }
                BL_DIS_R_W_PROTECT => {
                    handle_dis_rw_protect_cmd(&packet.payload[..packet.payload_len], &mut uart, tx);
                }
                _ => {
                    // Unknown command
                    UartComm::send_nack(tx);
                }
            }
        } else {
            UartComm::send_nack(tx);
        }
    }
}

/// Jump to user application
/// 
/// This function assumes the user application is located at FLASH_SECTOR2_BASE_ADDRESS.
/// It configures the MSP and jumps to the reset handler.
fn jump_to_user_app() -> ! {
    unsafe {
        // Configure MSP from app reset vector at FLASH_SECTOR2_BASE_ADDRESS
        let msp = core::ptr::read_volatile(FLASH_SECTOR2_BASE_ADDRESS as *const u32);
        cortex_m::register::msp::write(msp);
        
        // Fetch reset handler address (at FLASH_SECTOR2_BASE_ADDRESS + 4)
        let reset_handler = core::ptr::read_volatile(
            (FLASH_SECTOR2_BASE_ADDRESS + 4) as *const u32
        );
        
        // Jump to reset handler
        let jump: extern "C" fn() -> ! = core::mem::transmute(reset_handler);
        jump()
    }
}
