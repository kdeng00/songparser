pub async fn some_work(
    app: &crate::config::App,
    song_queue_id: &uuid::Uuid,
    user_id: &uuid::Uuid,
) -> Result<
    (
        icarus_models::song::Song,
        icarus_models::coverart::CoverArt,
        crate::api::get_metadata_queue::response::Metadata,
        crate::queued_item::QueuedSong,
        crate::queued_item::QueuedCoverArt,
    ),
    std::io::Error,
> {
    match prep_song(app, song_queue_id).await {
        Ok((queued_song, queued_coverart, metadata)) => {
            println!("Prepping song");

            match crate::metadata::apply_metadata(&queued_song, &queued_coverart, &metadata).await {
                Ok(_applied) => {
                    match crate::api::update_queued_song::update_queued_song(app, &queued_song)
                        .await
                    {
                        Ok(response) => {
                            match response
                                .json::<crate::api::update_queued_song::response::Response>()
                                .await
                            {
                                Ok(_inner_response) => {
                                    println!("Updated queued song");
                                    println!("Response: {_inner_response:?}");

                                    let song_type = String::from(
                                        icarus_meta::detection::song::constants::FLAC_TYPE,
                                    );

                                    match crate::api::create_song::create(
                                        app, &metadata, user_id, &song_type,
                                    )
                                    .await
                                    {
                                        Ok(response) => match response
                                            .json::<crate::api::create_song::response::Response>()
                                            .await
                                        {
                                            Ok(resp) => {
                                                println!("Response: {resp:?}");

                                                let mut song = resp.data[0].clone();
                                                song.directory = queued_song.song.directory.clone();
                                                song.filename = queued_song.song.filename.clone();

                                                match crate::api::create_coverart::create(app, &song, &queued_coverart).await {
                                                    Ok(response) => match response.json::<crate::api::create_coverart::response::Response>().await {
                                                        Ok(resp) => {
                                                            println!("CoverArt sent and successfully parsed response");
                                                            println!("json: {resp:?}");
                                                            let mut coverart = resp.data[0].clone();
                                                            coverart.directory = queued_coverart.coverart.directory.clone();
                                                            coverart.filename = queued_coverart.coverart.filename.clone();

                                                            Ok((song.clone(), coverart.clone(), metadata, queued_song.clone(), queued_coverart.clone()))
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

pub async fn prep_song(
    app: &crate::config::App,
    song_queue_id: &uuid::Uuid,
) -> Result<
    (
        crate::queued_item::QueuedSong,
        crate::queued_item::QueuedCoverArt,
        crate::api::get_metadata_queue::response::Metadata,
    ),
    reqwest::Error,
> {
    match crate::api::fetch_song_queue_data::get_data(app, song_queue_id).await {
        Ok(response) => {
            // Process data here...
            match crate::api::parsing::parse_response_into_bytes(response).await {
                Ok(song_bytes) => {
                    let song = icarus_models::song::Song {
                        directory: icarus_envy::environment::get_root_directory().await.value,
                        filename: icarus_models::song::generate_filename(
                            icarus_models::types::MusicType::FlacExtension,
                            true,
                        )
                        .unwrap(),
                        data: song_bytes,
                        ..Default::default()
                    };
                    let songpath = song.song_path().unwrap_or_default();

                    let queued_song: crate::queued_item::QueuedSong =
                        match song.save_to_filesystem() {
                            Ok(_) => crate::queued_item::QueuedSong {
                                id: *song_queue_id,
                                song,
                                path: songpath,
                            },
                            Err(err) => {
                                eprintln!("Error: {err:?}");
                                crate::queued_item::QueuedSong {
                                    ..Default::default()
                                }
                            }
                        };

                    println!("Saved at: {:?}", queued_song.path);

                    match crate::api::get_metadata_queue::get(app, &queued_song.id).await {
                        Ok(response) => {
                            match response
                                .json::<crate::api::get_metadata_queue::response::Response>()
                                .await
                            {
                                Ok(response) => {
                                    let bod = &response.data[0];
                                    match process_coverart(app, &queued_song.id, bod).await {
                                        Ok(qc) => Ok((queued_song, qc, bod.metadata.clone())),
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

pub async fn process_coverart(
    app: &crate::config::App,
    queued_song_id: &uuid::Uuid,
    queued_item: &crate::api::get_metadata_queue::response::QueueItem,
) -> Result<crate::queued_item::QueuedCoverArt, reqwest::Error> {
    let id = queued_item.id;
    let created_at = queued_item.created_at;
    let metadata = &queued_item.metadata;
    println!("Id: {id:?}");
    println!("Metadata: {metadata:?}");
    println!("Created at: {created_at:?}");

    println!("Getting coverart queue");
    match crate::api::get_coverart_queue::get(app, queued_song_id).await {
        Ok(response) => {
            match response
                .json::<crate::api::get_coverart_queue::response::Response>()
                .await
            {
                Ok(response) => {
                    let coverart_queue = &response.data[0];
                    let coverart_queue_id = coverart_queue.id;
                    println!("Coverart queue Id: {coverart_queue_id:?}");

                    match crate::api::get_coverart_queue::get_data(app, &coverart_queue_id).await {
                        Ok(response) => {
                            match crate::api::parsing::parse_response_into_bytes(response).await {
                                Ok(coverart_queue_bytes) => {
                                    let queued_coverart = init_queued_coverart(
                                        &coverart_queue_id,
                                        &coverart_queue.file_type,
                                        coverart_queue_bytes,
                                    )
                                    .await;
                                    println!(
                                        "Saved coverart queue file at: {:?}",
                                        queued_coverart.path
                                    );
                                    println!(
                                        "Queued CoverArt file type: {:?}",
                                        queued_coverart.coverart.file_type
                                    );

                                    Ok(queued_coverart)
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

async fn init_queued_coverart(
    coverart_queue_id: &uuid::Uuid,
    file_type: &str,
    bytes: Vec<u8>,
) -> crate::queued_item::QueuedCoverArt {
    // TODO: Consider separating song and coverart when saving to the filesystem
    let covart_type = if file_type == icarus_meta::detection::coverart::constants::PNG_TYPE {
        icarus_models::types::CoverArtType::PngExtension
    } else if file_type == icarus_meta::detection::coverart::constants::JPEG_TYPE {
        icarus_models::types::CoverArtType::JpegExtension
    } else if file_type == icarus_meta::detection::coverart::constants::JPG_TYPE {
        icarus_models::types::CoverArtType::JpgExtension
    } else {
        icarus_models::types::CoverArtType::None
    };
    let coverart = icarus_models::coverart::CoverArt {
        directory: icarus_envy::environment::get_root_directory().await.value,
        filename: match icarus_models::coverart::generate_filename(covart_type, true) {
            Ok(filename) => filename,
            Err(err) => {
                eprintln!("Error generating CoverArt filename: {err:?}");
                panic!("Error initializing queued CoverArt");
            }
        },
        file_type: String::from(file_type),
        data: bytes,
        ..Default::default()
    };
    coverart.save_to_filesystem().unwrap();
    let coverart_queue_fs_path = coverart.get_path().unwrap();
    crate::queued_item::QueuedCoverArt {
        id: *coverart_queue_id,
        coverart,
        path: coverart_queue_fs_path,
    }
}

pub async fn cleanup(
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
