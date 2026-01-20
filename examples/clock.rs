#![no_std]
#![no_main]

// ESP32 Hardware abstraction
use esp_hal::main;
use esp_hal::time::Rate;
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
use defmt::{error, info};
use {defmt_rtt as _, esp_backtrace as _};

use m5dial_bsp::bsp::*;
extern crate alloc;
use m5dial_bsp::rtc8563::{Date, Time};

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Debug, PartialEq, Eq)]
enum ClockMode {
    EditHours,
    EditMinutes,
    EditDay,
    EditMonth,
    EditYear,
    Run,
}

struct Clock {
    mode: ClockMode,
    time: Time,
    date: Date,
    time_style: SevenSegmentStyle<Rgb565>,
    date_style: SevenSegmentStyle<Rgb565>,
}

fn modify_wrap<T>(value: &mut T, increment: i8, ceiling: T)
where
    T: Copy + PartialOrd + From<u8>,
    i16: From<T>,
    T: TryFrom<i16>,
{
    let new_val = i16::from(*value) + increment as i16;
    let ceiling_i16 = i16::from(ceiling);

    *value = if new_val >= ceiling_i16 {
        T::from(0u8)
    } else if new_val < 0 {
        T::try_from(ceiling_i16 - 1).unwrap_or(T::from(0u8))
    } else {
        T::try_from(new_val).unwrap_or(T::from(0u8))
    };
}

impl Clock {
    const COLOR: Rgb565 = Rgb565::CSS_ORANGE;

    fn new(time_height: u32, date_height: u32) -> Self {
        Self {
            mode: ClockMode::Run,
            time: Time::default(),
            date: Date::default(),
            // Define a new style.
            time_style: SevenSegmentStyleBuilder::new()
                .digit_size(Size::new(time_height / 2, time_height))
                .digit_spacing(time_height / 4)
                .segment_width(time_height / 8)
                .segment_color(Self::COLOR)
                .build(),
            date_style: SevenSegmentStyleBuilder::new()
                .digit_size(Size::new(date_height / 2, date_height))
                .digit_spacing(date_height / 4)
                .segment_width(date_height / 8)
                .segment_color(Self::COLOR)
                .build(),
        }
    }

    fn update_from(&mut self, rtc_time: &Time) {
        if self.mode == ClockMode::Run {
            self.time = *rtc_time;
        }
    }

    fn update_date_from(&mut self, rtc_date: &Date) {
        if self.mode == ClockMode::Run {
            self.date = *rtc_date;
        }
    }

    fn get_time(&self) -> &Time {
        &self.time
    }

    fn get_date(&self) -> &Date {
        &self.date
    }

    fn on_click(&mut self) {
        self.mode = match self.mode {
            ClockMode::Run => ClockMode::EditHours,
            ClockMode::EditHours => ClockMode::EditMinutes,
            ClockMode::EditMinutes => ClockMode::EditDay,
            ClockMode::EditDay => ClockMode::EditMonth,
            ClockMode::EditMonth => ClockMode::EditYear,
            ClockMode::EditYear => ClockMode::Run,
        };
    }

    fn edit(&mut self, delta: i8) {
        match self.mode {
            ClockMode::Run => {}
            ClockMode::EditHours => {
                modify_wrap(&mut self.time.hours, delta, 24);
            }
            ClockMode::EditMinutes => {
                modify_wrap(&mut self.time.minutes, delta, 60);
            }
            ClockMode::EditDay => {
                modify_wrap(&mut self.date.day, delta, 31);
            }
            ClockMode::EditMonth => {
                modify_wrap(&mut self.date.month, delta, 12);
            }
            ClockMode::EditYear => {
                modify_wrap(&mut self.date.year, delta, 3000);
            }
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

        // Render time string
        write!(
            &mut buffer,
            "{:02}:{:02}:{:02}",
            self.time.hours, self.time.minutes, self.time.seconds
        )
        .unwrap_or_else(|_| {
            error!("Error writing buffer!");
        });

        // Render time as 7 segments digits
        let _ = Text::with_alignment(
            &buffer,
            center_point,
            self.time_style,
            embedded_graphics::text::Alignment::Center,
        )
        .draw(display)
        .unwrap_or_else(|_| {
            error!("Error rendering!");
            Point::default()
        });

        // Render date
        let date_offset = Point::new(0, 2 * self.date_style.digit_size.height as i32);
        buffer.clear();
        write!(
            &mut buffer,
            "{:02}-{:02}-{:04}",
            self.date.day, self.date.month, self.date.year
        )
        .unwrap_or_else(|_| {
            error!("Error writing buffer!");
        });

        let date_text = Text::with_alignment(
            &buffer,
            center_point + date_offset,
            self.date_style,
            embedded_graphics::text::Alignment::Center,
        );
        date_text.draw(display).unwrap_or_else(|_| {
            error!("Error rendering!");
            Point::default()
        });

        // Underline selected field when in Edit modes
        if ClockMode::Run != self.mode {
            let (line_length, line_width) = match self.mode {
                ClockMode::EditHours | ClockMode::EditMinutes => (
                    (self.time_style.digit_size.width * 2 + self.time_style.digit_spacing) as i32,
                    5,
                ),
                ClockMode::EditYear => (
                    (self.date_style.digit_size.width * 4 + self.date_style.digit_spacing * 3)
                        as i32,
                    2,
                ),
                _ => (
                    (self.date_style.digit_size.width * 2 + self.date_style.digit_spacing) as i32,
                    2,
                ),
            };

            let y_offset = match self.mode {
                ClockMode::EditHours | ClockMode::EditMinutes => center_point.y + 5,
                _ => center_point.y + 2 * self.date_style.digit_size.height as i32 + 3,
            };

            let p1 = match self.mode {
                ClockMode::EditHours => Point::new(20, y_offset),
                ClockMode::EditMinutes => Point::new(center_point.x - 25, y_offset),
                ClockMode::EditDay => Point::new(date_text.bounding_box().top_left.x, y_offset),
                ClockMode::EditMonth => Point::new(
                    date_text.bounding_box().top_left.x
                        + 3 * (self.date_style.digit_spacing + self.date_style.digit_size.width)
                            as i32,
                    y_offset,
                ),
                ClockMode::EditYear => Point::new(
                    date_text.bounding_box().top_left.x
                        + 6 * (self.date_style.digit_spacing + self.date_style.digit_size.width)
                            as i32,
                    y_offset,
                ),
                _ => Point::new(0, 0),
            };
            let p2 = p1 + Point::new(line_length, 0);
            let _ = Line::new(p1, p2)
                .into_styled(PrimitiveStyle::with_stroke(Self::COLOR, line_width))
                .draw(display);
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
    let rtc = m5dial_bsp::get_rtc!(tp_i2c);

    // Show must go on
    board.set_backlight(true);

    // Memory allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let d = Delay::new();

    let mut clock = Clock::new(40, 16);
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
            let now = rtc.get_time(&mut tp_i2c);
            clock.update_from(&now);

            let today = rtc.get_date(&mut tp_i2c);
            clock.update_date_from(&today);
        } else {
            // Get the on-screen time & Date and set the RTC
            rtc.set_time(&mut tp_i2c, clock.get_time());
            rtc.set_date(&mut tp_i2c, clock.get_date());
        }

        // Update display
        display.clear();
        clock.draw(&mut display);
        display.flush().expect("Flush error!");

        // Sleep for a tenth of second
        d.delay_micros(100);
    }
}
