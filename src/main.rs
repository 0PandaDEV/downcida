use downcida::{Downcida, AudioFormat};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Downcida::download("5xPcP28rWbFUlYDOhcH58l", env::current_dir()?, Some("AU"), AudioFormat::FLAC).await?;
    Ok(())
}