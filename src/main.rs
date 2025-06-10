#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_base_url = get_icarus_url().await;

    loop {
        println!("Base URL: {}", app_base_url);

        match api::fetch_next_queue_item(&app_base_url).await {
            Ok(response) => {
                match response
                    .json::<responses::fetch_next_queue_item::SongQueueItem>()
                    .await
                {
                    Ok(song_queue_item) => {
                        if !song_queue_item.data.is_empty() {
                            println!("Song queue item: {:?}", song_queue_item);

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
                        } else {
                            println!("No data to fetch");
                        }
                    }
                    Err(err) => {
                        eprintln!("Error: {:?}", err);
                    }
                }
            }
            Err(e) => eprintln!("API call failed: {}", e),
        }

        println!("Sleeping");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

mod responses {
    pub mod fetch_next_queue_item {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct QueueItem {
            pub id: uuid::Uuid,
            pub filename: String,
            pub status: String,
        }

        #[derive(Debug, Deserialize, Serialize)]
        pub struct SongQueueItem {
            pub message: String,
            pub data: Vec<QueueItem>,
        }
    }
}

mod api {
    pub async fn fetch_next_queue_item(
        base_url: &String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let fetch_endpoint = String::from("api/v2/song/queue/next");
        let api_url = format!("{}/{}", base_url, fetch_endpoint);
        client.get(api_url).send().await
    }
}

async fn get_icarus_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("ICARUS_BASE_API_URL").expect("Could not find url")
}
