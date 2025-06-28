pub async fn update_queued_song(
    base_url: &String,
    song_path: &String,
    song_queue_id: &uuid::Uuid,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::builder().build()?;

    println!("Song path: {:?}", song_path);

    // TODO: Make the filename random
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(std::fs::read(song_path).unwrap())
            .file_name("track01.flac"),
    );

    let url = format!("{}/api/v2/song/queue/{}", base_url, song_queue_id);
    println!("Url: {:?}", url);

    let request = client.patch(url).multipart(form);

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
