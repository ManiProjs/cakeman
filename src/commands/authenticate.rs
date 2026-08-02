use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::{
    auth::storage,
    terminal::{info, success},
};

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

pub async fn execute() -> Result<()> {
    info("Authenticating with GitHub...");

    let client = Client::new();

    let device = request_device_code(&client).await?;

    println!();

    info("Open this URL in your browser:");
    info(&device.verification_uri);

    println!();

    info("Enter this code:");
    info(&device.user_code);

    println!();

    info(&format!("Code expires in {} seconds", device.expires_in));

    let token = poll_for_token(&client, &device.device_code, device.interval.unwrap_or(5)).await?;

    storage::save(&token)?;

    success("Successfully authenticated!");

    Ok(())
}

async fn request_device_code(client: &Client) -> Result<DeviceResponse> {
    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", "repo")])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub device authorization failed: {}",
            response.status()
        ));
    }

    Ok(response.json().await?)
}

async fn poll_for_token(client: &Client, device_code: &str, interval: u64) -> Result<String> {
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let response: TokenResponse = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?
            .json()
            .await?;

        if let Some(token) = response.access_token {
            return Ok(token);
        }

        match response.error.as_deref() {
            Some("authorization_pending") => {
                continue;
            }

            Some("slow_down") => {
                tokio::time::sleep(Duration::from_secs(5)).await;

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
