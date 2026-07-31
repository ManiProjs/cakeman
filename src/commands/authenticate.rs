use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{thread, time::Duration};

use crate::terminal::{info, success};

const CLIENT_ID: &str = "Ov23li1y2zrP9nlfgT0b";

#[derive(Debug, Deserialize)]
struct DeviceResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthConfig {
    github_token: String,
}

pub fn execute() -> Result<()> {
    info("Authenticating with GitHub...");

    let client = Client::new();

    let device = request_device_code(&client)?;

    println!();
    info("Open this URL in your browser:");
    info(&device.verification_uri);
    println!();
    info("Enter this code:");
    info(&device.user_code);
    println!();

    info(&format!("Code expires in {} seconds", device.expires_in));

    let token = poll_for_token(&client, &device.device_code, device.interval.unwrap_or(5))?;

    save_token(&token)?;

    success("Successfully authenticated!");

    Ok(())
}

fn request_device_code(client: &Client) -> Result<DeviceResponse> {
    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID)])
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub device authorization failed: {}",
            response.status()
        ));
    }

    Ok(response.json()?)
}

fn poll_for_token(client: &Client, device_code: &str, interval: u64) -> Result<String> {
    loop {
        thread::sleep(Duration::from_secs(interval));

        let response: TokenResponse = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()?
            .json()?;

        if let Some(token) = response.access_token {
            return Ok(token);
        }

        match response.error.as_deref() {
            Some("authorization_pending") => {
                continue;
            }

            Some("slow_down") => {
                thread::sleep(Duration::from_secs(5));
                continue;
            }

            Some(error) => {
                return Err(anyhow!("GitHub authentication failed: {}", error));
            }

            None => {
                return Err(anyhow!("Unknown GitHub authentication error"));
            }
        }
    }
}

fn save_token(token: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Cannot find config directory"))?
        .join("cakeman");

    std::fs::create_dir_all(&config_dir)?;

    let config = AuthConfig {
        github_token: token.to_string(),
    };

    let path = config_dir.join("auth.json");

    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;

    Ok(())
}
