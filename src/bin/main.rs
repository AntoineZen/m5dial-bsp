#![no_std]
#![no_main]

// ESP32 Hardware abstraction
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output};
use esp_hal::main;
use esp_hal::spi::{
    master::{Config, Spi},
    Mode,
};
use esp_hal::time::RateExtU32;

// Generic hardware abstraction
use embedded_hal_bus::spi::ExclusiveDevice;

// Embedded graphics
use embedded_graphics::{
    mono_font::{ascii::FONT_7X13, MonoTextStyle},
    prelude::*,
    text::Text,
};
// Screen driver
use gc9a01::{prelude::*, Gc9a01, SPIDisplayInterface};

// heap-less string buffer
use heapless::String;

// Logging stuff
use core::fmt::Write;
use defmt::{error, info};
use {defmt_rtt as _, esp_backtrace as _};

extern crate alloc;

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut delay = Delay::new();

    // Split GPIO for peripherals blcok
    let sclk = peripherals.GPIO6;
    let mosi = peripherals.GPIO5;
    let cs = peripherals.GPIO7;
    let rs = peripherals.GPIO4;
    let reset = peripherals.GPIO8;
    let bl = peripherals.GPIO9;

    // Create SPI driver
    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(50.MHz())
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi);
    //.with_cs(cs);

    // Create output drivers from GPIOs
    let rs_out = Output::new(rs, Level::High);
    let cs_out = Output::new(cs, Level::High);
    let mut bl_out = Output::new(bl, Level::Low);
    let mut reset_out = Output::new(reset, Level::High);

    // Create SPI device and display interface adapter
    let display_dev = ExclusiveDevice::new_no_delay(spi, cs_out).unwrap();
    let display_iface = SPIDisplayInterface::new(display_dev, rs_out);

    // Create the display driver
    let mut display = Gc9a01::new(
        display_iface,
        DisplayResolution240x240,
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics();

    if display.reset(&mut reset_out, &mut delay).is_err() {
        error!("Display reset error");
        loop {}
    }
    // Init and clear display
    if display.init(&mut delay).is_err() {
        error!("Display Init error");
        loop {}
    }
    display.clear();
    display.fill(0x0);
    if display.flush().is_err() {
        error!("Display flush error");
    }

    // Show must go on !
    bl_out.set_high();

    let norm_style = MonoTextStyle::new(&FONT_7X13, RgbColor::RED);

    let mut buffer: String<16> = String::new();

    esp_alloc::heap_allocator!(72 * 1024);

    if write!(&mut buffer, "TEST").is_ok() {
        // Create a text at position (20, 30) and draw it using the previously defined style
        Text::new(&buffer, Point::new(100, 100), norm_style)
            .draw(&mut display)
            .unwrap();
        if display.flush().is_err() {
            error!("Display flush error");
        }
    }

    loop {
        info!("Hello world!");
        delay.delay_millis(500);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
