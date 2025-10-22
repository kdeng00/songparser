pub fn path_buf_to_string(path: &std::path::Path) -> String {
    match path.to_str() {
        Some(val) => String::from(val),
        None => String::new(),
    }
}

// TODO: Consider having something like this in icarus_models
pub async fn generate_coverart_queue_dir_and_filename(file_type: &str) -> (String, String) {
    use rand::Rng;

    let mut filename: String = String::new();
    let filename_len = 10;

    let some_chars: String = String::from("abcdefghij0123456789");
    let mut rng = rand::rng();

    for _ in 0..filename_len {
        let random_number: i32 = rng.random_range(0..=19);
        let index = random_number as usize;
        let rando_char = some_chars.chars().nth(index);

        if let Some(c) = rando_char {
            filename.push(c);
        }
    }

    filename += if file_type == icarus_meta::detection::coverart::constants::JPEG_TYPE
        || file_type == icarus_meta::detection::coverart::constants::JPG_TYPE
    {
        icarus_models::constants::file_extensions::image::JPEGEXTENSION
    } else if file_type == icarus_meta::detection::coverart::constants::PNG_TYPE {
        icarus_models::constants::file_extensions::image::PNGEXTENSION
    } else {
        ""
    };

    // TODO: Consider separating song and coverart when saving to the filesystem
    let directory = icarus_envy::environment::get_root_directory().await.value;

    (directory, filename)
}
