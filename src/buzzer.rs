use core::result::Result::{Err, Ok};

// ESP32 Hardware abstraction
use esp_hal::{
    gpio::{AnyPin, Level},
    rmt::{Channel, ContinuousTxTransaction, LoopMode, PulseCode, Rmt, Tx, TxChannelConfig, TxChannelCreator},
    Blocking,
};
use defmt::{error};

const ESP32S3_RMT_DEFAULT_CLK_FREQ_HZ: u32 = 80_000_000;
const ESP32S3_RMT_MAX_TX_LOOP_NUM: u32 = 1023; // see ESP32S3 PAC: register is 10 bits wide
const CLK_FREQ_HZ: u32 = 3_200_000;
const PULSE_COUNT: usize = 1;

#[derive(Debug)]
pub enum Error {
    FreqTooLow(u16),
    DurationTooLongForFreq((u16, u16)),
    StopError(esp_hal::rmt::Error),
    TxError(esp_hal::rmt::Error),
}

#[derive(Debug)]
enum Resource<'b> {
    Channel(Channel<'b, Blocking, Tx>),
    ContinuousTx(ContinuousTxTransaction<'b>),
}

/// Buzzer driver using RMT peripheral to generate the signal.
#[derive(Debug)]
pub struct Buzzer<'b> {
    buf: [PulseCode; PULSE_COUNT + 1], // end marker required
    resource: Resource<'b>,
}

impl<'b> Buzzer<'b> {
    /// Build a new buzzer driver.
    ///
    /// This requires an RMT peripheral driver `rmt` and the output `pin`
    pub fn new(rmt: Rmt<'b, Blocking>, pin: AnyPin<'b>) -> Self {
        let channel = rmt
            .channel0
            .configure_tx(&TxChannelConfig::default()
                          .with_clk_divider((ESP32S3_RMT_DEFAULT_CLK_FREQ_HZ / CLK_FREQ_HZ).try_into().unwrap())
                          .with_idle_output_level(Level::Low)
                          .with_idle_output(false)
                          .with_carrier_modulation(false)
                          .with_carrier_high(1)
                          .with_carrier_low(1)
                          .with_carrier_level(Level::Low)
            ).expect("RMT channel0 could not configure")
            .with_pin(pin);
        Buzzer { buf: [PulseCode::default(), PulseCode::end_marker()], resource: Resource::Channel(channel) }
    }

    pub fn tone(mut self, freq_hz: u16, duration_ms: u16) -> Result<Self, (Self, Error)> {
        if freq_hz as u32 <= CLK_FREQ_HZ / (1 << 16) {
            // 50% duty on 15 bits is 16 bits for a single PulseCode -> ~49 Hz
            return Err((self, Error::FreqTooLow(freq_hz)));
        }

        let wave_length = CLK_FREQ_HZ / (freq_hz as u32); // range checked by freq_hz
        let count_u32 = (duration_ms as u32 * freq_hz as u32) / 1000;
        let count = if count_u32 > ESP32S3_RMT_MAX_TX_LOOP_NUM {
            return Err((self, Error::DurationTooLongForFreq((duration_ms, freq_hz))));
        } else {
            count_u32 as u16
        };

        let high = (wave_length / 2) as u16;
        let low = (wave_length - high as u32) as u16;
        self.buf[0] = PulseCode::new(Level::High, high, Level::Low, low);

        let ch = match self.resource {
            Resource::ContinuousTx(txn) => match txn.stop() {
                Ok(ch) => ch,
                Err((e, ch)) => return Err((Buzzer { buf: self.buf, resource: Resource::Channel(ch)}, Error::StopError(e))),
            },
            Resource::Channel(ch) => ch,
        };
        match ch.transmit_continuously(&self.buf, LoopMode::Finite(count)) {
            Ok(txn) => Ok(Buzzer { buf: self.buf, resource: Resource::ContinuousTx(txn) }),
            Err((e, ch)) => {
                error!("count = {}", count);
                Err((Buzzer { buf: self.buf, resource: Resource::Channel(ch) }, Error::TxError(e)))
            },
        }
    }
}
