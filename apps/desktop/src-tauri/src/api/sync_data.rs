use std::collections::HashMap;

use crate::{
    crud::{
        recipe::{delete_recipe, insert_recipe},
        recipes::get_recipes,
        schedule_cloud_id::{
            delete_schedule_cloud_id, get_cloud_schedule_ids_for_user, insert_schedule_cloud_id,
        },
        schedules::{delete_schedule, update_recipe_schedules},
        tag::delete_tag,
        tags::get_tags_with_cloud_ids,
        Downloadable, DownloadableWith, RemoteUpdatable, Updatable, Uploadable,
    },
    errors::StringifyError,
    img_proc::{convert_cloud_img_to_local, delete_recipe_img},
    macros::{run_tx, run_tx_with_error},
    types::{
        cloud_structs::{
            CloudScheduleWithIds, DownloadedRecipe, LastSyncedRecord, RecipeExistenceRecord,
        },
        db_params::{
            ExcludedRecipeIds, RecipeIds, UsernameAndUpdatedFilter, UsernameFilter,
            UsernameFilterWithImagesLibPath,
        },
        raw_db::{
            IntegerValue, RawSchedule, ScheduleCloudId, ScheduleFormDataList,
            ScheduleFormDataWithCloudId, ScheduleFormDataWithId, ScheduleId, ToRecipeFormData,
        },
        response_bodies::{Recipe, RecipeTag},
    },
    AppState,
};
use chrono::NaiveDateTime;
use sqlx::{query_file_as, Pool, Sqlite, Transaction};
use tauri::{AppHandle, Manager, State};

use super::auth::check_auth::get_username;

async fn get_local_id_from_cloud(
    db: &Pool<Sqlite>,
    recipe_id: &str,
    username: &str,
) -> Result<Option<i64>, Box<dyn std::error::Error>> {
    let result: Option<i64> =
        run_tx_with_error!(db, async |tx: &mut Transaction<'_, Sqlite>| -> Result<
            Option<i64>,
            Box<dyn std::error::Error>,
        > {
            let local_row_res =
                query_file_as!(IntegerValue, "db/get_local_id.sql", recipe_id, username)
                    .fetch_optional(&mut **tx)
                    .await;

            match local_row_res {
                Ok(opt_row) => Ok(opt_row.and_then(|r| r.value)),
                Err(e) => Err(Box::from(e)),
            }
        });
    Ok(result)
}

async fn update_local_recipe_from_downloaded(
    recipe: DownloadedRecipe,
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<Option<i64>, Box<dyn std::error::Error>> {
    let username = get_username(state).await?;
    let local_id_opt: Option<i64> =
        get_local_id_from_cloud(&state.db, &recipe.id.as_str(), &username).await?;

    let Some(local_id) = local_id_opt else {
        return Ok(None);
    };

    // If converting the image fails, treat that as an error so the caller can abort
    // and avoid updating last_synced when a local update did not complete.
    let local_image_path = match convert_cloud_img_to_local(
        &recipe.image_path,
        &recipe.id,
        app,
        &state.images_lib_path,
    )
    .await
    {
        Ok(opt) => opt,
        Err(e) => return Err(e),
    };

    run_tx_with_error!(
        &state.db,
        async |tx: &mut Transaction<'_, Sqlite>| -> Result<Option<i64>, Box<dyn std::error::Error>> {
            let mut local_recipe = recipe.into_local_recipe(local_id);
            local_recipe.image_path = local_image_path;
            delete_recipe_img(tx, local_id, &state.images_lib_path).await?;
            local_recipe.update(tx).await?;
            Ok(Some(local_id))
        }
    );

    Ok(Some(local_id))
}

async fn sync_recipes(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = &state.db;

    // Get username
    let username = get_username(&state).await?;

    // Get all local recipes before downloading
    let local_recipes = match get_recipes::<UsernameFilterWithImagesLibPath>(
        db,
        UsernameFilterWithImagesLibPath {
            username: &username,
            images_lib_path: &state.images_lib_path,
        },
    )
    .await
    {
        Ok(recipes) => recipes,
        Err(e) => return Err(e.to_string()),
    };

    // Download recipes that only exist on the server
    let recipe_ids = local_recipes
        .iter()
        .map(|r| r.cloud_parent_id.clone())
        .filter(|r| r.is_some())
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();
    let downloaded_recipes: Vec<DownloadedRecipe> = Vec::<DownloadedRecipe>::download_with(
        &app,
        ExcludedRecipeIds {
            excluded_recipe_ids: &recipe_ids,
        },
    )
    .await
    .map_err(|e: Box<dyn std::error::Error>| e.to_string())?;
    for recipe in downloaded_recipes {
        let username_option = Some(username.clone());
        let cloud_parent_id = recipe.id.clone();
        let local_image_path = convert_cloud_img_to_local(
            &recipe.image_path,
            &recipe.id,
            &app,
            &state.images_lib_path,
        )
        .await
        .map_err(|e| e.to_string())?;
        let mut recipe = recipe.into_form_data();
        recipe.image_path = local_image_path;
        recipe.cloud_parent_id = Some(cloud_parent_id);

        match insert_recipe(&state.db, &recipe, username_option).await {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }

    // Delete recipes locally that have a cloud_parent_id but that don't exist on the server
    let nonexistent_recipe_cloud_records =
        Vec::<RecipeExistenceRecord>::download_with(&app, RecipeIds { recipe_ids })
            .await
            .map_err(|e| e.to_string())?;
    let nonexistent_recipe_cloud_ids = nonexistent_recipe_cloud_records
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<String>>();
    let recipes_with_dead_cloud_parent: Vec<&Recipe> = local_recipes
        .iter()
        .filter(|r| {
            let cloud_parent_id = r.cloud_parent_id.as_ref();
            cloud_parent_id.is_some()
                && nonexistent_recipe_cloud_ids.contains(cloud_parent_id.unwrap_or(&String::new()))
        })
        .filter(|recipe| recipe.id.is_some())
        .collect();
    for recipe in recipes_with_dead_cloud_parent {
        delete_recipe(db, recipe, &state.images_lib_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Upload recipes that don't exist on the server (error protection)
    let no_cloud_parent: Vec<&Recipe> = local_recipes
        .iter()
        .filter(|r| r.cloud_parent_id.is_none())
        .filter(|r| r.id.is_some())
        .collect();
    for recipe in no_cloud_parent {
        let form_data = recipe
            .into_recipe_form_data()
            .into_local_recipe(recipe.id.unwrap());
        form_data.upload(&app).await.map_err(|e| e.to_string())?;
    }

    let username_ref = &username;

    // Update recipes on the server that have a newer version stored locally
    let last_synced_db = run_tx!(db, async |tx: &mut Transaction<'_, Sqlite>| {
        query_file_as!(LastSyncedRecord, "db/get_last_synced.sql", username_ref)
            .fetch_optional(&mut **tx)
            .await
    });
    let last_synced_datetime = match last_synced_db {
        Some(record) => record.last_synced.unwrap_or(
            NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default(),
        ),
        None => NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap_or_default(),
    };
    let updated_local_recipes: Vec<Recipe> = get_recipes::<UsernameAndUpdatedFilter<'_>>(
        db,
        UsernameAndUpdatedFilter {
            username: &username,
            updated_after: &last_synced_datetime,
            images_lib_path: &state.images_lib_path,
        },
    )
    .await
    .map_err(|e| e.to_string())?;
    for recipe in updated_local_recipes {
        let recipe_id_opt = recipe.id;
        if recipe_id_opt.is_none() {
            continue;
        }
        let recipe_id = recipe_id_opt.unwrap();
        let local_recipe = recipe.into_recipe_form_data().into_local_recipe(recipe_id);
        local_recipe
            .update_remote(&app)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Update local recipes that have a newer version stored on the server
    let downloaded_recipes = Vec::<DownloadedRecipe>::download_with(&app, last_synced_datetime)
        .await
        .map_err(|e| e.to_string())?;
    for recipe in downloaded_recipes {
        let _ = update_local_recipe_from_downloaded(recipe, &app, &state)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn sync_tags(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let db = &state.db;
    let username = get_username(&state).await?;
    let local_tags = get_tags_with_cloud_ids(
        db,
        UsernameFilter {
            username: &username,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    let downloaded_tags = Vec::<RecipeTag>::download(&app)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|tag| tag.name.is_some())
        .map(|tag| tag.name.clone().unwrap())
        .collect::<Vec<_>>();

    for tag in local_tags.iter() {
        if let Some(tag_name) = &tag.name {
            if !downloaded_tags.contains(tag_name) {
                delete_tag(db, tag).await.map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

async fn sync_schedules(app: AppHandle) -> Result<(), String> {
    // Download schedules where the server has a different version
    let state = app.state::<AppState>();
    let db = &state.db;
    let username = get_username(&state).await?;
    let downloaded_schedules = Vec::<CloudScheduleWithIds>::download(&app)
        .await
        .map_err(|e| e.to_string())?;
    let schedule_cloud_ids = get_cloud_schedule_ids_for_user(db, &username)
        .await
        .map_err(|e| e.to_string())?;

    // Group ALL downloaded schedules by recipe_id first
    let mut server_by_recipe: HashMap<i64, Vec<&CloudScheduleWithIds>> = HashMap::new();
    for schedule in downloaded_schedules.iter() {
        if let Some(local_recipe_id) = get_local_id_from_cloud(db, &schedule.recipe_id, &username)
            .await
            .map_err(|e| e.to_string())?
        {
            server_by_recipe
                .entry(local_recipe_id)
                .or_insert_with(Vec::new)
                .push(schedule);
        }
    }

    let mut to_insert: Vec<ScheduleFormDataList<ScheduleFormDataWithCloudId>> = Vec::new();

    for (local_recipe_id, server_schedules) in server_by_recipe.iter() {
        let list = server_schedules
            .iter()
            .map(|s| ScheduleFormDataWithCloudId {
                cloud_id: s.id.clone(),
                recipe_id: *local_recipe_id,
                date: s.date,
                repeat: s.repeat.clone(),
                end_repeat: s.end_repeat,
            })
            .collect();
        to_insert.push(ScheduleFormDataList {
            recipe_id: *local_recipe_id,
            list,
        });
    }

    for recipe_schedules in to_insert.iter() {
        let inserted_ids = update_recipe_schedules(db, &recipe_schedules)
            .await
            .map_err(|e| e.to_string())?;
        for (schedule, id) in recipe_schedules.list.iter().zip(inserted_ids) {
            let schedule_cloud_id = ScheduleCloudId {
                local_id: Some(id),
                cloud_id: Some(schedule.cloud_id.clone()),
                username: Some(username.clone()),
            };
            insert_schedule_cloud_id(db, &schedule_cloud_id)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // Delete local schedules that have a cloud parent defined but no real cloud parent
    let ids_in_cloud = downloaded_schedules
        .into_iter()
        .map(|s| s.id)
        .collect::<Vec<String>>();
    for cloud_id in schedule_cloud_ids
        .iter()
        .filter(|cid| cid.cloud_id.is_some())
    {
        if !ids_in_cloud.contains(&cloud_id.cloud_id.as_ref().unwrap()) {
            delete_schedule_cloud_id(db, cloud_id)
                .await
                .map_err(|e| e.to_string())?;
            delete_schedule(
                db,
                ScheduleId {
                    id: cloud_id.local_id.unwrap(),
                },
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    // Upload schedules that don't have a cloud parent
    let local_only_schedules = run_tx!(
        db,
        async |tx: &mut Transaction<'_, Sqlite>| -> Result<Vec<RawSchedule>, sqlx::Error> {
            sqlx::query_file_as!(RawSchedule, "db/get_schedules_with_no_cloud_parent.sql")
                .fetch_all(&mut **tx)
                .await
        }
    );

    let mut grouped: HashMap<i64, Vec<ScheduleFormDataWithId>> = HashMap::new();

    for schedule in local_only_schedules.iter() {
        grouped
            .entry(schedule.recipe_id)
            .or_insert_with(Vec::new)
            .push(ScheduleFormDataWithId {
                id: schedule.id,
                recipe_id: schedule.recipe_id,
                date: schedule.date,
                repeat: schedule.repeat.clone().unwrap_or(String::from("none")),
                end_repeat: schedule.end_repeat,
            });
    }

    let to_upload: Vec<ScheduleFormDataList<ScheduleFormDataWithId>> = grouped
        .into_iter()
        .map(|(recipe_id, list)| ScheduleFormDataList { recipe_id, list })
        .collect();

    for recipe_schedules in to_upload.iter() {
        recipe_schedules
            .clone()
            .try_update_remote(&app)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn sync_all(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let username = match get_username(&state).await {
        Ok(username) => username,
        Err(_) => return Ok(()),
    };
    sync_recipes(app.clone()).await?;
    sync_tags(app.clone()).await?;
    sync_schedules(app.clone()).await?;
    let db = &state.db;
    run_tx!(db, async |tx: &mut Transaction<'_, Sqlite>| {
        let username_ref = &username;
        sqlx::query_file!("db/update_last_synced.sql", username_ref)
            .execute(&mut **tx)
            .await?;
        Ok::<(), sqlx::Error>(())
    });
    Ok(())
}

#[tauri::command]
pub async fn sync_data(app: AppHandle) -> Result<(), String> {
    match sync_all(app).await {
        Ok(_) => Ok(()),
        Err(e) => Err({
            println!("Error syncing data: {}", e);
            format!("Error syncing data")
        }),
    }
}
