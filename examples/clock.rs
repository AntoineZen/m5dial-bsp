#![no_std]
#![no_main]

use alloc::rc;
use embedded_hal::delay::DelayNs;
// ESP32 Hardware abstraction
use esp_hal::main;
use esp_hal::time::{Duration, Rate};
use esp_hal::{clock::CpuClock, delay::Delay};

// Embedded graphics
use eg_seven_segment::{SevenSegmentStyle, SevenSegmentStyleBuilder};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
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
use m5dial_bsp::rtc8563::Time;

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Debug, PartialEq, Eq)]
enum ClockMode {
    EditHours,
    EditMinutes,
    Run,
}

struct Clock {
    mode: ClockMode,
    time: Time,
    style: SevenSegmentStyle<Rgb565>,
}

impl Clock {
    fn new() -> Self {
        Self {
            mode: ClockMode::Run,
            time: Time {
                hours: 0,
                minutes: 0,
                seconds: 0,
            },
            // Define a new style.
            style: SevenSegmentStyleBuilder::new()
                .digit_size(Size::new(20, 40)) // digits are 10x20 pixels
                .digit_spacing(10) // 5px spacing between digits
                .segment_width(5) // 5px wide segments
                .segment_color(Rgb565::GREEN) // active segments are green
                .build(),
        }
    }

    fn update_from(&mut self, rtc_time: &Time) {
        if self.mode == ClockMode::Run {
            self.time = *rtc_time;
        }
    }

    fn get_time(&self) -> &Time {
        &self.time
    }

    fn on_click(&mut self) {
        self.mode = match self.mode {
            ClockMode::Run => ClockMode::EditHours,
            ClockMode::EditHours => ClockMode::EditMinutes,
            ClockMode::EditMinutes => ClockMode::Run,
        };
    }

    fn edit(&mut self, detla: i8) {
        match self.mode {
            ClockMode::Run => {}
            ClockMode::EditHours => self.time.hours = (self.time.hours as i8 + detla) as u8,
            ClockMode::EditMinutes => self.time.minutes = (self.time.minutes as i8 + detla) as u8,
        }
    }

    fn up(&mut self) {
        self.edit(1);
    }

    fn down(&mut self) {
        self.edit(-1);
    }

    fn is_running(&self) -> bool {
        self.mode == ClockMode::Run
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&self, display: &mut D) {
        let mut buffer: String<64> = String::new();

        let center_point = display.bounding_box().center() + Point::new(0, 20);

        write!(
            &mut buffer,
            "{:02}:{:02}:{:02}",
            self.time.hours, self.time.minutes, self.time.seconds
        )
        .unwrap_or_else(|_| {
            error!("Error writing buffer!");
        });

        let _ = Text::with_alignment(
            &buffer,
            center_point,
            self.style,
            embedded_graphics::text::Alignment::Center,
        )
        .draw(display)
        .unwrap_or_else(|_| {
            error!("Error rendering!");
            Point::default()
        });

        match self.mode {
            ClockMode::EditHours => {
                let p1 = Point::new(20, center_point.y + 5);
                let p2 = p1 + Point::new(50, 0);

                let _ = Line::new(p1, p2)
                    .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 5))
                    .draw(display);
            }
            ClockMode::EditMinutes => {
                let p1 = center_point - Point::new(25, -5);
                let p2 = center_point + Point::new(25, 5);

                let _ = Line::new(p1, p2)
                    .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 5))
                    .draw(display);
            }
            _ => {}
        }
    }
}

#[main]
fn main() -> ! {
    // Get periferals from the hal.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut display = m5dial_bsp::get_screen!(peripherals);
    let mut encoder = m5dial_bsp::get_encoder!(peripherals);

    let mut board = m5dial_bsp::board_init!(peripherals);
    let mut tp_i2c = m5dial_bsp::get_internal_i2C!(peripherals);
    let mut rtc = m5dial_bsp::get_rtc!(tp_i2c);

    // Show must go on
    board.set_backlight(true);

    // Memory allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let d = Delay::new();

    let mut clock = Clock::new();
    info!("Clock demo running!");

    loop {
        // Map encoder action to clock
        match encoder.update().unwrap() {
            Direction::Clockwise => clock.up(),
            Direction::CounterClockwise => clock.down(),
            Direction::None => {}
        };
        // Map button action to clock application
        if let Some(state) = board.has_button_changed() {
            if !state {
                clock.on_click();
            }
        }

        if clock.is_running() {
            // Get the RTC time and update the Clock APP
            let t = rtc.get_time(&mut tp_i2c);
            clock.update_from(&t);
        } else {
            // Get the on-screen time and set the RTC
            let t = clock.get_time();
            rtc.set_time(&mut tp_i2c, t);
        }

        // Update display
        display.clear();
        clock.draw(&mut display);
        display.flush();

        // Sleep for a tenth of second
        d.delay_micros(100);
    }
}
