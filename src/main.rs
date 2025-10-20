pub mod api;
pub mod auth;
pub mod config;
pub mod metadata;
pub mod util;

pub const SECONDS_TO_SLEEP: u64 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = config::App {
        uri: icarus_envy::environment::get_icarus_base_api_url()
            .await
            .value,
        auth_uri: icarus_envy::environment::get_icarus_auth_base_api_url()
            .await
            .value,
        ..Default::default()
    };
    println!("Base URL: {:?}", app.uri);
    println!("Auth URL: {:?}", app.auth_uri);

    match auth::get_token(&app).await {
        Ok(login_result) => {
            app.token = login_result;
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(-1);
        }
    };

    loop {
        println!("Token: {:?}", app.token);

        if app.token.token_expired() {
            println!("Token expired");
            app.token = match auth::get_refresh_token(&app).await {
                Ok(login_result) => login_result,
                Err(err) => {
                    eprintln!("Error: {err:?}");
                    continue;
                }
            };

            println!("Token refreshed");
            println!("Refreshed token: {:?}", app.token);
        } else {
            println!("Token did not expire");
        }

        match is_queue_empty(&app).await {
            Ok((empty, song_queue_item)) => {
                if !empty {
                    println!("Queue is not empty");
                    println!("SongQueueItem: {song_queue_item:?}");

                    let song_queue_id = song_queue_item.data[0].id;
                    let user_id = song_queue_item.data[0].user_id;

                    match some_work(&app, &song_queue_id, &user_id).await {
                        Ok((
                            song,
                            coverart,
                            (song_queue_id, _song_queue_path),
                            (coverart_queue_id, _coverart_queue_path),
                        )) => {
                            match wipe_data_from_queues(&app, &song_queue_id, &coverart_queue_id)
                                .await
                            {
                                Ok(_) => match cleanup(&song, &coverart).await {
                                    Ok(_) => {
                                        println!("Successful cleanup");
                                    }
                                    Err(err) => {
                                        eprintln!("Error: {err:?}");
                                    }
                                },
                                Err(err) => {
                                    eprintln!("Error: {err:?}");
                                }
                            }
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
    app: &config::App,
    song_queue_id: &uuid::Uuid,
    coverart_queue_id: &uuid::Uuid,
) -> Result<(), std::io::Error> {
    match api::wipe_data::song_queue::wipe_data(app, song_queue_id).await {
        Ok(response) => match response
            .json::<api::wipe_data::song_queue::response::Response>()
            .await
        {
            Ok(_resp) => {
                match api::wipe_data::coverart_queue::wipe_data(app, coverart_queue_id).await {
                    Ok(inner_response) => match inner_response
                        .json::<api::wipe_data::coverart_queue::response::Response>()
                        .await
                    {
                        Ok(_inner_resp) => {
                            println!("Wiped data from CoverArt queue");
                            println!("Resp: {_inner_resp:?}");
                            Ok(())
                        }
                        Err(err) => Err(std::io::Error::other(err.to_string())),
                    },
                    Err(err) => Err(std::io::Error::other(err.to_string())),
                }
            }
            Err(err) => Err(std::io::Error::other(err.to_string())),
        },
        Err(err) => Err(std::io::Error::other(err.to_string())),
    }
}

async fn cleanup(
    song: &icarus_models::song::Song,
    coverart: &icarus_models::coverart::CoverArt,
) -> Result<(), std::io::Error> {
    match song.remove_from_filesystem() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: Problem cleaning up SongQueue files {err:?}");
        }
    }

    match coverart.remove_from_filesystem() {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

async fn is_queue_empty(
    app: &config::App,
) -> Result<(bool, api::fetch_next_queue_item::response::SongQueueItem), reqwest::Error> {
    match api::fetch_next_queue_item::fetch_next_queue_item(app).await {
        Ok(response) => {
            match response
                .json::<api::fetch_next_queue_item::response::SongQueueItem>()
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
    app: &crate::config::App,
    song_queue_id: &uuid::Uuid,
    user_id: &uuid::Uuid,
) -> Result<
    (
        icarus_models::song::Song,
        icarus_models::coverart::CoverArt,
        (uuid::Uuid, String),
        (uuid::Uuid, String),
    ),
    std::io::Error,
> {
    match prep_song(app, song_queue_id).await {
        Ok((
            (song_directory, song_filename),
            (coverart_directory, coverart_filename),
            metadata,
            coverart_queue_id,
        )) => {
            println!("Prepping song");

            let mut song_queue_path: String = String::new();
            let p = std::path::Path::new(&song_directory);
            let sp = p.join(&song_filename);
            song_queue_path.push_str(sp.to_str().unwrap_or_default());
            let coverart_queue = icarus_models::coverart::CoverArt {
                directory: coverart_directory,
                filename: coverart_filename,
                ..Default::default()
            };
            let coverart_queue_path = match coverart_queue.get_path() {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Could not get CoverArt path");
                    eprintln!("Error: {err:?}");
                    std::process::exit(-1);
                }
            };

            println!("CoverArt path: {coverart_queue_path:?}");

            match metadata::apply_metadata(&song_queue_path, &coverart_queue_path, &metadata).await
            {
                Ok(_applied) => {
                    match api::update_queued_song::update_queued_song(
                        app,
                        &song_queue_path,
                        song_queue_id,
                    )
                    .await
                    {
                        Ok(response) => {
                            match response
                                .json::<api::update_queued_song::response::Response>()
                                .await
                            {
                                Ok(_inner_response) => {
                                    println!("Updated queued song");
                                    println!("Response: {_inner_response:?}");

                                    // TODO: Place this somewhere else
                                    let song_type = String::from("flac");

                                    match api::create_song::create(
                                        app, &metadata, user_id, &song_type,
                                    )
                                    .await
                                    {
                                        Ok(response) => match response
                                            .json::<api::create_song::response::Response>()
                                            .await
                                        {
                                            Ok(resp) => {
                                                println!("Response: {resp:?}");

                                                let mut song = resp.data[0].clone();
                                                song.directory = song_directory;
                                                song.filename = song_filename;

                                                match api::create_coverart::create(app, &song.id, &coverart_queue_id).await {
                                                    Ok(response) => match response.json::<api::create_coverart::response::Response>().await {
                                                        Ok(resp) => {
                                                            println!("CoverArt sent and successfully parsed response");
                                                            println!("json: {resp:?}");
                                                            let mut coverart = resp.data[0].clone();
                                                            coverart.directory = coverart_queue.directory;
                                                            coverart.filename = coverart_queue.filename;

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
    app: &crate::config::App,
    song_queue_id: &uuid::Uuid,
) -> Result<
    (
        (String, String),
        (String, String),
        api::get_metadata_queue::response::Metadata,
        uuid::Uuid,
    ),
    reqwest::Error,
> {
    match api::fetch_song_queue_data::get_data(app, song_queue_id).await {
        Ok(response) => {
            // Process data here...
            match api::parsing::parse_response_into_bytes(response).await {
                Ok(song_bytes) => {
                    let (song_directory, song_filename) =
                        generate_song_queue_dir_and_filename().await;
                    let song = icarus_models::song::Song {
                        directory: song_directory,
                        filename: song_filename,
                        data: song_bytes,
                        ..Default::default()
                    };
                    let songpath = song.song_path().unwrap_or_default();
                    let song_queue_path = match song.save_to_filesystem() {
                        Ok(_) => std::path::Path::new(&songpath),
                        Err(_err) => std::path::Path::new(""),
                    };

                    println!("Saved at: {song_queue_path:?}");

                    match api::get_metadata_queue::get(app, song_queue_id).await {
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
                                    match api::get_coverart_queue::get(app, song_queue_id).await {
                                        Ok(response) => {
                                            match response.json::<api::get_coverart_queue::response::Response>().await {
                                                Ok(response) => {
                                                    let coverart_queue_id = &response.data[0].id;
                                                    println!("Coverart queue Id: {coverart_queue_id:?}");

                                                    match api::get_coverart_queue::get_data(app, coverart_queue_id).await {
                                                        Ok(response) => match api::parsing::parse_response_into_bytes(response).await {
                                                            Ok(coverart_queue_bytes) => {
                                                                let (directory, filename) = generate_coverart_queue_dir_and_filename().await;
                                                                let coverart = icarus_models::coverart::CoverArt {
                                                                    directory,
                                                                    filename,
                                                                    data: coverart_queue_bytes,
                                                                    ..Default::default()
                                                                };
                                                                coverart.save_to_filesystem().unwrap();
                                                                let coverart_queue_fs_path = match coverart.get_path() {
                                                                    Ok(path) => {
                                                                        path
                                                                    }
                                                                    Err(err) => {
                                                                        eprintln!("Error: {err:?}");
                                                                        std::process::exit(-1);
                                                                    }
                                                                };
                                                                let coverart_queue_path = std::path::Path::new(&coverart_queue_fs_path);
                                                                println!("Saved coverart queue file at: {coverart_queue_path:?}");

                                                                Ok(((song.directory, song.filename), (coverart.directory, coverart.filename), metadata.clone(), *coverart_queue_id))
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
    song.filename = icarus_models::song::generate_filename(
        icarus_models::types::MusicTypes::FlacExtension,
        true,
    );

    song.directory = icarus_envy::environment::get_root_directory().await.value;

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
    let directory = icarus_envy::environment::get_root_directory().await.value;

    (directory, filename)
}
