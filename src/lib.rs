#![doc = include_str!("../README.md")]
#![no_std]

pub mod bsp;

pub mod bm8563;
pub mod buzzer;
pub mod ft3267;

pub use bsp::M5DialBsp;
