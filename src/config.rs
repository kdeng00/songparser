#[derive(Default, Debug)]
pub struct App {
    pub uri: String,
    pub auth_uri: String,
    pub token: icarus_models::login_result::LoginResult,
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
        uri: icarus_envy::environment::get_icarus_base_api_url()
            .await
            .value,
        auth_uri: icarus_envy::environment::get_icarus_auth_base_api_url()
            .await
            .value,
        root_directory: icarus_envy::environment::get_root_directory().await.value,
        ..Default::default()
    }
}
