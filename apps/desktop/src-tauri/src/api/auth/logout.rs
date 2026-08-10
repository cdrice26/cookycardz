use crate::{
    api::{ErrorResponse, SuccessResponse},
    errors::StringifyError,
    token_keyring::clear_refresh_token,
    AppState,
};
use tauri::{AppHandle, Emitter, State};

pub async fn logout(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<SuccessResponse, ErrorResponse> {
    let mut access_token_mutex = state.access_token.lock().await;
    *access_token_mutex = None;
    clear_refresh_token().string_err()?;
    app.emit("logout", "Logged out")
        .map_err(|e| e.to_string())?;
    Ok(SuccessResponse::new("Logout successful".to_string()))
}

#[tauri::command]
pub async fn api_auth_logout(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SuccessResponse, ErrorResponse> {
    logout(&app, &state).await
}
