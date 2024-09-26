use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct RequestReplicator {
    client: Client,
}

impl RequestReplicator {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn replicate_requests(&self, spotify_url: &str, country: &str) -> Result<(), Box<dyn std::error::Error>> {
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

        println!("\n[1] Sending initial request to https://lucida.to/api/load?url=/api/fetch/stream/v2");
        let response = self.client
            .post("https://lucida.to/api/load?url=/api/fetch/stream/v2")
            .json(&initial_request)
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        println!("\n[1] Initial Response Status: {}", status);
        println!("\n[1] Initial Response Headers: {:#?}", headers);
        println!("\n[1] Initial Response Body: {}", body);

        let initial_response: Value = serde_json::from_str(&body)?;

        println!("\n[1] Parsed Initial Response: {:#?}", initial_response);

        if !initial_response["success"].as_bool().unwrap_or(false) {
            let error_message = initial_response["error"].as_str().unwrap_or("Unknown error");
            return Err(format!("Initial request failed: {}. Please check the Spotify URL and try again.", error_message).into());
        }

        let handoff = initial_response["handoff"].as_str().ok_or("No handoff value in response")?;
        let server = initial_response["server"].as_str().ok_or("No server value in response")?;

        let completion_check_url = format!("https://lucida.to/api/load?url=/api/fetch/request/{}&force={}", handoff, server);
        println!("\n[2] Checking completion status: GET {}", completion_check_url);
        let completion_response = self.client.get(&completion_check_url).send().await?;
        
        println!("\n[2] Completion check response status: {}", completion_response.status());
        let completion_body = completion_response.text().await?;
        println!("\n[2] Completion check response body: {}", completion_body);

        let completion_url = format!("https://{}.lucida.to/api/fetch/request/{}", server, handoff);
        println!("\n[3] Polling completion status: GET {}", completion_url);

        loop {
            let completion_response = self.client.get(&completion_url).send().await?;
            let completion_body = completion_response.text().await?;
            println!("\n[3] Completion Response: {}", completion_body);

            let completion_json: Value = serde_json::from_str(&completion_body)?;

            if completion_json["status"].as_str() == Some("completed") {
                break;
            }

            if completion_json["status"].as_str() == Some("error") {
                let error_message = completion_json["message"].as_str().unwrap_or("Unknown error");
                return Err(format!("API request failed: {}", error_message).into());
            }

            println!("Waiting for completion...");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let download_url = format!("https://{}.lucida.to/api/fetch/request/{}/download", server, handoff);
        println!("\n[4] Downloading file: GET {}", download_url);
        let mut download_response = self.client.get(&download_url).send().await?;

        println!("\n[4] Download response status: {}", download_response.status());

        let file_name = format!("{}.flac", handoff);
        let path = Path::new(&file_name);
        let mut file = File::create(path)?;

        let mut total_bytes = 0;
        while let Some(chunk) = download_response.chunk().await? {
            file.write_all(&chunk)?;
            total_bytes += chunk.len();
            println!("Downloaded {} bytes", total_bytes);
        }

        println!("\nDownload completed. File saved as: {}", file_name);

        Ok(())
    }
}