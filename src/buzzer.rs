// ESP32 Hardware abstraction
use esp_hal::{
    gpio::Output,
    ledc::{
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    time::RateExtU32,
};

use fugit::HertzU32;

pub struct Buzzer {
    ledc: Ledc<'static>,
    pin: Output<'static>,
}

impl Buzzer {
    /// Build a new buzzer driver.
    ///
    /// This required a LEDC periferal driver `ledc` and the output `pin`
    pub fn new(mut ledc: Ledc<'static>, pin: Output<'static>) -> Self {
        // Initialize and create handle for LEDC peripheral
        //let mut ledc = Ledc::new(led_perif);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        Buzzer { ledc, pin }
    }

    // Private function to do the actual PWM frequency and dutty settings.
    fn configure(&mut self, freq: HertzU32, duty: u8) {
        let mut timer = self.ledc.timer::<LowSpeed>(timer::Number::Timer0);
        let mut channel = self.ledc.channel(channel::Number::Channel0, &mut self.pin);

        timer
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty5Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: freq,
            })
            .unwrap();

        channel
            .configure(channel::config::Config {
                timer: &timer,
                duty_pct: duty,
                pin_config: channel::config::PinConfig::PushPull,
            })
            .unwrap();
    }

    /// Set buzzing frequency, in Hz
    pub fn set_frequency(&mut self, freq: HertzU32) {
        self.configure(freq, 50);
    }

    /// Turn the buzzer OFF
    pub fn off(&mut self) {
        self.configure(1000.Hz(), 0);
    }
}
