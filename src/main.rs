use anyhow::Result;
use core::str;

use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals};

mod board;
mod keypad;
mod kodi;
mod led;
mod wifi;

#[toml_cfg::toml_config]
struct Config {
    #[default("guest")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("http://10.1.2.3:8080/jsonrpc")]
    kodi_endpoint: &'static str,
    #[default("Basic dGVzdDpwYXNz")]
    kodi_auth_basic: &'static str,
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // `CONFIG` constant is auto-generated by toml_config
    let kodi_config = CONFIG;

    let _wifi = wifi::wifi(
        kodi_config.wifi_ssid,
        kodi_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    let mut board = board::init(peripherals.pins, peripherals.rmt)?;
    let mut last: Option<char> = None;
    let mut debounce = 1i8;

    let mut toggle_led = false;

    loop {
        let mut key = keypad::scan_keypad(&mut board.keypad)?;
        // work around bad keyboard
        if key == Some('#') && last == Some('3') {
            key = Some('3');
        }
        if key != last {
            if debounce >= 0 {
                log::info!("{last:?} bounced to {key:?}?");
            }
            last = key;
            debounce = 3i8;
        } else if debounce > 0 {
            debounce -= 1;
        } else if debounce == 0 {
            debounce = -1;
            log::info!("Key pressed {key:?}");
            match key {
                Some('1') => {
                    log::info!("toggling play/pause");
                    kodi::play_pause()?;
                }
                Some('2') => {
                    if toggle_led {
                        log::info!("Turning LED on");
                        led::neopixel(led::Rgb::new(10, 10, 0), &mut board.led)?;
                        toggle_led = false;
                    } else {
                        log::info!("Turning LED off");
                        led::neopixel(led::Rgb::new(0, 0, 0), &mut board.led)?;
                        toggle_led = true;
                    }
                }
                _ => (),
            };
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
