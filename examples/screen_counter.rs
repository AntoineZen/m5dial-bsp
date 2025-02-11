#![no_std]
#![no_main]

// ESP32 Hardware abstraction
use esp_hal::clock::CpuClock;
use esp_hal::main;

// Embedded graphics
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20 as THE_FONT, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
// Screen driver

// Rotary encoder
use rotary_encoder_hal::Direction;

// heap-less string buffer

use heapless::String;

// Logging stuff
use core::fmt::Write;
use defmt::{error, info};
use {defmt_rtt as _, esp_backtrace as _};

use m5dial_hal::m5dial;

extern crate alloc;

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut board = m5dial::init(peripherals);

    // Show must go on !
    board.set_backlight(true);

    const NORM_STYLE: MonoTextStyle<Rgb565> = MonoTextStyle::new(&THE_FONT, RgbColor::BLUE);

    let mut buffer: String<64> = String::new();

    esp_alloc::heap_allocator!(72 * 1024);

    let mut pos: i32 = 1;
    let mut need_redraw = true;
    loop {
        match board.encoder.update().unwrap() {
            Direction::Clockwise => {
                pos += 1;
                info!("UP");
                need_redraw = true;
            }
            Direction::CounterClockwise => {
                pos -= 1;
                info!("DOWN");
                need_redraw = true;
            }
            Direction::None => {
                info!("NOP");
            }
        }

        if need_redraw {
            buffer.clear();
            if write!(&mut buffer, "Position {}", pos).is_ok() {
                board.display.clear();
                // Create a text at position (20, 30) and draw it using the previously defined style
                Text::with_alignment(
                    &buffer,
                    board.display.bounding_box().center(),
                    NORM_STYLE,
                    embedded_graphics::text::Alignment::Center,
                )
                .draw(&mut board.display)
                .unwrap();

                if board.display.flush().is_err() {
                    error!("Display flush error");
                }
            } else {
                error!("Buffer overflow");
            }
            need_redraw = false;
        }

        info!("Position {}", pos);

        //delay.delay_millis(50);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
