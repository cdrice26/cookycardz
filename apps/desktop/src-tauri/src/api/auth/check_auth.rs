use crate::api::GenericResponse;
use crate::errors::StringifyError;
use crate::request::refresh_access_token;
use crate::AppState;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckAuthResponse {
    pub profile: Profile,
}

pub async fn make_auth_request(access_token: &str) -> Result<Response, String> {
    let client = reqwest::Client::new();
    client
        .get(format!("{}/auth/checkAuth", env!("API_URL")))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .string_err()
}

pub async fn parse_response(
    response: Response,
) -> Result<GenericResponse<CheckAuthResponse>, String> {
    let data: GenericResponse<CheckAuthResponse> = response.json().await.string_err()?;
    Ok(data)
}

pub async fn check_auth(
    state: &State<'_, AppState>,
) -> Result<GenericResponse<CheckAuthResponse>, String> {
    let needs_refresh = {
        let access_token = state.access_token.lock().await;
        access_token.is_none()
    };

    if needs_refresh {
        refresh_access_token(&state).await?;
    }

    // PHASE 2: Read the (possibly refreshed) token
    let token = {
        let access_token = state.access_token.lock().await;
        access_token.clone().ok_or("Not logged in".to_string())?
    };

    // PHASE 3: Make the authenticated request
    let mut response = make_auth_request(&token).await?;

    // PHASE 4: If unauthorized, try refreshing once
    if response.status().is_client_error() {
        refresh_access_token(&state).await?;

        // Read the updated token
        let token = {
            let access_token = state.access_token.lock().await;
            access_token.clone().ok_or("Not logged in".to_string())?
        };

        response = make_auth_request(&token).await?;
    }

    // PHASE 5: Final response handling
    if response.status().is_success() {
        parse_response(response).await
    } else {
        let mut access_token_mutex = state.access_token.lock().await;
        *access_token_mutex = None;
        Err("Not authenticated".to_string())
    }
}

pub async fn get_username(state: &State<'_, AppState>) -> Result<String, String> {
    Ok(check_auth(&state).await?.data.profile.username)
}

#[tauri::command]
pub async fn api_auth_check_auth(
    state: State<'_, AppState>,
) -> Result<GenericResponse<CheckAuthResponse>, String> {
    check_auth(&state).await
}
