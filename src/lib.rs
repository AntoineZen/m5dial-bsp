#![doc = include_str!("../README.md")]
#![no_std]

pub mod bsp;

pub mod buzzer;
pub mod ft3267;
pub mod rtc8563;

pub use bsp::M5DialBsp;
