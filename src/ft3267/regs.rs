pub const FT3267_SLAVE_ADDRESS: u8 = 0x38;
pub const FT3267_ADDR: u8 = 0x38;

/** @brief FT3267 register map and function codes */

pub const FT3267_DEVICE_MODE: u8 = 0x00;
pub const FT3267_GESTURE_ID: u8 = 0x01;
pub const FT3267_TOUCH_POINTS: u8 = 0x02;

pub const FT3267_TOUCH1_EV_FLAG: u8 = 0x03;
pub const FT3267_TOUCH1_XH: u8 = 0x03;
pub const FT3267_TOUCH1_XL: u8 = 0x04;
pub const FT3267_TOUCH1_YH: u8 = 0x05;
pub const FT3267_TOUCH1_YL: u8 = 0x06;

pub const FT3267_TOUCH2_EV_FLAG: u8 = 0x09;
pub const FT3267_TOUCH2_XH: u8 = 0x09;
pub const FT3267_TOUCH2_XL: u8 = 0x0A;
pub const FT3267_TOUCH2_YH: u8 = 0x0B;
pub const FT3267_TOUCH2_YL: u8 = 0x0C;

pub const FT3267_TOUCH3_EV_FLAG: u8 = 0x0F;
pub const FT3267_TOUCH3_XH: u8 = 0x0F;
pub const FT3267_TOUCH3_XL: u8 = 0x10;
pub const FT3267_TOUCH3_YH: u8 = 0x11;
pub const FT3267_TOUCH3_YL: u8 = 0x12;

pub const FT3267_TOUCH4_EV_FLAG: u8 = 0x15;
pub const FT3267_TOUCH4_XH: u8 = 0x15;
pub const FT3267_TOUCH4_XL: u8 = 0x16;
pub const FT3267_TOUCH4_YH: u8 = 0x17;
pub const FT3267_TOUCH4_YL: u8 = 0x18;

pub const FT3267_TOUCH5_EV_FLAG: u8 = 0x1B;
pub const FT3267_TOUCH5_XH: u8 = 0x1B;
pub const FT3267_TOUCH5_XL: u8 = 0x1C;
pub const FT3267_TOUCH5_YH: u8 = 0x1D;
pub const FT3267_TOUCH5_YL: u8 = 0x1E;

pub const FT3267_ID_G_THGROUP: u8 = 0x80;
pub const FT3267_ID_G_THPEAK: u8 = 0x81;
pub const FT3267_ID_G_THCAL: u8 = 0x82;
pub const FT3267_ID_G_THWATER: u8 = 0x83;
pub const FT3267_ID_G_THTEMP: u8 = 0x84;
pub const FT3267_ID_G_THDIFF: u8 = 0x85;
pub const FT3267_ID_G_CTRL: u8 = 0x86;
pub const FT3267_ID_G_TIME_ENTER_MONITOR: u8 = 0x87;
pub const FT3267_ID_G_PERIODACTIVE: u8 = 0x88;
pub const FT3267_ID_G_PERIODMONITOR: u8 = 0x89;
pub const FT3267_ID_G_AUTO_CLB_MODE: u8 = 0xA0;
pub const FT3267_ID_G_LIB_VERSION_H: u8 = 0xA1;
pub const FT3267_ID_G_LIB_VERSION_L: u8 = 0xA2;
pub const FT3267_ID_G_CIPHER: u8 = 0xA3;
pub const FT3267_ID_G_MODE: u8 = 0xA4;
pub const FT3267_ID_G_PMODE: u8 = 0xA5;
pub const FT3267_ID_G_FIRMID: u8 = 0xA6;
pub const FT3267_ID_G_STATE: u8 = 0xA7;
pub const FT3267_ID_G_FT5201ID: u8 = 0xA8;
pub const FT3267_ID_G_ERR: u8 = 0xA9;
