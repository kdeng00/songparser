pub mod fetch_next_queue_item {

    pub async fn fetch_next_queue_item(
        app: &crate::config::App,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let fetch_endpoint = String::from("api/v2/song/queue/next");
        let api_url = format!("{}/{fetch_endpoint}", app.uri);
        let (key, header) = super::auth_header(app).await;

        client.get(api_url).header(key, header).send().await
    }

    pub mod response {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct QueueItem {
            pub id: uuid::Uuid,
            pub filename: String,
            pub status: String,
            pub user_id: uuid::Uuid,
        }

        #[derive(Debug, Deserialize, Serialize)]
        pub struct SongQueueItem {
            pub message: String,
            pub data: Vec<QueueItem>,
        }
    }
}

pub async fn auth_header(
    app: &crate::config::App,
) -> (reqwest::header::HeaderName, reqwest::header::HeaderValue) {
    let bearer = format!("Bearer {}", app.token.token);
    let header_value = reqwest::header::HeaderValue::from_str(&bearer).unwrap();
    (reqwest::header::AUTHORIZATION, header_value)
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
        app: &crate::config::App,
        id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let endpoint = String::from("api/v2/song/queue");
        let api_url = format!("{}/{endpoint}/{id}", app.uri);
        let (key, header) = super::auth_header(app).await;
        client.get(api_url).header(key, header).send().await
    }
}

pub mod get_metadata_queue {
    pub async fn get(
        app: &crate::config::App,
        song_queue_id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let endpoint = String::from("api/v2/song/metadata/queue");
        let api_url = format!("{}/{endpoint}", app.uri);
        let (key, header) = super::auth_header(app).await;
        client
            .get(api_url)
            .query(&[("song_queue_id", song_queue_id)])
            .header(key, header)
            .send()
            .await
    }

    pub mod response {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Serialize)]
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
        app: &crate::config::App,
        song_queue_id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let endpoint = String::from("api/v2/coverart/queue");
        let api_url = format!("{}/{endpoint}", app.uri);
        let (key, header) = super::auth_header(app).await;
        client
            .get(api_url)
            .query(&[("song_queue_id", song_queue_id)])
            .header(key, header)
            .send()
            .await
    }

    pub async fn get_data(
        app: &crate::config::App,
        coverart_queue_id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let endpoint = String::from("api/v2/coverart/queue/data");
        let api_url = format!("{}/{endpoint}/{coverart_queue_id}", app.uri);
        let (key, header) = super::auth_header(app).await;
        client.get(api_url).header(key, header).send().await
    }

    pub mod response {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct CoverArtQueue {
            pub id: uuid::Uuid,
            pub file_type: String,
            pub song_queue_id: uuid::Uuid,
        }

        #[derive(Debug, Deserialize, Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<CoverArtQueue>,
        }
    }
}

pub mod service_token {
    pub mod response {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<icarus_models::login_result::LoginResult>,
        }
    }
}

pub mod refresh_token {
    pub mod response {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<icarus_models::login_result::LoginResult>,
        }
    }
}

pub mod update_queued_song {
    pub async fn update_queued_song(
        app: &crate::config::App,
        queued_song: &crate::queued_item::QueuedSong,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::builder().build()?;

        println!("Queued song path: {:?}", queued_song.path);

        // TODO: Make the filename random
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(std::fs::read(&queued_song.path).unwrap())
                .file_name("track01.flac"),
        );

        let url = format!("{}/api/v2/song/queue/{}", app.uri, queued_song.id);
        println!("Url: {url:?}");

        let (key, header) = crate::api::auth_header(app).await;
        let request = client.patch(url).multipart(form).header(key, header);

        let response = request.send().await?;

        Ok(response)
    }

    pub mod response {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<uuid::Uuid>,
        }
    }
}

pub mod create_song {
    pub async fn create(
        app: &crate::config::App,
        metadata_queue: &crate::api::get_metadata_queue::response::Metadata,
        user_id: &uuid::Uuid,
        song_type: &String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let payload = serde_json::json!(
        {
            "album": &metadata_queue.album,
            "album_artist": &metadata_queue.album_artist,
            "artist": &metadata_queue.artist,
            "disc": metadata_queue.disc,
            "disc_count": metadata_queue.disc_count,
            "duration": metadata_queue.duration,
            "genre": &metadata_queue.genre,
            "title": &metadata_queue.title,
            "track": metadata_queue.track,
            "track_count": metadata_queue.track_count,
            "date": metadata_queue.year.to_string(),
            "audio_type": &song_type,
            "user_id": &user_id,
            "song_queue_id": &metadata_queue.song_queue_id,
        }
        );

        let client = reqwest::Client::builder().build()?;

        let url = format!("{}/api/v2/song", app.uri);
        let (key, header) = crate::api::auth_header(app).await;

        let request = client.post(url).json(&payload).header(key, header);
        request.send().await
    }

    pub mod response {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<icarus_models::song::Song>,
        }
    }
}

pub mod create_coverart {
    pub async fn create(
        app: &crate::config::App,
        song: &icarus_models::song::Song,
        queued_coverart: &crate::queued_item::QueuedCoverArt,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::builder().build()?;
        let url = format!("{}/api/v2/coverart", app.uri);
        let payload = get_payload(&song.id, &queued_coverart.id);
        let (key, header) = crate::api::auth_header(app).await;
        let request = client.post(url).json(&payload).header(key, header);

        request.send().await
    }

    fn get_payload(song_id: &uuid::Uuid, coverart_queue_id: &uuid::Uuid) -> serde_json::Value {
        serde_json::json!({
            "song_id": &song_id,
            "coverart_queue_id": &coverart_queue_id,
        })
    }

    pub mod response {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<icarus_models::coverart::CoverArt>,
        }
    }
}

pub mod wipe_data {
    pub mod song_queue {
        pub async fn wipe_data(
            app: &crate::config::App,
            queued_song: &crate::queued_item::QueuedSong,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::builder().build()?;
            let url = format!("{}/api/v2/song/queue/data/wipe", app.uri);
            let payload = serde_json::json!({
                "song_queue_id": queued_song.id
            });
            let (key, header) = crate::api::auth_header(app).await;
            let request = client.patch(url).json(&payload).header(key, header);

            request.send().await
        }

        pub mod response {
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub struct Response {
                pub message: String,
                pub data: Vec<uuid::Uuid>,
            }
        }
    }
    pub mod coverart_queue {
        pub async fn wipe_data(
            app: &crate::config::App,
            queued_coverart: &crate::queued_item::QueuedCoverArt,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::builder().build()?;
            let url = format!("{}/api/v2/coverart/queue/data/wipe", app.uri);
            let payload = serde_json::json!({
                "coverart_queue_id": queued_coverart.id
            });
            let (key, header) = crate::api::auth_header(app).await;
            let request = client.patch(url).json(&payload).header(key, header);

            request.send().await
        }

        pub mod response {
            #[derive(Debug, serde::Deserialize, serde::Serialize)]
            pub struct Response {
                pub message: String,
                pub data: Vec<uuid::Uuid>,
            }
        }
    }
}
