//! BM8563 RTC driver
//!
//! Mainly ported to Rust from this repo: https://github.com/tanakamasayuki/I2C_BM8563

use embedded_hal::i2c::I2c;

pub const RTC8563_DEFAULT_I2C_ADDRESS: u8 = 0x51;

const SECONDS_REG: u8 = 0x02;
//const MINI2CES_REG: u8 = 0x03;
//const HOURS_REG: u8 = 0x04;
const DAYS_REG: u8 = 0x05;
//const WEEKDAY_REG: u8 = 0x06;
//const MONI2CHS_REG: u8 = 0x07;
//const YEAR_REG: u8 = 0x08;

#[derive(Debug, Clone, Copy, Default)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Date {
    pub week_day: u8,
    pub month: u8,
    pub day: u8,
    pub year: i16,
}

pub struct Rtc8563 {
    address: u8,
}

pub type Bcd = u8;

fn bcd2byte(bdc: Bcd) -> u8 {
    let dec = (bdc & 0xF0) >> 4;
    let unit = bdc & 0x0F;

    (10 * dec) + unit
}

fn byte2bcd(value: u8) -> Bcd {
    let mut bcd_high: u8 = 0;
    let mut bcd_low: u8 = value;

    while bcd_low >= 10 {
        bcd_high += 1;
        bcd_low -= 10;
    }

    (bcd_high << 4) + bcd_low
}

impl Rtc8563 {
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    /// Initialize chip.
    ///
    /// I2Chis clear Control/status 1 and 2.
    pub fn init<I2C: I2c>(&self, bus: &mut I2C) -> Result<(), I2C::Error> {
        let buffer: [u8; 3] = [0, 0, 0];
        bus.write(self.address, &buffer)
    }

    pub fn get_time<I2C: I2c>(&self, bus: &mut I2C) -> Result<Time, I2C::Error> {
        let addr_buffer: [u8; 1] = [SECONDS_REG];
        let mut buffer: [u8; 3] = [0, 0, 0];

        bus.write_read(self.address, &addr_buffer, &mut buffer)?;

        Ok(Time {
            hours: bcd2byte(buffer[2] & 0x3F),
            minutes: bcd2byte(buffer[1] & 0x7F),
            seconds: bcd2byte(buffer[0] & 0x7F),
        })
    }

    pub fn set_time<I2C: I2c>(&self, bus: &mut I2C, time: &Time) -> Result<(), I2C::Error> {
        let buffer: [u8; 4] = [
            SECONDS_REG,
            byte2bcd(time.seconds),
            byte2bcd(time.minutes),
            byte2bcd(time.hours),
        ];

        bus.write(self.address, &buffer)
    }

    pub fn get_date<I2C: I2c>(&self, bus: &mut I2C) -> Result<Date, I2C::Error> {
        let addr_buffer: [u8; 1] = [DAYS_REG];
        let mut buffer: [u8; 4] = [0; 4];

        bus.write_read(self.address, &addr_buffer, &mut buffer)?;

        Ok(Date {
            day: bcd2byte(buffer[0] & 0x3f),
            week_day: bcd2byte(buffer[1] & 0x0f),
            month: bcd2byte(buffer[2] & 0x1f),
            year: if buffer[2] & 0x80 == 0x80 {
                1900 + bcd2byte(buffer[3]) as i16
            } else {
                2000 + bcd2byte(buffer[3]) as i16
            },
        })
    }

    pub fn set_date<I2C: I2c>(&self, bus: &mut I2C, date: &Date) -> Result<(), I2C::Error> {
        let mut buffer: [u8; 5] = [DAYS_REG, 0, 0, 0, 0];
        buffer[1] = byte2bcd(date.day) & 0x3f;
        buffer[2] = byte2bcd(date.week_day) & 0x0f;
        buffer[3] = byte2bcd(date.month) & 0x1f;

        if date.year < 2000 {
            buffer[3] |= 0x80;
        }
        buffer[4] = byte2bcd((date.year % 100) as u8);

        bus.write(self.address, &buffer)
    }

    #[allow(dead_code)]
    fn write_register<I2C: I2c>(
        &self,
        bus: &mut I2C,
        reg_addr: u8,
        reg_value: u8,
    ) -> Result<(), I2C::Error> {
        let buffer: [u8; 2] = [reg_addr, reg_value];

        bus.write(self.address, &buffer)
    }

    #[allow(dead_code)]
    fn read_register<I2C: I2c>(&self, bus: &mut I2C, reg_addr: u8) -> Result<u8, I2C::Error> {
        let addr_buffer: [u8; 1] = [reg_addr];
        let mut buffer: [u8; 1] = [0];

        bus.write_read(self.address, &addr_buffer, &mut buffer)?;

        Ok(buffer[0])
    }
}
