#![no_std]
#![no_main]

// ESP32 Hardware abstraction
use esp_hal::main;
use esp_hal::time::{Duration, Rate};
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

// Logging
use core::fmt::Write;
use defmt::{debug, error, info, Debug2Format};
use {defmt_rtt as _, esp_backtrace as _};

use m5dial_bsp::bsp::*;
extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let buzzer = m5dial_bsp::get_buzzer!(peripherals);
    let mut display = m5dial_bsp::get_screen!(peripherals);
    let mut encoder = m5dial_bsp::get_encoder!(peripherals);

    let mut board = m5dial_bsp::board_init!(peripherals);

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

    // String buffer
    let mut buffer: String<64> = String::new();

    // Memory allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);

    info!("On screen counter demo running!");

    // Emit a sound
    let mut tone_freq: u32 = 261;
    let mut buzzer = buzzer
        .tone(Rate::from_hz(tone_freq), Duration::from_millis(100))
        .expect("start tone failed");

    let mut pos: i32 = 0;
    let mut pos_delta: i32;
    let mut need_redraw = true;
    loop {
        // Change Test color on button push
        if let Some(state) = board.has_button_changed() {
            if !state {
                style_index = (style_index + 1) % STYLE_LIST.len();
                need_redraw = true;
            }
        }

        // Test if encoder has rotated
        pos_delta = match encoder.update().unwrap() {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
            Direction::None => 0,
        };

        if pos_delta < 0 {
            need_redraw = true;
            tone_freq = (tone_freq * 1000) / 1059;
        } else if pos_delta > 0 {
            need_redraw = true;
            tone_freq = (tone_freq * 1059) / 1000;
        }
        pos += pos_delta;

        debug!("tone_freq = {}", tone_freq);
        buzzer = match buzzer.tone(Rate::from_hz(tone_freq), Duration::from_millis(100)) {
            Ok(buzzer) => buzzer,
            Err((buzzer, e)) => {
                error!("{}", Debug2Format(&e));
                buzzer
            }
        };

        // Redraw screen if need refresh
        if need_redraw {
            buffer.clear();
            if write!(&mut buffer, "Position {}", pos).is_ok() {
                display.clear();
                // Create a text at position (20, 30) and draw it using the previously defined style
                Text::with_alignment(
                    &buffer,
                    display.bounding_box().center(),
                    STYLE_LIST[style_index],
                    embedded_graphics::text::Alignment::Center,
                )
                .draw(&mut display)
                .unwrap();

                if display.flush().is_err() {
                    error!("Display flush error");
                }
            } else {
                error!("Buffer overflow");
            }
            need_redraw = false;
        }
    }
}
