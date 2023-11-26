use anyhow::{anyhow, Result};
use embedded_svc::{http, io::Write};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;

use crate::CONFIG;

fn req(payload: impl AsRef<[u8]>) -> Result<()> {
    let connection = EspHttpConnection::new(&Configuration::default())?;
    let mut client = http::client::Client::wrap(connection);
    let headers = [
        ("accept", "application/json"),
        ("Authorization", CONFIG.kodi_auth_basic),
    ];
    let mut request = client.post(CONFIG.kodi_endpoint, &headers)?;
    request.write_all(payload.as_ref())?;
    request.flush()?;
    let response = request.submit()?;
    let status = response.status();
    info!("Response status: {}", status);
    if !(200..300).contains(&status) {
        Err(anyhow!("Request failed"))?;
    }
    // XXX extract useful infos from response to return..
    Ok(())
}

pub fn play_pause() -> Result<()> {
    req(br#"{"jsonrpc": "2.0", "id": 0, "method": "Player.PlayPause", "params": { "playerid": 0 }}"#)
}
