use std::io::Write;

pub const SECONDS_TO_SLEEP: u64 = 5;

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

                            println!("Fetching song queue data");
                            match api::fetch_song_queue_data::get_data(
                                &app_base_url,
                                &song_queue_item.data[0].id,
                            )
                            .await
                            {
                                Ok(response) => {
                                    // Process data here...
                                    let all_bytes =
                                        api::fetch_song_queue_data::response::parse_response(
                                            response,
                                        )
                                        .await?;

                                    let (directory, filename) =
                                        generate_song_queue_dir_and_filename().await;
                                    let save_path =
                                        save_song_to_fs(&directory, &filename, &all_bytes).await;

                                    println!("Saved at: {:?}", save_path);

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
                                Err(err) => {
                                    eprintln!("Error fetching song queue data: {:?}", err);
                                }
                            }
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
        tokio::time::sleep(tokio::time::Duration::from_secs(SECONDS_TO_SLEEP)).await;
    }
}

// TODO: Consider having something like this in icarus_models
pub async fn generate_song_queue_dir_and_filename() -> (String, String) {
    let mut song = icarus_models::song::Song::default();
    song.filename = song.generate_filename(icarus_models::types::MusicTypes::FlacExtension, true);

    song.directory = icarus_envy::environment::get_root_directory().await;

    (song.directory, song.filename)
}

// TODO: Check to see if this is available in icarus_models
pub async fn save_song_to_fs(
    directory: &String,
    filename: &String,
    data: &[u8],
) -> std::path::PathBuf {
    // TODO: Add function to save bytes to a file in icarus_models
    // repo
    let dir = std::path::Path::new(directory);
    let save_path = dir.join(filename);

    let mut file = std::fs::File::create(&save_path).unwrap();
    file.write_all(data).unwrap();

    save_path
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

    pub mod fetch_song_queue_data {
        pub async fn get_data(
            base_url: &String,
            id: &uuid::Uuid,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::new();
            let endpoint = String::from("api/v2/song/queue");
            let api_url = format!("{}/{}/{}", base_url, endpoint, id);
            client.get(api_url).send().await
        }

        pub mod response {
            use futures::StreamExt;

            pub async fn parse_response(
                response: reqwest::Response,
            ) -> Result<Vec<u8>, reqwest::Error> {
                // TODO: At some point, handle the flow if the size is small or
                // large
                let mut byte_stream = response.bytes_stream();
                let mut all_bytes = Vec::new();

                while let Some(chunk) = byte_stream.next().await {
                    let chunk = chunk?;
                    all_bytes.extend_from_slice(&chunk);
                }

                Ok(all_bytes)
            }
        }
    }
}

async fn get_icarus_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("ICARUS_BASE_API_URL").expect("Could not find url")
}
