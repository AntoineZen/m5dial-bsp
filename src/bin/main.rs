#![no_std]
#![no_main]

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output};
use esp_hal::main;
use esp_hal::spi::{
    master::{Config, Spi},
    Mode,
};
use esp_hal::time::RateExtU32;

use embedded_hal_bus::spi::ExclusiveDevice;

use embedded_hal::delay::DelayNs;

// Embedded graphics
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{ascii::FONT_7X13, MonoTextStyle},
    prelude::*,
    text::Text,
};

use core::fmt::Write;
use heapless::String;

// Screen driver
use gc9a01::GC9A01;

use {defmt_rtt as _, esp_backtrace as _};

extern crate alloc;

#[main]
fn main() -> ! {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut delay = Delay::new();

    let sclk = peripherals.GPIO6;
    let mosi = peripherals.GPIO5;
    let cs = peripherals.GPIO7;
    let rs = peripherals.GPIO4;
    let reset = peripherals.GPIO8;
    let bl = peripherals.GPIO9;

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(10.MHz())
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi);
    //.with_cs(cs);

    let rs_out = Output::new(rs, Level::High);
    let cs_out = Output::new(cs, Level::High);
    let mut bl_out = Output::new(bl, Level::Low);

    let mut reset_out = Output::new(reset, Level::High);

    let display_dev = ExclusiveDevice::new_no_delay(spi, cs_out).unwrap();
    let mut display = GC9A01::new(display_dev, rs_out);

    // Reset display controller
    delay.delay_ms(100);
    reset_out.set_low();
    delay.delay_ms(100);
    reset_out.set_high();
    delay.delay_ms(100);

    bl_out.set_high();

    display.setup();
    delay.delay_ms(100);

    display.clear(RgbColor::BLACK).unwrap();

    let norm_style = MonoTextStyle::new(&FONT_7X13, RgbColor::CYAN);

    let mut buffer: String<16> = String::new();

    esp_alloc::heap_allocator!(72 * 1024);

    if write!(&mut buffer, "TEST").is_ok() {
        // Create a text at position (20, 30) and draw it using the previously defined style
        Text::new(&buffer, Point::new(100, 100), norm_style)
            .draw(&mut display)
            .unwrap();
    }

    loop {
        info!("Hello world!");
        delay.delay_millis(500);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
