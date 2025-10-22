/// Applies metadata to the queued song
pub async fn apply_metadata(
    queued_song: &crate::queued_item::QueuedSong,
    queued_coverart: &crate::queued_item::QueuedCoverArt,
    metadata: &crate::api::get_metadata_queue::response::Metadata,
) -> Result<bool, std::io::Error> {
    // Apply metadata fields
    let types = icarus_meta::types::all_metadata_types();

    for t in types {
        match t {
            icarus_meta::types::Type::Album => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::AlbumArtist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.album_artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Artist => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.artist.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Date => {
                // TODO: Do something about this discrepancy
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.year.to_string());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Disc => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc);
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Genre => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.genre.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Title => {
                let meta_type =
                    icarus_meta::types::MetadataType::from_string(metadata.title.clone());
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::Track => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track);
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::TrackCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.track_count);
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
            icarus_meta::types::Type::DiscCount => {
                let meta_type = icarus_meta::types::MetadataType::from_int(metadata.disc_count);
                match icarus_meta::meta::metadata::set_meta_value(t, &queued_song.path, meta_type) {
                    Ok(_) => {}
                    Err(_err) => {
                        return Err(_err);
                    }
                }
            }
        }
    }

    // Apply coverart
    match icarus_meta::meta::coverart::contains_coverart(&queued_song.path) {
        Ok((exists, size)) => {
            if exists {
                println!("Coverart exists: {size:?} size");
                match icarus_meta::meta::coverart::remove_coverart(&queued_song.path) {
                    Ok(_data) => {}
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            match icarus_meta::meta::coverart::set_coverart(
                &queued_song.path,
                &queued_coverart.path,
            ) {
                Ok(_data) => {
                    if _data.is_empty() {
                        println!("There was an issue");
                        Ok(false)
                    } else {
                        println!("Success in applying coverart to song");
                        Ok(true)
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
