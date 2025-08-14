#[derive(Default, Debug)]
pub struct App {
    pub uri: String,
    pub auth_uri: String,
    pub token: icarus_models::login_result::LoginResult,
}
