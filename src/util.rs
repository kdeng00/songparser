pub fn path_buf_to_string(path: &std::path::Path) -> String {
    match path.to_str() {
        Some(val) => String::from(val),
        None => String::new(),
    }
}
