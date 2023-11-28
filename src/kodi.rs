use anyhow::{anyhow, Context, Result};
use embedded_svc::{
    http,
    io::{Read, Write},
};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;
use serde::Deserialize;

use crate::CONFIG;

// We don't use this as we build json directly
//#[derive(Seralize)]
//struct JsonRequest<Payload> {
//    Jsonrpc: String,
//    Id: u8,
//    Method: String,
//    Params: Payload,
//}

#[derive(Deserialize)]
struct JsonResponse<Payload> {
    #[allow(unused)]
    jsonrpc: String,
    #[allow(unused)]
    id: u8,
    result: Payload,
}

#[derive(Deserialize)]
struct Vol {
    volume: i8,
}

fn req<Resp>(payload: impl AsRef<[u8]>, get_response: bool) -> Result<Option<Resp>>
where
    Resp: for<'a> Deserialize<'a>,
{
    if cfg!(feature = "nowifi") {
        return Ok(None);
    }
    let connection = EspHttpConnection::new(&Configuration::default())?;
    let mut client = http::client::Client::wrap(connection);
    let headers = [
        ("accept", "application/json"),
        ("Authorization", CONFIG.kodi_auth_basic),
    ];
    let mut request = client.post(CONFIG.kodi_endpoint, &headers)?;
    log::info!("Submitting {}", String::from_utf8_lossy(payload.as_ref()));
    request.write_all(payload.as_ref())?;
    request.flush()?;
    let mut response = request.submit()?;
    let status = response.status();
    info!("Response status: {}", status);
    if !(200..300).contains(&status) {
        Err(anyhow!("Request failed"))?;
    }
    let mut buf = [0_u8; 256];
    let size = Read::read(&mut response, &mut buf)?;
    log::info!("Got {}", String::from_utf8_lossy(&buf[0..size]));

    // XXX check errors; sent as 200...
    // I (14433) kodiremote::kodi: Got {"error":{"code":-32700,"message":"Parse error."},"id":null,"jsonrpc":"2.0"}
    if !get_response {
        return Ok(None);
    }

    let json_response: JsonResponse<Resp> = serde_json::from_slice(&buf[0..size])?;
    Ok(Some(json_response.result))
}

pub fn play_pause() -> Result<()> {
    req::<()>(br#"{"jsonrpc": "2.0", "id": 0, "method": "Player.PlayPause", "params": { "playerid": 0 }}"#, false)?;
    Ok(())
}

pub fn set_vol(vol: i8) -> Result<()> {
    let payload = format!(
        r#"{{"jsonrpc": "2.0", "id": 0, "method": "Application.SetVolume", "params": {{ "volume": {vol} }}}}"#
    );
    req::<()>(payload.as_bytes(), false)?;
    Ok(())
}
pub fn update_vol(increment: i8) -> Result<()> {
    let cur_vol = req::<Vol>(br#"{"jsonrpc": "2.0", "id": 0, "method": "Application.GetProperties", "params": { "properties": ["volume"]}}"#, true)?.context("No response")?;
    let new_vol = match cur_vol.volume + increment {
        i if i < 0 => 0,
        i if i > 100 => 100,
        i => i,
    };
    set_vol(new_vol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vol() {
        let response = br#"{"id":0,"jsonrpc":"2.0","result":{"muted":false,"volume":38}}"#;
        let json_response: JsonResponse<Vol> = serde_json::from_slice(response).unwrap();
        assert_eq!(json_response.result.volume, 38i8)
    }
}
