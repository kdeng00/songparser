// use std::error::Error;
// use tokio::io::AsyncReadExt;
// use tokio::net::{TcpListener, TcpStream};
// use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let app_base_url = get_icarus_url().await;

    loop {
        println!("Base URL: {}", app_base_url);

        // TODO: Update the api/v2/song/queue/next endpoint to only retrieve queued song that is
        // ready to be processed. Make necessary changes to other endpoints

        let api_url = format!("{}/api/v2/song/queue/next", app_base_url);

        match client.get(api_url).send().await {
            Ok(response) => {
                let body = response.text().await?;
                println!("API response: {}", body);
                // Process data here...

                // TODO: Parse the response body to a struct
                // TODO: Get queued song data
                // TODO: Get queued song's metadata
                // TODO: Get queued coverart
                // TODO: Get queued coverart's data
                // TODO: Apply metadata to the queued song
                // TODO: Update the queued song with the updated queued song
                // TODO: Create song
                // TODO: Create coverart
                // TODO: Wipe data from queued song
                // TODO: Wipe data from queued coverart
            }
            Err(e) => eprintln!("API call failed: {}", e),
        }

        println!("Sleeping");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    // Ok(())
}

async fn get_icarus_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("ICARUS_BASE_API_URL").expect("Could not find url")
}
