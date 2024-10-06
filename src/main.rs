#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[allow(dead_code)]
fn fast_config() -> embassy_stm32::Config {
    let mut cfg = embassy_stm32::Config::default();
    cfg.rcc.hse = Some(embassy_stm32::rcc::Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: embassy_stm32::rcc::HseMode::Oscillator,
    });
    cfg.rcc.apb1_pre = embassy_stm32::rcc::APBPrescaler::DIV2;
    cfg.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_P;
    cfg.rcc.pll = Some(embassy_stm32::rcc::Pll {
        src: embassy_stm32::rcc::PllSource::HSE,
        prediv: embassy_stm32::rcc::PllPreDiv::DIV1,
        mul: embassy_stm32::rcc::PllMul::MUL9,
    });
    cfg
}

fn print_chip_info() {
    // STM32F10x maps CoreSight debug ROM table at 0xE00FF000 - 0xE00FFFFF.
    // The format is described by ARM Debug Interface spec, see section 13.3,
    // "The Peripheral ID Registers".
    //
    // The relevant entries are:
    // addr   reg     stm32_val    description
    // =======================================
    // 0xFD0  (ID4)   0000 0000    [3:0] is the JEP106 page number [*]
    // 0xFE4  (ID1)   0000 0100    [7:4] is the JEP106 bits [3:0]
    // 0xFE8  (ID2)   0000 1010    [2:0] is the JEP106 bits [6:4], [3] is 1 if the code is used.
    //
    // [*] JEP106 encodes the page by the number of leading continuation
    // bytes, but ARM just encodes the number (0-15) of these bytes.
    let id4 = unsafe { *(0xE00F_FFD0 as *const u32) };
    let id1 = unsafe { *(0xE00F_FFE4 as *const u32) };
    let id2 = unsafe { *(0xE00F_FFE8 as *const u32) };
    let jep_code_found = id2 & 0x8 != 0;
    let page = id4 & 0xF;
    let code = (id1 & 0xF0) >> 4 | (id2 & 0x7) << 4;
    if jep_code_found && page == 0 && code == 0x20 {
        info!("JEP106 code matches STMicro.");
    } else if !jep_code_found {
        info!("JEP106 code not found.");
    } else {
        info!("JEP106 page: {}, code: {} (decimal).", page, code,);
    }

    let dbgmcu_idcode = unsafe { *(0xE004_2000 as *const u32) };
    let dev_id = dbgmcu_idcode & 0xFFF;
    let rev_id = (dbgmcu_idcode & 0xFFFF0000) >> 16;
    info!(
        "DBGMCU_IDCODE: {:#010X}, dev_id: {:#06X}, rev_id: {:#06X}.",
        dbgmcu_idcode, dev_id, rev_id
    );

    let flash_size_kb = unsafe { *(0x1FFFF7E0 as *const u16) };
    info!("Flash size: {} KiB.", flash_size_kb);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(fast_config());

    print_chip_info();

    let mut led = Output::new(p.PB1, Level::Low, Speed::Low);

    loop {
        info!("Hi!");
        led.set_high();
        Timer::after(Duration::from_millis(1000)).await;
        led.set_low();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
