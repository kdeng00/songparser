pub mod fetch_next_queue_item {
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
