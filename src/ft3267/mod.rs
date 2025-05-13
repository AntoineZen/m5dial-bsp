//! RT3267 Touch driver
//!
//! Ported from https://github.com/mmMicky/TouchLib/blob/main/src/ModulesFT3267.tpp

// Register definitins
#[allow(dead_code)]
mod regs;

use embedded_hal::i2c::I2c;

pub enum Ft3265Gesture {
    Ft3267GestureNone = 0x00,
    Ft3267GestureMoveUp = 0x10,
    Ft3267GestureMoveLeft = 0x14,
    Ft3267GestureMoveDown = 0x18,
    Ft3267GestureMoveRight = 0x1c,
    Ft3267GestureZomeIn = 0x48,
    Ft3267GestureZomeOut = 0x49,
}

#[derive(Debug)]
pub struct Ft3267 {
    address: u8,
    rotation: u8,
}

#[derive(Debug)]
pub struct TouchPoint {
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

impl Ft3267 {
    fn write_register<T: I2c>(&self, bus: &mut T, reg_addr: u8, reg_value: u8) {
        let buffer: [u8; 2] = [reg_addr, reg_value];

        // TODO : Error handling
        let _ = bus.write(self.address, &buffer);
    }

    fn read_register<T: I2c>(&self, bus: &mut T, reg_addr: u8, buffer: &mut [u8]) {
        let addr_buffer: [u8; 1] = [reg_addr];

        // TODO : Error handling
        let _ = bus.write_read(self.address, &addr_buffer, buffer);
    }

    pub fn new(rotation: u8) -> Self {
        Ft3267 {
            address: regs::FT3267_ADDR,
            rotation: rotation,
        }
    }

    pub fn init<T: I2c>(&self, bus: &mut T) -> &Self {
        self.write_register(bus, regs::FT3267_ID_G_THGROUP, 70);

        // valid touching peak detect threshold
        self.write_register(bus, regs::FT3267_ID_G_THPEAK, 60);

        // Touch focus threshold
        self.write_register(bus, regs::FT3267_ID_G_THCAL, 16);

        // threshold when there is surface water
        self.write_register(bus, regs::FT3267_ID_G_THWATER, 60);

        // threshold of temperature compensation
        self.write_register(bus, regs::FT3267_ID_G_THTEMP, 10);

        // Touch difference threshold
        self.write_register(bus, regs::FT3267_ID_G_THDIFF, 20);

        // Delay to enter 'Monitor' status (s)
        self.write_register(bus, regs::FT3267_ID_G_TIME_ENTER_MONITOR, 2);

        // Period of 'Active' status (ms)
        self.write_register(bus, regs::FT3267_ID_G_PERIODACTIVE, 12);

        // Timer to enter 'idle' when in 'Monitor' (ms)
        self.write_register(bus, regs::FT3267_ID_G_PERIODMONITOR, 40);

        self
    }

    pub fn pool<T: I2c>(&self, bus: &mut T) -> u8 {
        let mut raw_data: [u8; 1] = [0];
        self.read_register(bus, regs::FT3267_TOUCH_POINTS, &mut raw_data);
        raw_data[0] & 0x0f
    }

    pub fn get_point<T: I2c>(&self, bus: &mut T, n: u8) -> TouchPoint {
        let mut buf: [u8; 4] = [0; 4];

        match n {
            0 => {
                self.read_register(bus, regs::FT3267_TOUCH1_XH, &mut buf);
            }
            1 => {
                self.read_register(bus, regs::FT3267_TOUCH2_XH, &mut buf);
            }
            2 => {
                self.read_register(bus, regs::FT3267_TOUCH3_XH, &mut buf);
            }
            3 => {
                self.read_register(bus, regs::FT3267_TOUCH4_XH, &mut buf);
            }
            _ => {}
        }

        let x = (((buf[0] & 0x0f) as u16) << 8) + buf[1] as u16;
        let y = (((buf[2] & 0x0f) as u16) << 8) + buf[3] as u16;

        if self.rotation == 0 {
            TouchPoint { id: n, x, y }
        } else {
            TouchPoint { id: n, x: y, y: x }
        }
    }
}
