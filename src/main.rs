use downcida::Downcida;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let spotify_id = "5xPcP28rWbFUlYDOhcH58l";
    let output_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let country = Some("US");

    match Downcida::download(spotify_id, output_dir, country).await {
        Ok(file_path) => println!("Download completed successfully. File saved as: {}", file_path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}