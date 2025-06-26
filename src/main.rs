use std::io::Write;

pub const SECONDS_TO_SLEEP: u64 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_base_url = icarus_envy::environment::get_icarus_base_api_url().await;

    loop {
        println!("Base URL: {}", app_base_url);

        match is_queue_empty(&app_base_url).await {
            Ok((empty, song_queue_item)) => {
                if !empty {
                    println!("Queue is not empty");
                    println!("SongQueueItem: {:?}", song_queue_item);
                    let song_queue_id = song_queue_item.data[0].id;

                    // TODO: Do something with the result later
                    let _ = process_song(&app_base_url, &song_queue_id).await;
                } else {
                    println!("Queue is empty");
                }
            }
            Err(err) => {
                eprintln!("Error checking if queue is empty: {:?}", err);
            }
        }

        println!("Sleeping");
        tokio::time::sleep(tokio::time::Duration::from_secs(SECONDS_TO_SLEEP)).await;
    }
}

async fn is_queue_empty(
    api_url: &String,
) -> Result<(bool, responses::fetch_next_queue_item::SongQueueItem), reqwest::Error> {
    match api::fetch_next_queue_item(api_url).await {
        Ok(response) => {
            match response
                .json::<responses::fetch_next_queue_item::SongQueueItem>()
                .await
            {
                Ok(response) => {
                    if response.data.is_empty() {
                        Ok((true, response))
                    } else {
                        Ok((false, response))
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

async fn process_song(api_url: &String, song_queue_id: &uuid::Uuid) -> Result<(), reqwest::Error> {
    match api::fetch_song_queue_data::get_data(api_url, song_queue_id).await {
        Ok(response) => {
            // Process data here...
            match api::parsing::parse_response_into_bytes(response).await {
                Ok(song_bytes) => {
                    let (directory, filename) = generate_song_queue_dir_and_filename().await;
                    let song_queue_path = save_file_to_fs(&directory, &filename, &song_bytes).await;

                    println!("Saved at: {:?}", song_queue_path);

                    match api::get_metadata_queue::get(api_url, song_queue_id).await {
                        Ok(response) => {
                            match response
                                .json::<api::get_metadata_queue::response::Response>()
                                .await
                            {
                                Ok(response) => {
                                    let id = &response.data[0].id;
                                    let metadata = &response.data[0].metadata;
                                    let created_at = &response.data[0].created_at;
                                    println!("Id: {:?}", id);
                                    println!("Metadata: {:?}", metadata);
                                    println!("Created at: {:?}", created_at);

                                    println!("Getting coverart queue");
                                    match api::get_coverart_queue::get(api_url, song_queue_id).await
                                    {
                                        Ok(response) => {
                                            match response.json::<api::get_coverart_queue::response::Response>().await {
                                                Ok(response) => {
                                                    let coverart_queue_id = &response.data[0].id;
                                                    println!("Coverart queue Id: {:?}", coverart_queue_id);

                                                    match api::get_coverart_queue::get_data(api_url, coverart_queue_id).await {
                                                        Ok(response) => match api::parsing::parse_response_into_bytes(response).await {
                                                            Ok(coverart_queue_bytes) => {
                                                                let (directory, filename) = generate_coverart_queue_dir_and_filename().await;
                                                                let coverart_queue_path = save_file_to_fs(&directory, &filename, &coverart_queue_bytes).await;

                                                                println!("Saved coverart queue file at: {:?}", coverart_queue_path);

                                                                match apply_metadata(song_queue_path, coverart_queue_path, metadata).await {
                                                                    Ok(_) => {
                                                                        // TODO: Update the queued song with the updated queued song
                                                                        // TODO: Create song
                                                                        // TODO: Create coverart
                                                                        // TODO: Wipe data from queued song
                                                                        // TODO: Wipe data from queued coverart
                                                                    }
                                                                    Err(err) => {
                                                                        eprintln!("Error: {:?}", err);
                                                                    }
                                                                }
                                                            }
                                                            Err(err) => {
                                                                eprintln!("Error: {:?}", err);
                                                            }
                                                        }
                                                        Err(err) => {
                                                            eprintln!("Error: {:?}", err);
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    eprintln!("Error: {:?}", err);
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            eprintln!("Error: {:?}", err);
                                        }
                                    }
                                    Ok(())
                                }
                                Err(err) => {
                                    eprintln!("Error: {:?}", err);
                                    Err(err)
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error: {:?}", err);
                            Err(err)
                        }
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

// TODO: Consider having something like this in icarus_models
pub async fn generate_song_queue_dir_and_filename() -> (String, String) {
    let mut song = icarus_models::song::Song::default();
    song.filename = song.generate_filename(icarus_models::types::MusicTypes::FlacExtension, true);

    song.directory = icarus_envy::environment::get_root_directory().await;

    (song.directory, song.filename)
}

// TODO: Consider having something like this in icarus_models
pub async fn generate_coverart_queue_dir_and_filename() -> (String, String) {
    use rand::Rng;

    let mut filename: String = String::new();
    let filename_len = 10;

    let some_chars: String = String::from("abcdefghij0123456789");
    let mut rng = rand::rng();

    for _i in 0..filename_len {
        let random_number: i32 = rng.random_range(0..=19);
        let index = random_number as usize;
        let rando_char = some_chars.chars().nth(index);

        if let Some(c) = rando_char {
            filename.push(c);
        }
    }

    // TODO: Do not hard code the file extension
    filename += ".jpeg";

    // TODO: Consider separating song and coverart when saving to the filesystem
    let directory = icarus_envy::environment::get_root_directory().await;

    (directory, filename)
}

// TODO: Check to see if this is available in icarus_models
pub async fn save_file_to_fs(
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

pub async fn apply_metadata(
    song_queue_path: std::path::PathBuf,
    coverart_queue_path: std::path::PathBuf,
    metadata: &api::get_metadata_queue::response::Metadata,
) -> Result<bool, std::io::Error> {
    // Apply metadata fields
    let s_path = match song_queue_path.to_str() {
        Some(val) => String::from(val),
        None => String::new(),
    };

    if s_path.is_empty() {
        println!("Song queue path is empty");
        return Ok(false);
    }

    let types = icarus_meta::types::all_metadata_types();

    for t in types {
        match t {
            icarus_meta::types::Type::Album => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::AlbumArtist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album_artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Artist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Date => {
                // TODO: Do something about this discrepancy
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.year.to_string());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Disc => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc);
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Genre => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.genre.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Title => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.title.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Track => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track);
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::TrackCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track_count);
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::DiscCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc_count);
                match icarus_meta::meta::metadata::set_meta_value(t, &s_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
        }
    }

    // Apply coverart
    let c_path: String = match coverart_queue_path.to_str() {
        Some(val) => String::from(val),
        None => String::new(),
    };

    match icarus_meta::meta::coverart::contains_coverart(&s_path) {
        Ok((exists, size)) => {
            if exists {
                println!("Coverart exists: {:?} size", size);
                match icarus_meta::meta::coverart::remove_coverart(&s_path) {
                    Ok(_data) => {}
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            match icarus_meta::meta::coverart::set_coverart(&s_path, &c_path) {
                Ok(_data) => {
                    if _data.is_empty() {
                        println!("There was an issue");
                        Ok(false)
                    } else {
                        println!("Success in applying coverart to song");
                        Ok(true)
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
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

    pub mod parsing {
        use futures::StreamExt;

        pub async fn parse_response_into_bytes(
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
    }

    pub mod get_metadata_queue {
        pub async fn get(
            base_url: &String,
            song_queue_id: &uuid::Uuid,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::new();
            let endpoint = String::from("api/v2/song/metadata/queue");
            let api_url = format!("{}/{}", base_url, endpoint);
            client
                .get(api_url)
                .query(&[("song_queue_id", song_queue_id)])
                .send()
                .await
        }

        pub mod response {
            use serde::{Deserialize, Serialize};

            #[derive(Debug, Deserialize, Serialize)]
            pub struct Metadata {
                pub song_queue_id: uuid::Uuid,
                pub album: String,
                pub album_artist: String,
                pub artist: String,
                pub disc: i32,
                pub disc_count: i32,
                pub duration: i64,
                pub genre: String,
                pub title: String,
                pub track: i32,
                pub track_count: i32,
                pub year: i32,
            }

            #[derive(Debug, Deserialize, Serialize)]
            pub struct QueueItem {
                pub id: uuid::Uuid,
                pub metadata: Metadata,
                #[serde(with = "time::serde::rfc3339")]
                pub created_at: time::OffsetDateTime,
                pub song_queue_id: uuid::Uuid,
            }

            #[derive(Debug, Deserialize, Serialize)]
            pub struct Response {
                pub message: String,
                pub data: Vec<QueueItem>,
            }
        }
    }

    pub mod get_coverart_queue {
        pub async fn get(
            base_url: &String,
            song_queue_id: &uuid::Uuid,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::new();
            let endpoint = String::from("api/v2/coverart/queue");
            let api_url = format!("{}/{}", base_url, endpoint);
            client
                .get(api_url)
                .query(&[("song_queue_id", song_queue_id)])
                .send()
                .await
        }

        pub async fn get_data(
            base_url: &String,
            coverart_queue_id: &uuid::Uuid,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::new();
            let endpoint = String::from("api/v2/coverart/queue/data");
            let api_url = format!("{}/{}/{}", base_url, endpoint, coverart_queue_id);
            client.get(api_url).send().await
        }

        pub mod response {
            use serde::{Deserialize, Serialize};

            #[derive(Debug, Deserialize, Serialize)]
            pub struct CoverArtQueue {
                pub id: uuid::Uuid,
                pub song_queue_id: uuid::Uuid,
            }

            #[derive(Debug, Deserialize, Serialize)]
            pub struct Response {
                pub message: String,
                pub data: Vec<CoverArtQueue>,
            }
        }
    }
}
