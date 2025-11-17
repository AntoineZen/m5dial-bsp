#![no_std]
#![no_main]
///! This example draws a cycle on the screen that you can move with your finger.
///! The cycle can change color on clicks. This demonstrate the Touch driver
///! functionality
// ESP32 Hardware abstraction
use esp_hal::clock::CpuClock;
use esp_hal::main;

// Embedded graphics
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder},
};

// Logging stuff
use defmt::{error, info};
use {defmt_rtt as _, esp_backtrace as _};

use m5dial_bsp::bsp::*;

extern crate alloc;

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut touch = m5dial_bsp::get_touch!(peripherals);
    let mut display = m5dial_bsp::get_screen!(peripherals);

    // Initialize the BSP
    let mut board = m5dial_bsp::board_init!(peripherals);

    // Show must go on !
    board.set_backlight(true);

    const COLOR_LIST: [Rgb565; 7] = [
        RgbColor::BLUE,
        RgbColor::RED,
        RgbColor::GREEN,
        RgbColor::CYAN,
        RgbColor::YELLOW,
        RgbColor::MAGENTA,
        RgbColor::WHITE,
    ];

    let mut style_index: usize = 0;

    esp_alloc::heap_allocator!(size: 72 * 1024);

    info!("On screen counter demo running!");
    let (w, h) = display.bounds();
    let mut point = Point::new((w / 2).into(), (h / 2).into());

    let mut need_redraw = true;
    loop {
        // Change Test color on button push
        if let Some(state) = board.has_button_changed() {
            if state == false {
                style_index = (style_index + 1) % COLOR_LIST.len();
                need_redraw = true;
            }
        }

        if let Some(_) = touch.count() {
            //info!("Touch: {}", touch_count);
            let p = touch.position(0);
            info!("Pos: x={} y={}", p.x, p.y);
            point.x = p.x.into();
            point.y = p.y.into();
            need_redraw = true;
        }

        if need_redraw {
            display.clear();
            // Create the cycle with the given color
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(COLOR_LIST[style_index])
                .stroke_width(3)
                .fill_color(Rgb565::BLACK)
                .build();

            Circle::with_center(point, 50)
                .into_styled(style)
                .draw(&mut display)
                .unwrap();

            if display.flush().is_err() {
                error!("Display flush error");
            }

            need_redraw = false;
        }
    }
}
