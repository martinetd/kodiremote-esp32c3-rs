use anyhow::Result;
use esp_idf_svc::hal::gpio::*;

use crate::keypad::Keypad;

pub struct Board {
    pub sw9: PinDriver<'static, Gpio9, Input>,
    pub led7: PinDriver<'static, Gpio7, Output>,
    pub keypad: Keypad,
}

pub fn init(pins: Pins) -> Result<Board> {
    let sw9 = PinDriver::input(pins.gpio9)?;
    let led7 = PinDriver::output(pins.gpio7)?;
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
        led7,
        keypad: Keypad {
            rows: (r1, r2, r3, r4),
            cols: (c1, c2, c3, c4),
        },
    })
}
