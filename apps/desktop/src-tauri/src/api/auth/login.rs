use crate::api::auth::check_auth::{make_auth_request, parse_response};
use crate::api::sync_data::sync_all;
use crate::api::{ErrorResponse, SuccessResponse};
use crate::macros::run_tx_with_error;
use crate::{errors::StringifyError, token_keyring, AppState};
use reqwest::Response;
use serde::Deserialize;
use sqlx::{Pool, Sqlite, Transaction};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tokens {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    data: Tokens,
}

struct MaybeUsername {
    username: Option<String>,
}

fn contains_only<T: PartialEq>(v: &[T], x: &T) -> bool {
    v.iter().all(|item| item == x)
}

async fn get_all_usernames(db: &Pool<Sqlite>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(run_tx_with_error!(db, async |tx: &mut Transaction<
        '_,
        Sqlite,
    >| {
        let usernames = sqlx::query_file_as!(MaybeUsername, "db/get_usernames.sql")
            .fetch_all(&mut **tx)
            .await?
            .into_iter()
            .filter_map(|x| x.username)
            .collect();
        Ok::<Vec<std::string::String>, Box<dyn std::error::Error>>(usernames)
    }))
}

async fn delete_all_local_data(db: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    Ok(run_tx_with_error!(db, async |tx: &mut Transaction<
        '_,
        Sqlite,
    >| {
        sqlx::query_file!("db/delete_local_data.sql")
            .execute(&mut **tx)
            .await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }))
}

#[tauri::command]
pub async fn api_auth_login(
    app: AppHandle,
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<SuccessResponse, ErrorResponse> {
    let usernames = match get_all_usernames(&state.db).await {
        Ok(u) => u,
        Err(_) => {
            return Err(ErrorResponse::new(String::from(
                "Login failed due to local database issues.",
            )));
        }
    };
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/auth/login", env!("API_URL")))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .string_err()?;

    if response.status().is_success() {
        let rjson: LoginResponse = response.json().await.string_err()?;
        let access_token = rjson.data.access_token;
        let username_response: Response = make_auth_request(&access_token).await?;
        if username_response.status().is_success() {
            let username: String = parse_response(username_response)
                .await?
                .data
                .profile
                .username;
            let conflicts = !contains_only::<String>(&usernames, &username);
            if conflicts {
                let first_username = &usernames[0];
                let should_wipe = app.dialog().message(format!("You have recipes on this computer from another CookyCardz account: {first_username}.
                    To access those recipes again, sign back into that account. If you want to sign in with this account instead, click below to
                    clear the local recipes on this computer. As long as they are synced, they can be redownloaded by signing back into the previous
                    account. If you wish to save a local copy of the previous account's recipes, go to More > Export All Recipes To Archive."))
                    .title("Account Conflict")
                    .buttons(MessageDialogButtons::OkCancelCustom(String::from("Delete Local Data and Sign In"), String::from("Cancel")))
                    .blocking_show();
                if should_wipe {
                    let _ = match delete_all_local_data(&state.db).await {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("{:?}", e);
                            return Err(ErrorResponse::new(
                                "Couldn't clear local data for sign-in".to_string(),
                            ));
                        }
                    };
                } else {
                    return Err(ErrorResponse::new("Login cancelled".to_string()));
                }
            }
            let mut access_token_mutex = state.access_token.lock().await;
            *access_token_mutex = Some(access_token);
            token_keyring::store_refresh_token(rjson.data.refresh_token.as_str()).string_err()?;
            tauri::async_runtime::spawn(async move {
                match sync_all(app.clone()).await {
                    Ok(_) => app.emit("sync_success", "Sync complete!"),
                    Err(_) => app.emit("sync_error", "Failed to sync data"),
                }
            });
            Ok(SuccessResponse::new("Login successful".to_string()))
        } else {
            Err(ErrorResponse::new("Login failed".to_string()))
        }
    } else {
        Err(ErrorResponse::new("Login failed".to_string()))
    }
}
