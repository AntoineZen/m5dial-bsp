#![doc = include_str!("../README.md")]
#![no_std]

pub mod bsp;

pub mod backlight;
pub mod buzzer;
pub use ft3267;
pub use rtc8563;

pub use bsp::M5DialBsp;
