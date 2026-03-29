use core::cell::Cell;
use core::result::Result::{Err, Ok};

// ESP32 Hardware abstraction
use defmt::error;
use esp_hal::time::{Duration, Rate};
use esp_hal::{
    Blocking,
    gpio::{AnyPin, Level},
    rmt::{
        Channel, ContinuousTxTransaction, LoopMode, PulseCode, Rmt, Tx, TxChannelConfig,
        TxChannelCreator,
    },
};

const ESP32S3_RMT_DEFAULT_CLK_FREQ_HZ: u32 = 80_000_000;
const ESP32S3_RMT_MAX_TX_LOOP_NUM: u32 = 1023; // see ESP32S3 PAC: register is 10 bits wide
const CLK_FREQ_HZ: u32 = 3_200_000;
const PULSE_COUNT: usize = 1;

#[derive(Debug)]
pub enum Error {
    FreqTooLow(Rate),
    DurationTooLongForFreq((Duration, Rate)),
    StopError(esp_hal::rmt::Error),
    TxError(esp_hal::rmt::Error),
    DefunkError,
}

enum Resource<'b> {
    Channel(Channel<'b, Blocking, Tx>),
    ContinuousTx(ContinuousTxTransaction<'b>),
    Empty,
}

impl Default for Resource<'_> {
    fn default() -> Self {
        Resource::Empty
    }
}

/// Buzzer driver using RMT peripheral to generate the signal.
pub struct Buzzer<'b> {
    buf: [PulseCode; PULSE_COUNT + 1], // end marker required
    resource: Cell<Resource<'b>>,
}

impl<'b> Buzzer<'b> {
    /// Build a new buzzer driver.
    ///
    /// This requires an RMT peripheral driver `rmt` and the output `pin`
    pub fn new(rmt: Rmt<'b, Blocking>, pin: AnyPin<'b>) -> Self {
        let channel = rmt
            .channel0
            .configure_tx(
                pin,
                TxChannelConfig::default()
                    .with_clk_divider(
                        (ESP32S3_RMT_DEFAULT_CLK_FREQ_HZ / CLK_FREQ_HZ)
                            .try_into()
                            .unwrap(),
                    )
                    .with_idle_output_level(Level::Low)
                    .with_idle_output(false)
                    .with_carrier_modulation(false)
                    .with_carrier_high(1)
                    .with_carrier_low(1)
                    .with_carrier_level(Level::Low),
            )
            .expect("RMT channel0 could not configure");
        Buzzer {
            buf: [PulseCode::default(), PulseCode::end_marker()],
            resource: Cell::new(Resource::Channel(channel)),
        }
    }

    pub fn tone(&mut self, freq: Rate, duration: Duration) -> Result<(), Error> {
        if freq.as_hz() <= CLK_FREQ_HZ / (1 << 16) {
            // 50% duty on 15 bits is 16 bits for a single PulseCode -> ~49 Hz
            return Err(Error::FreqTooLow(freq));
        }

        let wave_length = CLK_FREQ_HZ / (freq.as_hz()); // range checked by freq_hz
        let count_u32 = (duration.as_millis() as u32 * freq.as_hz()) / 1000;
        let count = if count_u32 > ESP32S3_RMT_MAX_TX_LOOP_NUM {
            return Err(Error::DurationTooLongForFreq((duration, freq)));
        } else {
            count_u32 as u16
        };

        let high = (wave_length / 2) as u16;
        let low = (wave_length - high as u32) as u16;
        self.buf[0] = PulseCode::new(Level::High, high, Level::Low, low);

        let ch = match self.resource.take() {
            Resource::ContinuousTx(txn) => match txn.stop() {
                Ok(ch) => ch,
                Err((e, ch)) => {
                    self.resource.set(Resource::Channel(ch));
                    return Err(Error::StopError(e));
                }
            },
            Resource::Channel(ch) => ch,
            Resource::Empty => return Err(Error::DefunkError),
        };
        match ch.transmit_continuously(&self.buf, LoopMode::Finite(count)) {
            Ok(txn) => {
                self.resource.set(Resource::ContinuousTx(txn));
                Ok(())
            }
            Err(e) => {
                error!("count = {}", e);
                Err(Error::TxError(e))
            }
        }
    }
}
