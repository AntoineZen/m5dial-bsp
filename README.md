# M5 Dial Board Support Package (BSP)

This crates is a Board support package for the [M5 Dial](https://shop.m5stack.com/products/m5stack-dial-esp32-s3-smart-rotary-knob-w-1-28-round-touch-screen).

### [API Documentation](https://antoinezen.github.io/m5dial-bsp)

Feature list/roadmap:

 - [X] Screen driver (GC9A01)
 - [X] Rotary encoder
 - [X] Button
 - [X] Power management (Shutdown)
 - [X] Touch-screen (FT3267)
 - [ ] Buzzer
 - [ ] Real-time clock (BM8563)
 - [X] Port A (I2C)
 - [X] Port B
 - [X] Upgrade to be compatible to esp-hal v1.0.0

## How to use

First generate a base project using [esp-generate](https://github.com/esp-rs/esp-generate)  as described in its documentation.
Then, add this crate to your `Cargo.toml`:

```toml
[dependencies]
....
m5dial-bsp= "0.5.0"
....
```

In your main function then initialize this hall and use it:

```rust
let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);

let mut display = m5dial_bsp::get_screen!(peripherals);
let mut encoder = m5dial_bsp::get_encoder!(peripherals);

let mut board = m5dial_bsp::board_init!(peripherals);

```

For more information, please refer to the [API Documentation](https://antoinezen.github.io/m5dial-bsp). (Don't refers to the doc on docs.rs, as they fail to compile due to the lack of ESP specific toolchain.)

## Examples

Examples are located in the `examples` folder. Here is an index:

 - [screen_counter.rs](examples/screen_counter.rs): Demonstrate the rotary encoder usage.
 - [screen_counter_irq.rs](examples/screen_counter_irq.rs):  Demonstrate the rotary encoder usage, using interrupts.
 - [touch.rs](examples/touch.rs): Demonstrate the touchscreen.

## License

MIT license ([LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)

# Contributions

... are welcome !

# Credit

This crates rely on many crates of the [embedded Rust working group](https://github.com/rust-embedded/wg), [ESP support crates](https://github.com/esp-rs) from Expressif, [embedded-graphics](https://github.com/embedded-graphics) and device drivers such as :

 - [rotary-encoder-hal](https://crates.io/crates/rotary-encoder-hal)
 - [gc9c01 display driver](https://crates.io/crates/gc9a01-rs)
