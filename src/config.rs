#[derive(Default, Debug)]
pub struct App {
    pub uri: String,
    pub auth_uri: String,
    pub token: simodels::login_result::LoginResult,
    pub root_directory: String,
}

impl App {
    pub fn does_root_directory_exists(&self) -> bool {
        let path = std::path::Path::new(&self.root_directory);
        if path.exists() { path.is_dir() } else { false }
    }
}

pub async fn initialize_app_config() -> App {
    App {
        uri: sienvy::environment::get_soaricarus_base_api_url().value,
        auth_uri: sienvy::environment::get_soaricarus_auth_base_api_url().value,
        root_directory: sienvy::environment::get_root_directory().value,
        ..Default::default()
    }
}
