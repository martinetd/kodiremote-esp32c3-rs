use anyhow::{anyhow, Context, Result};
use embedded_svc::{
    http,
    io::{Read, Write},
};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::info;
use serde::{Deserialize, Serialize};

use crate::CONFIG;

#[derive(Serialize)]
struct JsonRequest<'a, Payload> {
    jsonrpc: &'a str,
    id: u8,
    method: &'a str,
    params: &'a Payload,
}

#[derive(Serialize)]
struct PlayerId {
    playerid: i8,
}

#[derive(Serialize)]
struct Volume {
    volume: i8,
}

#[derive(Serialize)]
struct Properties<'a> {
    properties: &'a [&'a str],
}

#[derive(Deserialize)]
struct JsonResponse<Payload> {
    #[allow(unused)]
    jsonrpc: String,
    #[allow(unused)]
    id: u8,
    result: Payload,
}

#[derive(Deserialize)]
struct JsonErrorInner {
    #[allow(unused)]
    code: i32,
    message: String,
}

#[derive(Deserialize)]
struct JsonError {
    #[allow(unused)]
    jsonrpc: String,
    #[allow(unused)]
    id: u8,
    error: JsonErrorInner,
}

#[derive(Deserialize)]
struct Vol {
    volume: i8,
}

fn req<'a, Req, Resp>(method: &'a str, params: &'a Req, get_response: bool) -> Result<Option<Resp>>
where
    Req: Serialize,
    Resp: for<'b> Deserialize<'b>,
{
    if cfg!(feature = "nowifi") {
        return Ok(None);
    }

    let payload = serde_json::to_string(&JsonRequest::<Req> {
        jsonrpc: "2.0",
        id: 0,
        method,
        params,
    })?;

    let connection = EspHttpConnection::new(&Configuration::default())?;
    let mut client = http::client::Client::wrap(connection);
    let headers = [
        ("accept", "application/json"),
        ("Authorization", CONFIG.kodi_auth_basic),
    ];
    let mut request = client.post(CONFIG.kodi_endpoint, &headers)?;
    log::info!("Submitting {}", String::from_utf8_lossy(payload.as_ref()));
    request.write_all(payload.as_bytes())?;
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

    if let Ok(json_error) = serde_json::from_slice::<JsonError>(&buf[0..size]) {
        return Err(anyhow!("Failed: {}", json_error.error.message));
    }
    if !get_response {
        return Ok(None);
    }

    let json_response: JsonResponse<Resp> = serde_json::from_slice(&buf[0..size])?;
    Ok(Some(json_response.result))
}

pub fn play_pause() -> Result<()> {
    req::<PlayerId, ()>("Player.PlayPause", &PlayerId { playerid: 0 }, false)?;
    Ok(())
}

pub fn set_vol(volume: i8) -> Result<()> {
    req::<Volume, ()>("Application.SetVolume", &Volume { volume }, false)?;
    Ok(())
}
pub fn update_vol(increment: i8) -> Result<()> {
    let cur_vol = req::<Properties, Vol>(
        "Application.GetProperties",
        &Properties {
            properties: &["volume"],
        },
        true,
    )?
    .context("No response")?;
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
