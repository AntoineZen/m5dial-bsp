// ESP32 Hardware abstraction
use esp_hal::{
    gpio::{DriveMode, Output},
    ledc::{
        channel::{self, Channel, ChannelIFace},
        timer::Timer,
        Ledc, LowSpeed,
    },
};
pub struct M5DialBackLight<'a> {
    // Backlit command
    //display_bl: Output<'static>,

    // LED driver
    //ledc: Ledc<'a>,

    // Timer
    //timer: Timer<'a, LowSpeed>,

    // Timer's Channel
    channel: Channel<'a, LowSpeed>,
    //ch_config : channel::config::Config<'static, LowSpeed> ,
}

impl<'a> M5DialBackLight<'a> {
    pub fn new(outpin: Output<'a>, ledc: &'a Ledc<'a>, timer: &'a Timer<'a, LowSpeed>) -> Self {
        let mut channel0 = ledc.channel(channel::Number::Channel0, outpin);
        channel0
            .configure(channel::config::Config {
                timer: timer,
                duty_pct: 50,
                drive_mode: DriveMode::PushPull,
            })
            .expect("Failed to configure channel");

        //channel0.set_duty(10).expect("Failed to set duty");

        M5DialBackLight {
            //ledc: ledc,
            channel: channel0,
            //timer: lstimer0,
        }
    }

    pub fn set_backlight(&mut self, percent: u8) {
        self.channel.set_duty(percent).expect("Failed to sed duty");
    }
}
