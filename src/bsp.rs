//! M5Dial Board Support Package

// Use for debug
pub use defmt::error;

// Generic hardware abstraction
pub use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

// ESP32 Hardware abstraction
pub use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, OutputPin, Pull},
    i2c::master::{Config as I2cConfig, I2c as EspI2C},
    rmt::Rmt,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::Rate,
    Blocking,
};

// Screen driver
pub use gc9a01::{mode::BufferedGraphics, prelude::*, Gc9a01, SPIDisplayInterface};

// Touch screen driver (local)
pub use crate::ft3267::{Ft3267, TouchPoint};
pub use crate::rtc8563::{Rtc8563, RTC8563_DEFAULT_I2C_ADDRESS};

// Rotary encoder
pub use rotary_encoder_hal::{DefaultPhase, Rotary};

// Buzzer driver (local)
pub use crate::buzzer::Buzzer;

/// Define a type alias for the display
pub type M5DialDisplay = Gc9a01<
    SPIInterface<
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    DisplayResolution240x240,
    BufferedGraphics<DisplayResolution240x240>,
>;

pub type M5DialEncoder = Rotary<Input<'static>, Input<'static>, DefaultPhase>;

/// Holds the board peripherals
pub struct M5DialBsp {
    // Backlit command
    display_bl: Output<'static>,

    /// HOLD signal, must be set HIGH after startup to maintain power. Can be set LOW to power off.
    /// Note that this signal does not work on USB power
    hold: Output<'static>,

    /// WAKE signal. Get LOW on button push. Also temporary activate the +5V and +3.3V DC/DCs.
    wake: Input<'static>,

    /// State of wake last time it was pooled.
    last_wake_state: bool,
}

/// Get rottary encoder
#[macro_export]
macro_rules! get_encoder {
    ($peripherals:ident) => {{
        // Build the rotary encoder
        let pin_a = Input::new($peripherals.GPIO41, InputConfig::default());
        let pin_b = Input::new($peripherals.GPIO40, InputConfig::default());
        Rotary::new(pin_a, pin_b)
    }};
}

/// Get screen controller
#[macro_export]
macro_rules! get_screen {
    ($peripherals:ident) => {{
        let mut delay = Delay::new();
        // Create SPI driver
        let spi = Spi::new(
            $peripherals.SPI2,
            SpiConfig::default()
                .with_frequency(Rate::from_mhz(50))
                .with_mode(Mode::_0),
        )
        .unwrap()
        .with_sck($peripherals.GPIO6)
        .with_mosi($peripherals.GPIO5);

        // Create SPI device and display interface adapter
        let cs = Output::new($peripherals.GPIO7, Level::High, OutputConfig::default());
        let rs = Output::new($peripherals.GPIO4, Level::High, OutputConfig::default());
        let display_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
        let display_iface = SPIDisplayInterface::new(display_dev, rs);

        // Create the display driver, taking ownership of the above
        let mut display = Gc9a01::new(
            display_iface,
            DisplayResolution240x240,
            DisplayRotation::Rotate180,
        )
        .into_buffered_graphics();

        // Reset the display
        let mut display_reset =
            Output::new($peripherals.GPIO8, Level::High, OutputConfig::default());
        if display.reset(&mut display_reset, &mut delay).is_err() {
            panic!("Display reset error");
        }
        // Init and clear display
        if display.init(&mut delay).is_err() {
            panic!("Display Init error");
        }
        display.clear();
        display.fill(0x0);
        if display.flush().is_err() {
            error!("Display flush error");
        }
        display
    }};
}

/// Get the touch screen controller
#[macro_export]
macro_rules! get_touch {
    ($tp_i2c:ident) => {{
        let touch = Ft3267::new(0);
        touch.init(&mut $tp_i2c);
        touch
    }};
}

#[macro_export]
macro_rules! get_rtc {
    ($tp_i2c:ident) => {{
        let rtc = Rtc8563::new(RTC8563_DEFAULT_I2C_ADDRESS);
        rtc.init(&mut $tp_i2c);
        rtc
    }};
}

/// Get the the "tp" internal I2C Bus
#[macro_export]
macro_rules! get_internal_i2C {
    ($peripherals:ident) => {{
        EspI2C::new($peripherals.I2C0, I2cConfig::default())
            .expect("Failed to get I2C0")
            .with_sda($peripherals.GPIO11)
            .with_scl($peripherals.GPIO12)
    }};
}

/// Get the Port.A external I2C bus
#[macro_export]
macro_rules! get_port_a_i2c {
    ($peripherals:ident) => {{
        EspI2C::new($peripherals.I2C1, I2cConfig::default())
            .expect("Failed to get I2C1")
            .with_sda($peripherals.GPIO13)
            .with_scl($peripherals.GPIO15)
    }};
}

/// Get PORT.B input (white wire)
#[macro_export]
macro_rules! get_port_b_in {
    ($peripherals:ident, $input_config:expr) => {{
        Input::new($peripherals.GPIO1, $input_config)
    }};
}

/// Get PORT.B output (yellow wire)
#[macro_export]
macro_rules! get_port_b_out {
    ($peripherals:ident) => {{
        Output::new($peripherals.GPIO2, Level::Low)
    }};
}

/// Get the buzzer
#[macro_export]
macro_rules! get_buzzer {
    ($peripherals:ident) => {
        Buzzer::new(
            Rmt::new($peripherals.RMT, Rate::from_mhz(80)).unwrap(),
            $peripherals.GPIO3.into(),
        )
    };
}

/// Initialize board peripherals from ESP32 peripherals.
///
/// This function initialize the peripherals provided by this BSP
#[macro_export]
macro_rules! board_init {
    ($peripherals:ident) => {
        M5DialBsp::new(
            Output::new($peripherals.GPIO9, Level::Low, OutputConfig::default()),
            Output::new($peripherals.GPIO46, Level::High, OutputConfig::default()),
            Input::new($peripherals.GPIO42, InputConfig::default()),
        )
    };
}

impl M5DialBsp {
    pub fn new(bl: Output<'static>, hold: Output<'static>, wake: Input<'static>) -> M5DialBsp {
        let wake_state = wake.is_low();
        M5DialBsp {
            display_bl: bl,
            hold,
            wake,
            last_wake_state: wake_state,
        }
    }
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
    /// This has no effect when powered by USB. It only works when the board is powered using the green screw terminal (P5 on schematics).
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
}
