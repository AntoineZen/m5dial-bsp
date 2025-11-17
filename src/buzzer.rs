// ESP32 Hardware abstraction
use esp_hal::{
    gpio::{AnyPin, DriveMode},
    ledc::{
        channel::{self, Channel, ChannelIFace},
        timer::{self, Timer, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    time::Rate,
};

/// Buzzer driver using LEDC peripheral to generate the PWM signal.
pub struct Buzzer<'t> {
    timer: Timer<'t, LowSpeed>,
    channel: Channel<'t, LowSpeed>,
}

impl<'t> Buzzer<'t> {
    /// Build a new buzzer driver.
    ///
    /// This required a LEDC peripheral driver `ledc` and the output `pin`
    pub fn new(mut ledc: Ledc<'t>, pin: AnyPin<'t>) -> Self {
        // Initialize and create handle for LEDC peripheral
        //let mut ledc = Ledc::new(led_perif);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        let channel = ledc.channel(channel::Number::Channel0, pin);
        let timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
        Buzzer { timer, channel }
    }

    // Private function to do the actual PWM frequency and duty settings.
    fn configure(&'t mut self, freq: Rate, duty: u8) {
        //let mut timer = self.ledc.timer::<LowSpeed>(timer::Number::Timer0);
        //let mut channel = self.ledc.channel(channel::Number::Channel0, self.pin);

        self.timer
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty5Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: freq,
            })
            .unwrap();

        self.channel
            .configure(channel::config::Config {
                timer: &self.timer,
                duty_pct: duty,
                drive_mode: DriveMode::PushPull,
            })
            .unwrap();
    }

    /// Set buzzing frequency, in Hz
    pub fn set_frequency(&'t mut self, freq: Rate) {
        self.configure(freq, 50);
    }

    /// Turn the buzzer OFF
    pub fn off(&'t mut self) {
        self.configure(Rate::from_hz(1000), 0);
    }
}
