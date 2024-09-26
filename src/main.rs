use downcida::Downcida;

#[tokio::main]
async fn main() {
    let spotify_id = "5xPcP28rWbFUlYDOhcH58l";
    let country = Some("US");

    match Downcida::download(spotify_id, country).await {
        Ok(file_name) => println!("Download completed successfully. File saved as: {}", file_name),
        Err(e) => eprintln!("Error: {}", e),
    }
}