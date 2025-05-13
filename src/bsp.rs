//! M5Dial Board Support Package
use esp_hal::Blocking;
use gc9a01::{mode::BufferedGraphics, prelude::*, Gc9a01, SPIDisplayInterface};
// Screen driver

// ESP32 Hardware abstraction
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output, Pull};
use esp_hal::spi::{
    master::{Config as SpiConfig, Spi},
    Mode,
};
use esp_hal::time::RateExtU32;

use esp_hal::i2c::master::{Config as I2cConfig, I2c as EspI2C};

// Generic hardware abstraction
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

// Rotary encoder
use rotary_encoder_hal::{DefaultPhase, Rotary};

use defmt::error;

use crate::ft3267::{Ft3267, TouchPoint};

pub type M5DialDisplay = Gc9a01<
    SPIInterface<
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    DisplayResolution240x240,
    BufferedGraphics<DisplayResolution240x240>,
>;

/// Holds the board periferals
pub struct M5DialBsp {
    /// Display driver
    pub display: M5DialDisplay,

    // Touch screen controller
    touch: Ft3267,
    // Backlite command
    display_bl: Output<'static>,

    /// Rottary encoder
    pub encoder: Rotary<Input<'static>, Input<'static>, DefaultPhase>,

    /// HOLD signal, must be set HIGH after startup to maintain power. Can be set LOW to power off.
    /// Note that this signal does not work on USB power
    hold: Output<'static>,

    /// WAKE signal. Get LOW on button push. Also temporary activate the +5V and +3.3V DC/DCs.
    wake: Input<'static>,

    /// State of wake last time it was pooled.
    last_wake_state: bool,

    // Board I2C bus
    tp_i2c: EspI2C<'static, Blocking>,
}

/// Initialize board periferals from ESP32 peripherals.
///
/// This function initialize the peripherals provided by this BSP
pub fn init(peripherals: esp_hal::peripherals::Peripherals) -> M5DialBsp {
    let mut delay = Delay::new();

    // Create SPI driver
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
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

    let hold = Output::new(peripherals.GPIO46, Level::High);

    let wake = Input::new(peripherals.GPIO42, Pull::None);
    let wake_state = wake.is_low();

    let mut tp_i2c = EspI2C::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO11)
        .with_scl(peripherals.GPIO12);

    let touch = Ft3267::new(0);

    touch.init(&mut tp_i2c);

    M5DialBsp {
        display,
        touch,
        display_bl: bl,
        encoder,
        hold,
        wake,
        last_wake_state: wake_state,
        tp_i2c,
    }
}

impl M5DialBsp {
    /// Screen backlight control
    ///
    /// ## Arguments:
    ///    - **state**: Set backlight ON if true, OFF if false
    pub fn set_backlight(&mut self, state: bool) {
        if state {
            self.display_bl.set_high();
        } else {
            self.display_bl.set_low();
        }
    }

    /// Shutdown the board.
    ///
    /// This method shut-down the board by setting the pin G46 / signal HOLD low.
    /// This has no effect when powered by USB. It only works when the board is powered using the green screw terinal (P5 on schematics).
    ///
    /// **NOTE:** The pin signal is set high as start by the [init()](init) function.
    ///
    /// **NOTE:** Untested
    pub fn shutdown(&mut self) {
        self.hold.set_low();
    }

    /// Query current button state
    pub fn is_button_pushed(&mut self) -> bool {
        self.last_wake_state = self.wake.is_low();
        self.last_wake_state
    }

    /// Query if button state has changed since last call.
    ///
    /// ## Returns
    ///  - Some(current state) if button state has changed.
    ///  - None if button state has not changed.
    pub fn has_button_changed(&mut self) -> Option<bool> {
        let current_state = self.wake.is_low();
        if current_state != self.last_wake_state {
            self.last_wake_state = current_state;
            Some(current_state)
        } else {
            None
        }
    }

    /// Query if the touch screen is touched. If touch screen
    /// is un-touched, return None. Return Some() with detected
    /// finger count if touched (supports multi-touch).
    pub fn touch_count(&mut self) -> Option<u8> {
        let touch_count = self.touch.pool(&mut self.tp_i2c);

        if touch_count > 0 {
            Some(touch_count)
        } else {
            None
        }
    }

    /// Get Finger position for finger 'n'.
    pub fn touch_position(&mut self, n: u8) -> TouchPoint {
        self.touch.get_point(&mut self.tp_i2c, n)
    }
}
