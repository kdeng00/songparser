pub mod api;
pub mod auth;
pub mod config;
pub mod metadata;
pub mod parser;
pub mod queue;
pub mod queued_item;
pub mod util;

pub const SECONDS_TO_SLEEP: u64 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = config::initialize_app_config().await;
    println!("Base URL: {:?}", app.uri);
    println!("Auth URL: {:?}", app.auth_uri);

    if !app.does_root_directory_exists() {
        eprintln!("Root directory does not exist");
        println!("Attempting to create directory");
        let path = std::path::Path::new(&app.root_directory);
        match std::fs::create_dir(path) {
            Ok(_) => {
                println!("Directory created");
            }
            Err(err) => {
                eprintln!("Error creating directory: {err:?}");
                std::process::exit(-1);
            }
        }
    }

    app.token = match auth::get_token(&app).await {
        Ok(login_result) => login_result,
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(-1);
        }
    };

    loop {
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

        match queue::is_queue_empty(&app).await {
            Ok((empty, song_queue_item)) => {
                if !empty {
                    println!("Queue is not empty");
                    println!("SongQueueItem: {song_queue_item:?}");

                    let song_queue_id = song_queue_item.data[0].id;
                    let user_id = song_queue_item.data[0].user_id;

                    match parser::some_work(&app, &song_queue_id, &user_id).await {
                        Ok((song, coverart, _metadata, queued_song, queued_coverart)) => {
                            match queue::wipe_data_from_queues(&app, &queued_song, &queued_coverart)
                                .await
                            {
                                Ok(_) => match parser::cleanup(&song, &coverart).await {
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
