pub async fn fetch_next_queue_item(base_url: &String) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let fetch_endpoint = String::from("api/v2/song/queue/next");
    let api_url = format!("{base_url}/{fetch_endpoint}");
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
        let api_url = format!("{base_url}/{endpoint}/{id}");
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
        let api_url = format!("{base_url}/{endpoint}");
        client
            .get(api_url)
            .query(&[("song_queue_id", song_queue_id)])
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
        base_url: &String,
        song_queue_id: &uuid::Uuid,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();
        let endpoint = String::from("api/v2/coverart/queue");
        let api_url = format!("{base_url}/{endpoint}");
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
        let api_url = format!("{base_url}/{endpoint}/{coverart_queue_id}");
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
