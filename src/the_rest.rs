// TODO: Refactor this file when this app is functional

pub mod create_song {
    pub async fn create(
        base_url: &String,
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

        let url = format!("{base_url}/api/v2/song");

        let request = client.post(url).json(&payload);
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
        base_url: &String,
        song_id: &uuid::Uuid,
        coverart_queue_id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::builder().build()?;
        let url = format!("{base_url}/api/v2/coverart");
        let payload = get_payload(song_id, coverart_queue_id);
        let request = client.post(url).json(&payload);

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
            base_url: &String,
            song_queue_id: &uuid::Uuid,
        ) -> Result<reqwest::Response, reqwest::Error> {
            let client = reqwest::Client::builder().build()?;
            let url = format!("{base_url}/api/v2/song/queue/data/wipe");
            let payload = serde_json::json!({
                "song_queue_id": song_queue_id
            });
            let request = client.patch(url).json(&payload);

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
    // TODO: Wipe data from queued coverart
}
