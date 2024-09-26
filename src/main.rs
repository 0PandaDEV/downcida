use downcida::RequestReplicator;

#[tokio::main]
async fn main() {
    let replicator = RequestReplicator::new();
    let spotify_url = "https://open.spotify.com/track/5xPcP28rWbFUlYDOhcH58l";
    let country = "auto";

    match replicator.replicate_requests(spotify_url, country).await {
        Ok(_) => println!("Request completed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}