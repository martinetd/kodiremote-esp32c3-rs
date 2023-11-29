use anyhow::{anyhow, Context, Result};
use core::any::TypeId;
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

#[derive(Deserialize, Serialize)]
struct Volume {
    volume: i8,
}

#[derive(Serialize)]
struct Properties<'a> {
    properties: &'a [&'a str],
}

#[derive(Serialize)]
struct Repeat<'a> {
    playerid: i8,
    repeat: &'a str,
}

#[derive(Serialize)]
struct PlaylistPos {
    playlistid: i8,
    position: i32,
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
struct Speed {
    speed: i8,
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

fn req<'a, Req, Resp>(method: &'a str, params: &'a Req) -> Result<Option<Resp>>
where
    Req: Serialize,
    Resp: for<'b> Deserialize<'b> + 'static,
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
    if TypeId::of::<Resp>() == TypeId::of::<()>() {
        return Ok(None);
    }

    let json_response: JsonResponse<Resp> = serde_json::from_slice(&buf[0..size])?;
    Ok(Some(json_response.result))
}

pub fn play_pause() -> Result<()> {
    match req::<PlayerId, Speed>("Player.PlayPause", &PlayerId { playerid: 0 }) {
        Ok(Some(Speed { speed: i })) if i != 0 => {
            // ensure we have repeat on, it somehow keeps dropping...
            req::<Repeat, ()>(
                "Player.SetRepeat",
                &Repeat {
                    playerid: 0,
                    repeat: "all",
                },
            )?;
        }
        Err(_) => {
            // try to play player 0
            req::<PlaylistPos, ()>(
                "Player.Open",
                &PlaylistPos {
                    playlistid: 0,
                    position: 0,
                },
            )?;
        }
        _ => (),
    }
    Ok(())
}

pub fn set_vol(volume: i8) -> Result<()> {
    req::<Volume, ()>("Application.SetVolume", &Volume { volume })?;
    Ok(())
}
pub fn update_vol(increment: i8) -> Result<()> {
    let cur_vol = req::<Properties, Volume>(
        "Application.GetProperties",
        &Properties {
            properties: &["volume"],
        },
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
