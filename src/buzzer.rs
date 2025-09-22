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

pub struct Buzzer {
    ledc: Ledc<'static>,
    pin: Output<'static>,
}

impl Buzzer {
    pub fn new(mut ledc: Ledc<'static>, pin: Output<'static>) -> Self {
        // Initialize and create handle for LEDC peripheral
        //let mut ledc = Ledc::new(led_perif);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        Buzzer { ledc, pin }
    }

    fn configure(&mut self, freq: u32, duty: u8) {
        let mut timer = self.ledc.timer::<LowSpeed>(timer::Number::Timer0);
        let mut channel = self.ledc.channel(channel::Number::Channel0, &mut self.pin);

        timer
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty5Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: freq.Hz(),
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

    pub fn set_frequency(&mut self, freq: u32) {
        self.configure(freq, 50);
    }

    pub fn buzz_off(&mut self) {
        self.configure(1000, 0);
    }
}
