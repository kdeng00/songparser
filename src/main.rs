pub mod api;
pub mod responses;
pub mod the_rest;
pub mod update_queued_song;
pub mod util;

use std::io::Write;

pub const SECONDS_TO_SLEEP: u64 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_base_url = icarus_envy::environment::get_icarus_base_api_url().await;

    loop {
        println!("Base URL: {app_base_url}");

        match is_queue_empty(&app_base_url).await {
            Ok((empty, song_queue_item)) => {
                if !empty {
                    println!("Queue is not empty");
                    println!("SongQueueItem: {song_queue_item:?}");
                    let song_queue_id = song_queue_item.data[0].id;

                    // TODO: Do something with the result later
                    match some_work(&app_base_url, &song_queue_id).await {
                        Ok((
                            song,
                            coverart,
                            (song_queue_id, song_queue_path),
                            (coverart_queue_id, coverart_queue_path),
                        )) => {
                            // TODO: Wipe data from song and coverart queues
                            match wipe_data_from_queues(
                                &app_base_url,
                                &song_queue_id,
                                &coverart_queue_id,
                            )
                            .await
                            {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("Error: {err:?}");
                                }
                            }
                            // TODO: Cleanup files in local filesystem
                        }
                        Err(err) => {
                            eprintln!("Error: {err:?}");
                        }
                    }
                } else {
                    println!("Queue is empty");
                }
            }
            Err(err) => {
                eprintln!("Error checking if queue is empty: {err:?}");
            }
        }

        println!("Sleeping");
        tokio::time::sleep(tokio::time::Duration::from_secs(SECONDS_TO_SLEEP)).await;
    }
}

async fn wipe_data_from_queues(
    app_base_url: &String,
    song_queue_id: &uuid::Uuid,
    coverart_queue_id: &uuid::Uuid,
) -> Result<(), std::io::Error> {
    match the_rest::wipe_data::song_queue::wipe_data(app_base_url, song_queue_id).await {
        Ok(response) => match response
            .json::<the_rest::wipe_data::song_queue::response::Response>()
            .await
        {
            Ok(_resp) => {
                println!("Wiped data from song queue");
                Ok(())
            }
            Err(err) => Err(std::io::Error::other(err.to_string())),
        },
        Err(err) => Err(std::io::Error::other(err.to_string())),
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

async fn some_work(
    app_base_url: &String,
    song_queue_id: &uuid::Uuid,
) -> Result<
    (
        icarus_models::song::Song,
        icarus_models::coverart::CoverArt,
        (uuid::Uuid, String),
        (uuid::Uuid, String),
    ),
    std::io::Error,
> {
    match prep_song(app_base_url, song_queue_id).await {
        Ok((song_queue_path, coverart_queue_path, metadata, coverart_queue_id)) => {
            match apply_metadata(&song_queue_path, &coverart_queue_path, &metadata).await {
                Ok(_applied) => {
                    match update_queued_song::update_queued_song(
                        app_base_url,
                        &song_queue_path,
                        song_queue_id,
                    )
                    .await
                    {
                        Ok(response) => {
                            match response
                                .json::<update_queued_song::response::Response>()
                                .await
                            {
                                Ok(_inner_response) => {
                                    println!("Response: {_inner_response:?}");

                                    // TODO: Do not hard code this. Check if one of the existing
                                    // endpoints already have the user_id
                                    let user_id = uuid::Uuid::new_v4();
                                    // TODO: Place this somewhere else
                                    let song_type = String::from("flac");
                                    // Err(std::io::Error::other(err.to_string()))
                                    match the_rest::create_song::create(
                                        app_base_url,
                                        &metadata,
                                        &user_id,
                                        &song_type,
                                    )
                                    .await
                                    {
                                        Ok(response) => match response
                                            .json::<the_rest::create_song::response::Response>()
                                            .await
                                        {
                                            Ok(resp) => {
                                                println!("Response: {resp:?}");

                                                let song = &resp.data[0];
                                                match the_rest::create_coverart::create(app_base_url, &song.id, &coverart_queue_id).await {
                                                    Ok(response) => match response.json::<the_rest::create_coverart::response::Response>().await {
                                                        Ok(resp) => {
                                                            println!("CoverArt sent and successfully parsed response");
                                                            println!("json: {resp:?}");
                                                            let coverart = &resp.data[0];
                                                            Ok((song.clone(), coverart.clone(), (metadata.song_queue_id, song_queue_path), (coverart_queue_id, coverart_queue_path)))
                                                        }
                                                        Err(err) => {
                                                            Err(std::io::Error::other(err.to_string()))
                                                        }
                                                    }
                                                    Err(err) => {
                                                        Err(std::io::Error::other(err.to_string()))
                                                    }
                                                }
                                            }
                                            Err(err) => Err(std::io::Error::other(err.to_string())),
                                        },
                                        Err(err) => Err(std::io::Error::other(err.to_string())),
                                    }
                                }
                                Err(err) => Err(std::io::Error::other(err.to_string())),
                            }
                        }
                        Err(err) => Err(std::io::Error::other(err.to_string())),
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(std::io::Error::other(err.to_string())),
    }
}

async fn prep_song(
    api_url: &String,
    song_queue_id: &uuid::Uuid,
) -> Result<
    (
        String,
        String,
        api::get_metadata_queue::response::Metadata,
        uuid::Uuid,
    ),
    reqwest::Error,
> {
    match api::fetch_song_queue_data::get_data(api_url, song_queue_id).await {
        Ok(response) => {
            // Process data here...
            match api::parsing::parse_response_into_bytes(response).await {
                Ok(song_bytes) => {
                    let (directory, filename) = generate_song_queue_dir_and_filename().await;
                    let song_queue_path = save_file_to_fs(&directory, &filename, &song_bytes).await;

                    println!("Saved at: {song_queue_path:?}");

                    match api::get_metadata_queue::get(api_url, song_queue_id).await {
                        Ok(response) => {
                            match response
                                .json::<api::get_metadata_queue::response::Response>()
                                .await
                            {
                                Ok(response) => {
                                    let id = &response.data[0].id;
                                    let created_at = &response.data[0].created_at;
                                    let metadata = &response.data[0].metadata;
                                    println!("Id: {id:?}");
                                    println!("Metadata: {metadata:?}");
                                    println!("Created at: {created_at:?}");

                                    println!("Getting coverart queue");
                                    match api::get_coverart_queue::get(api_url, song_queue_id).await
                                    {
                                        Ok(response) => {
                                            match response.json::<api::get_coverart_queue::response::Response>().await {
                                                Ok(response) => {
                                                    let coverart_queue_id = &response.data[0].id;
                                                    println!("Coverart queue Id: {coverart_queue_id:?}");

                                                    match api::get_coverart_queue::get_data(api_url, coverart_queue_id).await {
                                                        Ok(response) => match api::parsing::parse_response_into_bytes(response).await {
                                                            Ok(coverart_queue_bytes) => {
                                                                let (directory, filename) = generate_coverart_queue_dir_and_filename().await;
                                                                let coverart_queue_path = save_file_to_fs(&directory, &filename, &coverart_queue_bytes).await;

                                                                println!("Saved coverart queue file at: {coverart_queue_path:?}");

                                                                let c_path = util::path_buf_to_string(&coverart_queue_path);
                                                                let s_path = util::path_buf_to_string(&song_queue_path);
                                                                Ok((s_path, c_path, metadata.clone(), *coverart_queue_id))
                                                            }
                                                            Err(err) => {
                                                                Err(err)
                                                            }
                                                        }
                                                        Err(err) => {
                                                            Err(err)
                                                        }
                                                    }
                                                }
                                                Err(err) => {
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
                        Err(err) => Err(err),
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
    song_queue_path: &String,
    coverart_queue_path: &String,
    metadata: &api::get_metadata_queue::response::Metadata,
) -> Result<bool, std::io::Error> {
    // Apply metadata fields
    let types = icarus_meta::types::all_metadata_types();

    for t in types {
        match t {
            icarus_meta::types::Type::Album => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::AlbumArtist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album_artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Artist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
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
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Disc => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc);
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Genre => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.genre.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Title => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.title.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Track => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track);
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::TrackCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track_count);
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::DiscCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc_count);
                match icarus_meta::meta::metadata::set_meta_value(t, song_queue_path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
        }
    }

    // Apply coverart
    match icarus_meta::meta::coverart::contains_coverart(song_queue_path) {
        Ok((exists, size)) => {
            if exists {
                println!("Coverart exists: {size:?} size");
                match icarus_meta::meta::coverart::remove_coverart(song_queue_path) {
                    Ok(_data) => {}
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            match icarus_meta::meta::coverart::set_coverart(song_queue_path, coverart_queue_path) {
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
