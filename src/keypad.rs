use anyhow::Result;

use esp_idf_svc::hal::gpio::*;

#[allow(clippy::type_complexity)]
pub struct Keypad {
    pub rows: (
        PinDriver<'static, Gpio2, Input>,
        PinDriver<'static, Gpio1, Input>,
        PinDriver<'static, Gpio0, Input>,
        PinDriver<'static, Gpio4, Input>,
    ),
    pub cols: (
        PinDriver<'static, Gpio5, Output>,
        PinDriver<'static, Gpio10, Output>,
        PinDriver<'static, Gpio8, Output>,
        PinDriver<'static, Gpio6, Output>,
    ),
}

static KEYMAP: [[char; 4]; 4] = [
    ['D', '#', '0', '*'],
    ['C', '9', '8', '7'],
    ['B', '6', '5', '4'],
    ['A', '3', '2', '1'],
];

pub fn scan_keypad(keypad: &mut Keypad) -> Result<Option<char>> {
    // hand roll loops because there's no iterator/.get over tuples...
    keypad.rows.0.set_pull(Pull::Up)?;
    keypad.rows.1.set_pull(Pull::Up)?;
    keypad.rows.2.set_pull(Pull::Up)?;
    keypad.rows.3.set_pull(Pull::Up)?;

    // check col 0
    keypad.cols.0.set_low()?;
    keypad.cols.1.set_high()?;
    keypad.cols.2.set_high()?;
    keypad.cols.3.set_high()?;
    if keypad.rows.0.is_low() {
        return Ok(Some(KEYMAP[0][0]));
    }
    if keypad.rows.1.is_low() {
        return Ok(Some(KEYMAP[0][1]));
    }
    if keypad.rows.2.is_low() {
        return Ok(Some(KEYMAP[0][2]));
    }
    if keypad.rows.3.is_low() {
        return Ok(Some(KEYMAP[0][3]));
    }

    // check col 1
    keypad.cols.0.set_high()?;
    keypad.cols.1.set_low()?;
    keypad.cols.2.set_high()?;
    keypad.cols.3.set_high()?;
    if keypad.rows.0.is_low() {
        return Ok(Some(KEYMAP[1][0]));
    }
    if keypad.rows.1.is_low() {
        return Ok(Some(KEYMAP[1][1]));
    }
    if keypad.rows.2.is_low() {
        return Ok(Some(KEYMAP[1][2]));
    }
    if keypad.rows.3.is_low() {
        return Ok(Some(KEYMAP[1][3]));
    }

    // check col 2
    keypad.cols.0.set_high()?;
    keypad.cols.1.set_high()?;
    keypad.cols.2.set_low()?;
    keypad.cols.3.set_high()?;
    if keypad.rows.0.is_low() {
        return Ok(Some(KEYMAP[2][0]));
    }
    if keypad.rows.1.is_low() {
        return Ok(Some(KEYMAP[2][1]));
    }
    if keypad.rows.2.is_low() {
        return Ok(Some(KEYMAP[2][2]));
    }
    if keypad.rows.3.is_low() {
        return Ok(Some(KEYMAP[2][3]));
    }

    // check col 3
    keypad.cols.0.set_high()?;
    keypad.cols.1.set_high()?;
    keypad.cols.2.set_high()?;
    keypad.cols.3.set_low()?;
    if keypad.rows.0.is_low() {
        return Ok(Some(KEYMAP[3][0]));
    }
    if keypad.rows.1.is_low() {
        return Ok(Some(KEYMAP[3][1]));
    }
    if keypad.rows.2.is_low() {
        return Ok(Some(KEYMAP[3][2]));
    }
    if keypad.rows.3.is_low() {
        return Ok(Some(KEYMAP[3][3]));
    }
    Ok(None)
}
