#![no_std]
#![no_main]

use embedded_hal::delay::DelayNs;
// ESP32 Hardware abstraction
use esp_hal::main;
use esp_hal::time::RateExtU32;
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
use defmt::{debug, error, info};
use {defmt_rtt as _, esp_backtrace as _};

use esp_hal::gpio::{Event, Input, Io, Level, Output, Pull};
use esp_hal::interrupt::InterruptConfigurable;
use esp_hal::{handler, ram};
use m5dial_bsp::bsp::*;

extern crate alloc;

use core::cell::RefCell;

use critical_section::Mutex;
use rotary_encoder_hal::{DefaultPhase, Rotary};

static ENCODER: Mutex<RefCell<Option<Rotary<Input<'static>, Input<'static>, DefaultPhase>>>> =
    Mutex::new(RefCell::new(None));

static POSITION: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));

#[handler]
#[ram]
fn encoder_irq() {
    //

    let rot = critical_section::with(|cs| {
        ENCODER
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .update()
            .unwrap()
    });
    match rot {
        Direction::Clockwise => critical_section::with(|cs| *POSITION.borrow_ref_mut(cs) += 1),
        Direction::CounterClockwise => {
            critical_section::with(|cs| *POSITION.borrow_ref_mut(cs) -= 1);
        }
        Direction::None => {}
    }
}

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut buzzer = m5dial_bsp::get_buzzer!(peripherals);
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
    esp_alloc::heap_allocator!(72 * 1024);

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(encoder_irq);

    info!("On screen counter demo running!");

    // Emit a sound
    buzzer.set_frequency(261.Hz());
    let mut delay = Delay::new();
    delay.delay_ms(100);
    buzzer.off();

    critical_section::with(|cs| {
        encoder.pin_a().listen(Event::AnyEdge);
        encoder.pin_b().listen(Event::AnyEdge);
        ENCODER.borrow_ref_mut(cs).replace(encoder)
    });

    let mut old_pos: i32 = 1;
    let mut current_pos: i32 = 1;
    let mut need_redraw = true;
    loop {
        // Change Test color on button push
        if let Some(state) = board.has_button_changed() {
            if state == false {
                style_index = (style_index + 1) % STYLE_LIST.len();
                need_redraw = true;
            }
        }

        current_pos = critical_section::with(|cs| *POSITION.borrow_ref_mut(cs));

        // Test if encoder has rotated
        if current_pos != old_pos {
            need_redraw = true;
            old_pos = current_pos;
        }

        // Redraw screen if need refresh
        if need_redraw {
            buffer.clear();
            if write!(&mut buffer, "Position {}", current_pos).is_ok() {
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

        //info!("Position {}", pos);
    }
}
