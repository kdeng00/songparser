#[derive(Clone, Debug, Default)]
pub struct QueuedSong {
    pub id: uuid::Uuid,
    pub song: icarus_models::song::Song,
    pub path: String,
}

#[derive(Clone, Debug, Default)]
pub struct QueuedCoverArt {
    pub id: uuid::Uuid,
    pub coverart: icarus_models::coverart::CoverArt,
    pub path: String,
}
