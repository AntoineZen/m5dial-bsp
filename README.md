# M5 Dial Board Support Package (BSP)

This crates is a Board support package for the [M5 Dial](https://shop.m5stack.com/products/m5stack-dial-esp32-s3-smart-rotary-knob-w-1-28-round-touch-screen).

Feature list/roadmap:

 - [X] Screen driver (GC9A01)
 - [X] Rotary encoder
 - [ ] Touch-screen (FT3267)
 - [ ] Buzzer
 - [ ] Real-time clock (BM8563)
 - [ ] Port A (I2C)
 - [ ] Port B

## How to use

First generate a base project using [esp-generate](https://github.com/esp-rs/esp-generate) or [esp-idf-template](https://github.com/esp-rs/esp-idf-template) as described in they respective documentations.
Then, add this crate to your `Cargo.toml`:

```toml
[dependencies]
...
rotary-encoder-hal = "0.6.0"
...
```

In your main function then initialize this hall and use it:

```rust
let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
let peripherals = esp_hal::init(config);

let mut board = m5dial::init(peripherals);
...
```

## Examples

See [screen_counter.rs](examples/screen_counter.rs)

## License

MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

# Contriutions

... are welcome !

# Credit

This crates rely on many crates of the [embedded Rust working group](https://github.com/rust-embedded/wg), [ESP support crates](https://github.com/esp-rs) from Expressif, [embedded-graphics](https://github.com/embedded-graphics) and device drivers such as :

 - [rotary-encoder-hal](https://crates.io/crates/rotary-encoder-hal)
 - [gc9c01 display driver](https://crates.io/crates/gc9a01-rs)
