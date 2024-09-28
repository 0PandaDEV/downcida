# Downcida

<ins>**This project does not work becuase of a Token that is needed see [#1](https://github.com/0PandaDEV/downcida/issues/1)**</ins>

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
use downcida::{Downcida, AudioFormat};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Downcida::download("5xPcP28rWbFUlYDOhcH58l", env::current_dir()?, Some("US"), AudioFormat::FLAC).await?;
    Ok(())
}
```
