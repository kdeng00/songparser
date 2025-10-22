pub async fn wipe_data_from_queues(
    app: &crate::config::App,
    queued_song: &crate::queued_item::QueuedSong,
    queued_coverart: &crate::queued_item::QueuedCoverArt,
) -> Result<(), std::io::Error> {
    match crate::api::wipe_data::song_queue::wipe_data(app, queued_song).await {
        Ok(response) => match response
            .json::<crate::api::wipe_data::song_queue::response::Response>()
            .await
        {
            Ok(_resp) => {
                match crate::api::wipe_data::coverart_queue::wipe_data(app, queued_coverart).await {
                    Ok(inner_response) => match inner_response
                        .json::<crate::api::wipe_data::coverart_queue::response::Response>()
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

pub async fn is_queue_empty(
    app: &crate::config::App,
) -> Result<
    (
        bool,
        crate::api::fetch_next_queue_item::response::SongQueueItem,
    ),
    reqwest::Error,
> {
    match crate::api::fetch_next_queue_item::fetch_next_queue_item(app).await {
        Ok(response) => {
            match response
                .json::<crate::api::fetch_next_queue_item::response::SongQueueItem>()
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
