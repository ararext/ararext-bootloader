# Build Configuration

## Prerequisites

Install the required tools:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM Cortex-M4 target
rustup target add thumbv7em-none-eabihf

# Install ARM tools
sudo apt-get install arm-none-eabi-binutils arm-none-eabi-gdb

# Optional: Install embedded debugger support
cargo install cargo-binutils
```

## Build Process

### Development Build
```bash
cargo build
```
- Unoptimized, larger binary (~45 KB)
- Faster compilation
- Better for debugging with GDB

### Release Build
```bash
cargo build --release
```
- Optimized for size (~25 KB)
- Longer compilation time
- Better runtime performance
- Suitable for deployment

### Check Only (Faster Iteration)
```bash
cargo check
```
- Syntax and type checking only
- No code generation
- Fastest feedback loop

## Extracting Binary

From ELF executable to raw binary:

```bash
arm-none-eabi-objcopy -O binary \
    target/thumbv7em-none-eabihf/release/ararext-bootloader \
    ararext-bootloader.bin

# Verify binary size
ls -lh ararext-bootloader.bin
```

## Flashing Methods

### Method 1: ST-Link with STM32 Cube Programmer
```bash
# Use STM32CubeProgrammer (GUI or CLI)
STM32_Programmer_CLI -c port=SWD -d ararext-bootloader.bin 0x08000000 -v
```

### Method 2: OpenOCD
```bash
openocd -f interface/stlink.cfg \
        -f target/stm32f4x.cfg \
        -c "init" \
        -c "flash write_image erase ararext-bootloader.bin 0x08000000" \
        -c "verify_image ararext-bootloader.bin 0x08000000" \
        -c "reset run" \
        -c "exit"
```

### Method 3: pyOCD
```bash
pyocd flash -t stm32f407xx ararext-bootloader.bin --address 0x08000000
```

### Method 4: GDB (via OpenOCD)
```bash
# Terminal 1: Start OpenOCD
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg

# Terminal 2: GDB
arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/ararext-bootloader
(gdb) target remote :3333
(gdb) load
(gdb) continue
```

## Debugging

### With OpenOCD and GDB

```bash
# Start OpenOCD in background
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg &

# Launch GDB
arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/ararext-bootloader
(gdb) target remote :3333
(gdb) load
(gdb) break main
(gdb) continue
(gdb) step
```

### Serial Port Debugging

Monitor USART3 (Debug UART) output:

```bash
miniterm /dev/ttyUSB0 115200
# or
screen /dev/ttyUSB0 115200
```

## Performance Metrics

### Binary Size Comparison
- Debug build: ~45 KB
- Release build: ~25 KB
- Original C version: ~28 KB

### Memory Usage
- Code (Flash): ~25 KB
- Data (RAM): ~1 KB
- Available for app: ~487 KB (Flash), 111 KB (RAM1)

### Build Times
- Fresh debug build: ~30-45 seconds
- Incremental debug build: ~5-10 seconds
- Release build: ~60-90 seconds (includes LTO)

## Troubleshooting

### "error: linker 'arm-none-eabi-gcc' not found"
```bash
sudo apt-get install binutils-arm-none-eabi gcc-arm-none-eabi
```

### "error: failed to find 'rust-lld'"
```bash
rustup component add rust-src
```

### Flash write fails
- Ensure bootloader sector is not write-protected
- Use BL_DIS_R_W_PROTECT command first
- Check power supply stability

### GDB connection refused
- Verify OpenOCD is running: `openocd -f interface/stlink.cfg -f target/stm32f4x.cfg`
- Check ST-Link connection (should appear as /dev/ttyUSB*)
- Try: `openocd -c "adapter serial XXXXX"` with your device serial number

## Continuous Integration

### GitHub Actions Example
```yaml
name: Build Bootloader
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: thumbv7em-none-eabihf
      - run: cargo build --release
      - run: cargo build --release --target thumbv7em-none-eabihf
```

## Performance Optimization Tips

1. **Use Release Mode**: Always use `--release` for deployment
2. **Enable LTO**: Already configured in Cargo.toml
3. **Reduce panic overhead**: Using `panic-halt` instead of default
4. **Optimize code paths**: Profile with `cargo-flamegraph` if available

## Code Size Breakdown

Typical release binary (~25 KB):
- HAL code: 35%
- Bootloader code: 25%
- Cortex-M runtime: 20%
- Dependencies: 20%

## Building for Deployment

Final checklist:

- [ ] `cargo build --release` completes without warnings
- [ ] Binary size < 32 KB (leaving space for future)
- [ ] Flash to device and verify operation
- [ ] Test all bootloader commands
- [ ] Verify CRC validation works
- [ ] Test user app jumping
