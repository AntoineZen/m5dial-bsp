#![doc = include_str!("../README.md")]
#![no_std]

pub mod bsp;

pub use bsp::init;
pub use bsp::M5DialBsp;
