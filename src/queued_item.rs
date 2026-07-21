#[derive(Clone, Debug, Default)]
pub struct QueuedSong {
    pub id: uuid::Uuid,
    pub song: simodels::song::Song,
    pub path: String,
}

#[derive(Clone, Debug, Default)]
pub struct QueuedCoverArt {
    pub id: uuid::Uuid,
    pub coverart: simodels::coverart::CoverArt,
    pub path: String,
}
