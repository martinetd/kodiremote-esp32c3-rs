use anyhow::Result;
use esp_idf_svc::hal::{gpio::*, rmt::config::TransmitConfig, rmt::*};

use crate::keypad::Keypad;

pub struct Board {
    pub sw9: PinDriver<'static, Gpio9, Input>,
    pub led: TxRmtDriver<'static>,
    pub keypad: Keypad,
}

pub fn init(pins: Pins, rmt: RMT) -> Result<Board> {
    let sw9 = PinDriver::input(pins.gpio9)?;
    let led_config = TransmitConfig::new().clock_divider(1);
    let led = TxRmtDriver::new(rmt.channel0, pins.gpio7, &led_config)?;
    let r1 = PinDriver::input(pins.gpio2)?;
    let r2 = PinDriver::input(pins.gpio1)?;
    let r3 = PinDriver::input(pins.gpio0)?;
    let r4 = PinDriver::input(pins.gpio4)?;
    let c1 = PinDriver::output_od(pins.gpio5)?;
    let c2 = PinDriver::output_od(pins.gpio10)?;
    let c3 = PinDriver::output_od(pins.gpio8)?;
    let c4 = PinDriver::output_od(pins.gpio6)?;
    Ok(Board {
        sw9,
        led,
        keypad: Keypad {
            rows: (r1, r2, r3, r4),
            cols: (c1, c2, c3, c4),
        },
    })
}
