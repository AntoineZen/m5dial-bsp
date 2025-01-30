#![no_std]
#![no_main]

// ESP32 Hardware abstraction
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output, Pull};
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

// Rotary encoder
use rotary_encoder_hal::{Direction, Rotary};

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

    let pin_a = Input::new(peripherals.GPIO41, Pull::None);
    let pin_b = Input::new(peripherals.GPIO40, Pull::None);

    let mut encoder = Rotary::new(pin_a, pin_b);

    // Create SPI driver
    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(50.MHz())
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO5);
    //.with_cs(cs);

    // Create output drivers from GPIOs
    let rs = Output::new(peripherals.GPIO4, Level::High);
    let cs = Output::new(peripherals.GPIO7, Level::High);
    let mut bl = Output::new(peripherals.GPIO9, Level::Low);
    let mut display_reset = Output::new(peripherals.GPIO8, Level::High);

    // Create SPI device and display interface adapter
    let display_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let display_iface = SPIDisplayInterface::new(display_dev, rs);

    // Create the display driver
    let mut display = Gc9a01::new(
        display_iface,
        DisplayResolution240x240,
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics();

    if display.reset(&mut display_reset, &mut delay).is_err() {
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
    bl.set_high();

    let norm_style = MonoTextStyle::new(&FONT_7X13, RgbColor::RED);

    let mut buffer: String<64> = String::new();

    esp_alloc::heap_allocator!(72 * 1024);

    let mut pos: u32 = 50;
    loop {
        match encoder.update().unwrap() {
            Direction::Clockwise => {
                pos += 1;
                info!("UP");
            }
            Direction::CounterClockwise => {
                pos -= 1;
                info!("DOWN");
            }
            Direction::None => {
                info!("NOP");
            }
        }

        buffer.clear();
        if write!(&mut buffer, "Position {}", pos).is_ok() {
            // Create a text at position (20, 30) and draw it using the previously defined style
            Text::new(&buffer, Point::new(100, 100), norm_style)
                .draw(&mut display)
                .unwrap();
            if display.flush().is_err() {
                error!("Display flush error");
            }
        } else {
            error!("Buffer overflow");
        }

        info!("Position {}", pos);

        //delay.delay_millis(50);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
