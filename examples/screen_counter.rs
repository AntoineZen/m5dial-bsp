#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
// ESP32 Hardware abstraction
use esp_hal::main;
use esp_hal::{clock::CpuClock, delay::Delay};

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
use defmt::{debug, error, info};
use {defmt_rtt as _, esp_backtrace as _};

use m5dial_bsp::bsp;

extern crate alloc;

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut board = bsp::init(peripherals);

    // Show must go on !
    board.set_backlight(true);

    const STYLE_LIST: [MonoTextStyle<Rgb565>; 7] = [
        MonoTextStyle::new(&THE_FONT, RgbColor::BLUE),
        MonoTextStyle::new(&THE_FONT, RgbColor::RED),
        MonoTextStyle::new(&THE_FONT, RgbColor::GREEN),
        MonoTextStyle::new(&THE_FONT, RgbColor::CYAN),
        MonoTextStyle::new(&THE_FONT, RgbColor::YELLOW),
        MonoTextStyle::new(&THE_FONT, RgbColor::MAGENTA),
        MonoTextStyle::new(&THE_FONT, RgbColor::WHITE),
    ];

    let mut style_index: usize = 0;

    let mut buffer: String<64> = String::new();

    esp_alloc::heap_allocator!(72 * 1024);

    info!("On screen counter demo running!");

    board.buzzer.set_frequency(261);
    let mut delay = Delay::new();
    delay.delay_ms(100);
    board.buzzer.buzz_off();

    let mut pos: i32 = 1;
    let mut need_redraw = true;
    loop {
        // Change Test color on button push
        if let Some(state) = board.has_button_changed() {
            if state == false {
                style_index = (style_index + 1) % STYLE_LIST.len();
                need_redraw = true;
            }
        }

        match board.encoder.update().unwrap() {
            Direction::Clockwise => {
                pos += 1;
                need_redraw = true;
            }
            Direction::CounterClockwise => {
                pos -= 1;
                need_redraw = true;
            }
            Direction::None => {}
        }

        if need_redraw {
            buffer.clear();
            if write!(&mut buffer, "Position {}", pos).is_ok() {
                board.display.clear();
                // Create a text at position (20, 30) and draw it using the previously defined style
                Text::with_alignment(
                    &buffer,
                    board.display.bounding_box().center(),
                    STYLE_LIST[style_index],
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

        //info!("Position {}", pos);
    }
}
