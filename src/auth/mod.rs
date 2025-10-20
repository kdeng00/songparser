pub async fn get_token(
    app: &crate::config::App,
) -> Result<icarus_models::login_result::LoginResult, std::io::Error> {
    let client = reqwest::Client::new();
    let endpoint = String::from("api/v2/service/login");
    let api_url = format!("{}/{endpoint}", app.auth_uri);

    let payload = serde_json::json!({
        "passphrase": icarus_envy::environment::get_service_passphrase().await.value,
    });

    match client.post(api_url).json(&payload).send().await {
        Ok(response) => match response
            .json::<crate::api::service_token::response::Response>()
            .await
        {
            Ok(resp) => {
                if resp.data.is_empty() {
                    Err(std::io::Error::other(String::from("No token returned")))
                } else {
                    Ok(resp.data[0].clone())
                }
            }
            Err(err) => Err(std::io::Error::other(err.to_string())),
        },
        Err(err) => Err(std::io::Error::other(err.to_string())),
    }
}

pub async fn get_refresh_token(
    app: &crate::config::App,
) -> Result<icarus_models::login_result::LoginResult, std::io::Error> {
    let client = reqwest::Client::new();
    let endpoint = String::from("api/v2/token/refresh");
    let api_url = format!("{}/{endpoint}", app.auth_uri);

    let payload = serde_json::json!({
        "access_token": app.token.token
    });

    match client.post(api_url).json(&payload).send().await {
        Ok(response) => match response
            .json::<crate::api::refresh_token::response::Response>()
            .await
        {
            Ok(resp) => {
                if resp.data.is_empty() {
                    Err(std::io::Error::other(String::from("No token returned")))
                } else {
                    Ok(resp.data[0].clone())
                }
            }
            Err(err) => Err(std::io::Error::other(err.to_string())),
        },
        Err(err) => Err(std::io::Error::other(err.to_string())),
    }
}
