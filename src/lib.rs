use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[macro_export]
macro_rules! downcida_err {
    ($($arg:tt)*) => {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!($($arg)*))))
    };
}

#[macro_export]
macro_rules! download_track {
    ($spotify_id:expr, $output_dir:expr, $country:expr, $format:expr) => {
        Downcida::download($spotify_id, $output_dir, $country, $format).await
    };
}

pub struct Downcida;

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    FLAC,
    M4A,
    MP3,
    OGG,
    OPUS,
    WAV,
}

impl AudioFormat {
    fn to_downscale_string(&self) -> &'static str {
        match self {
            AudioFormat::FLAC => "flac-16",
            AudioFormat::M4A => "m4a-320",
            AudioFormat::MP3 => "mp3-320",
            AudioFormat::OGG => "ogg-320",
            AudioFormat::OPUS => "opus-320",
            AudioFormat::WAV => "wav",
        }
    }

    fn to_extension(&self) -> &'static str {
        match self {
            AudioFormat::FLAC => "flac",
            AudioFormat::M4A => "m4a",
            AudioFormat::MP3 => "mp3",
            AudioFormat::OGG => "ogg",
            AudioFormat::OPUS => "opus",
            AudioFormat::WAV => "wav",
        }
    }
}

impl Downcida {
    /// Downloads a track from Spotify and saves it as an audio file.
    ///
    /// # Arguments
    ///
    /// * `spotify_id` - A string slice that holds the Spotify track ID.
    /// * `output_dir` - A PathBuf representing the directory where the downloaded file will be saved.
    /// * `country` - An optional string slice specifying the country code for the download region.
    /// * `format` - An AudioFormat enum specifying the desired audio format for the download.
    ///
    /// # Example
    ///
    /// ```
    /// use downcida::{Downcida, AudioFormat};
    /// use std::env;

    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     Downcida::download("5xPcP28rWbFUlYDOhcH58l", env::current_dir()?, Some("US"), AudioFormat::FLAC).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the download process fails at any stage,
    /// such as network issues, API errors, or file system problems.
    pub async fn download(
        spotify_id: &str,
        output_dir: PathBuf,
        country: Option<&str>,
        format: AudioFormat,
    ) -> Result<(PathBuf, u128), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap();

        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(spinner_style);

        progress_bar.set_message(format!("Starting download for Spotify ID: {}", spotify_id));
        let client = Client::new();
        let spotify_url = format!("https://open.spotify.com/track/{}", spotify_id);
        let country = country.unwrap_or("auto");

        let initial_request = serde_json::json!({
            "account": {
                "id": country,
                "type": "country"
            },
            "downscale": format.to_downscale_string(),
            "handoff": true,
            "metadata": false,
            "private": true,
            "token": {
                "expiry": 1727529052,
                "primary": "qwUGICHFtvwf98Jfn5m3L6E_O5U"
            },
            "upload": {
                "enabled": false,
                "service": "pixeldrain"
            },
            "url": spotify_url
        });

        progress_bar.set_message("Sending initial request to Lucida API");
        let response = client
            .post("https://lucida.to/api/load?url=/api/fetch/stream/v2")
            .json(&initial_request)
            .send()
            .await?;

        let initial_response: Value = response.json().await?;

        if !initial_response["success"].as_bool().unwrap_or(false) {
            let error_message = initial_response["error"].as_str().unwrap_or("Unknown error");
            progress_bar.finish_with_message(format!("❌ Initial request failed: {}", error_message));
            return downcida_err!("Initial request failed: {}. Please check the Spotify ID and try again.", error_message);
        }

        let handoff = initial_response["handoff"].as_str().ok_or("No handoff value in response")?;
        let server = initial_response["server"].as_str().ok_or("No server value in response")?;

        let completion_url = format!("https://{}.lucida.to/api/fetch/request/{}", server, handoff);

        progress_bar.set_message("Waiting for track processing to complete");
        loop {
            let completion_response: Value = client.get(&completion_url).send().await?.json().await?;

            if completion_response["status"].as_str() == Some("completed") {
                break;
            }

            if completion_response["status"].as_str() == Some("error") {
                let error_message = completion_response["message"].as_str().unwrap_or("Unknown error");
                progress_bar.finish_with_message(format!("❌ API request failed: {}", error_message));
                return downcida_err!("API request failed: {}", error_message);
            }

            progress_bar.tick();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let download_url = format!("https://{}.lucida.to/api/fetch/request/{}/download", server, handoff);
        progress_bar.set_message("Starting file download");
        let mut download_response = client.get(&download_url).send().await?;

        let file_extension = format.to_extension();
        let file_name = format!("{}.{}", handoff, file_extension);
        let output_path = output_dir.join(&file_name);

        let mut file = File::create(&output_path)?;

        let total_size = download_response.content_length().unwrap_or(0);
        let progress_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {bytes_per_sec}")
            .unwrap()
            .progress_chars("##-");

        let download_progress = ProgressBar::new(total_size);
        download_progress.set_style(progress_style);

        while let Some(chunk) = download_response.chunk().await? {
            file.write_all(&chunk)?;
            download_progress.inc(chunk.len() as u64);
        }

        download_progress.finish_with_message("Download completed");

        let duration = start_time.elapsed().as_millis();
        println!("✅ Download completed: {} in {} ms", output_path.display(), duration);
        Ok((output_path, duration))
    }
}