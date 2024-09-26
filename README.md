# Downcida

Downcida is a Rust crate that allows you to download Spotify tracks using the [Lucida API](https://lucida.to/). It provides a simple interface to download audio files from Spotify tracks and save them to a specified directory.

## Features

- [x] Different download Formats (FLAC, WAV, OGG, OPUS, M4A, MP3 e.g)
- [x] Spotify
- [ ] Qobuz
- [ ] Tidal
- [ ] Soundcloud
- [ ] Deezer
- [ ] Amazon Music
- [ ] Beatport

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
downcida = "0.1.2"
```

## Usage

Here's a basic example of how to use Downcida:

```rs
use downcida::Downcida;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
  let spotify_id = "5xPcP28rWbFUlYDOhcH58l"; // Replace with your Spotify track ID
  let output_dir = PathBuf::from("downloads"); // Set the folder where the flac file will be saved
  let country = Some("US"); // Optional: specify a country, or use None for auto

  match Downcida::download(spotify_id, output_dir, country).await {
    Ok(file_path) => println!("Download completed successfully. File saved as: {}", file_path.display()),
    Err(e) => eprintln!("Error: {}", e),
  }
}
```
