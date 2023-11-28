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

static mut BUF: [u8; 256] = [0_u8; 256];

fn req<Resp>(payload: impl AsRef<[u8]>, get_response: bool) -> Result<Option<Resp>>
where
    Resp: Deserialize<'static>,
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
    if !get_response {
        // XXX check errors; sent as 200...
        // I (14433) kodiremote::kodi: Got {"error":{"code":-32700,"message":"Parse error."},"id":null,"jsonrpc":"2.0"}

        unsafe {
            let size = Read::read(&mut response, &mut BUF)?;
            log::info!("Got {}", String::from_utf8_lossy(&BUF[0..size]));
        }
        return Ok(None);
    }
    // XXX unsafe: json apparently refernses the original buffer.. clone?
    unsafe {
        let size = Read::read(&mut response, &mut BUF)?;
        log::info!("Got {}", String::from_utf8_lossy(&BUF[0..size]));
        let json_response: JsonResponse<Resp> = serde_json::from_slice(&BUF[0..size])?;
        Ok(Some(json_response.result))
    }
}

pub fn play_pause() -> Result<()> {
    req::<()>(br#"{"jsonrpc": "2.0", "id": 0, "method": "Player.PlayPause", "params": { "playerid": 0 }}"#, false)?;
    Ok(())
}
