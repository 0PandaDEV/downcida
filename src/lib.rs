use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[macro_export]
macro_rules! downcida_err {
    ($($arg:tt)*) => {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))))
    };
}

pub struct Downcida;

impl Downcida {
    pub async fn download(spotify_id: &str, output_dir: PathBuf, country: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let client = Client::new();
        let spotify_url = format!("https://open.spotify.com/track/{}", spotify_id);
        let country = country.unwrap_or("auto");

        let initial_request = serde_json::json!({
            "url": spotify_url,
            "metadata": false,
            "private": true,
            "handoff": true,
            "account": {
                "type": "country",
                "id": country
            },
            "upload": {
                "enabled": false,
                "service": "pixeldrain"
            },
            "downscale": "original",
            "token": {
                "primary": "XLj8xzbZJSfloOrw-XUmNGAmG6k",
                "expiry": 1727419066
            }
        });

        let response = client
            .post("https://lucida.to/api/load?url=/api/fetch/stream/v2")
            .json(&initial_request)
            .send()
            .await?;

        let initial_response: Value = response.json().await?;

        if !initial_response["success"].as_bool().unwrap_or(false) {
            let error_message = initial_response["error"].as_str().unwrap_or("Unknown error");
            return downcida_err!("Initial request failed: {}. Please check the Spotify ID and try again.", error_message);
        }

        let handoff = initial_response["handoff"].as_str().ok_or("No handoff value in response")?;
        let server = initial_response["server"].as_str().ok_or("No server value in response")?;

        let completion_url = format!("https://{}.lucida.to/api/fetch/request/{}", server, handoff);

        loop {
            let completion_response: Value = client.get(&completion_url).send().await?.json().await?;

            if completion_response["status"].as_str() == Some("completed") {
                break;
            }

            if completion_response["status"].as_str() == Some("error") {
                let error_message = completion_response["message"].as_str().unwrap_or("Unknown error");
                return downcida_err!("API request failed: {}", error_message);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let download_url = format!("https://{}.lucida.to/api/fetch/request/{}/download", server, handoff);
        let mut download_response = client.get(&download_url).send().await?;

        let file_name = format!("{}.flac", handoff);
        let output_path = output_dir.join(&file_name);

        let mut file = File::create(&output_path)?;

        while let Some(chunk) = download_response.chunk().await? {
            file.write_all(&chunk)?;
        }

        Ok(output_path)
    }
}