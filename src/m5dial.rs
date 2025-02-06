use esp_hal::Blocking;
use gc9a01::{mode::BufferedGraphics, prelude::*, Gc9a01, SPIDisplayInterface};
// Screen driver

// ESP32 Hardware abstraction
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::spi::{
    master::{Config, Spi},
    Mode,
};
use esp_hal::time::RateExtU32;

// Generic hardware abstraction
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

// Rotary encoder
use rotary_encoder_hal::{DefaultPhase, Rotary};

use defmt::error;

pub struct M5Dial {
    // Display driver
    pub display: Gc9a01<
        SPIInterface<
            ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
            Output<'static>,
        >,
        DisplayResolution240x240,
        BufferedGraphics<DisplayResolution240x240>,
    >,
    // Backlite command
    pub display_bl: Output<'static>,

    // Rottary encoder
    pub encoder: Rotary<Input<'static>, Input<'static>, DefaultPhase>,
}

pub fn init(peripherals: esp_hal::peripherals::Peripherals) -> M5Dial {
    let mut delay = Delay::new();

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

    // Create SPI device and display interface adapter
    let cs = Output::new(peripherals.GPIO7, Level::High);
    let rs = Output::new(peripherals.GPIO4, Level::High);
    let display_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let display_iface = SPIDisplayInterface::new(display_dev, rs);

    let mut display = Gc9a01::new(
        display_iface,
        DisplayResolution240x240,
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics();

    let mut display_reset = Output::new(peripherals.GPIO8, Level::High);
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

    let pin_a = Input::new(peripherals.GPIO41, Pull::None);
    let pin_b = Input::new(peripherals.GPIO40, Pull::None);

    let encoder = Rotary::new(pin_a, pin_b);

    let bl = Output::new(peripherals.GPIO9, Level::Low);

    M5Dial {
        display: display,
        display_bl: bl,
        encoder: encoder,
    }
}
