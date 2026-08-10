use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, FilePath};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use crate::api::auth::logout::logout;

/// Recursively add a directory's contents to a ZipWriter.
fn zip_dir(
    writer: &mut ZipWriter<File>,
    src_dir: &Path,
    prefix: &Path,
    options: FileOptions<()>,
) -> io::Result<()> {
    for entry in WalkDir::new(src_dir).min_depth(1).into_iter().flatten() {
        let path = entry.path();
        let relative = path
            .strip_prefix(prefix)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Normalise separators so the archive is portable.
        let zip_path = relative.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            writer.add_directory(&zip_path, options)?;
        } else {
            writer.start_file(&zip_path, options)?;
            let mut f = File::open(path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            writer.write_all(&buf)?;
        }
    }
    Ok(())
}

/// Zip the app-data directory and prompt the user where to save the `.ccar` file.
#[tauri::command]
pub async fn zip_data(app: AppHandle) -> Result<(), String> {
    // Resolve the app-data directory.
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data dir: {e}"))?;

    if !data_dir.exists() {
        return Err("App data directory does not exist.".into());
    }

    // Show a save dialog restricted to .ccar files.
    let save_path: Option<PathBuf> = app
        .dialog()
        .file()
        .set_title("Save backup")
        .add_filter("App Archive", &["ccar"])
        .set_file_name("cookycardzdata.ccar")
        .blocking_save_file()
        .map(|fp: FilePath| fp.into_path().ok())
        .flatten();

    let dest = match save_path {
        Some(p) => p,
        None => return Ok(()), // User cancelled – not an error.
    };

    // Ensure the destination has the correct extension.
    let dest = if dest.extension().map_or(true, |e| e != "ccar") {
        dest.with_extension("ccar")
    } else {
        dest
    };

    // Build the zip archive in memory, then write it to disk.
    let out_file =
        File::create(&dest).map_err(|e| format!("Could not create archive file: {e}"))?;

    let mut zip = ZipWriter::new(out_file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    zip_dir(&mut zip, &data_dir, &data_dir, options)
        .map_err(|e| format!("Failed to zip app data: {e}"))?;

    zip.finish()
        .map_err(|e| format!("Failed to finalise archive: {e}"))?;

    Ok(())
}

/// Open a `.ccar` file chosen by the user and extract it into the app-data
/// directory, replacing whatever is already there.
#[tauri::command]
pub async fn unzip_data(app: AppHandle) -> Result<(), String> {
    // Logout to prevent cloud conflicts
    logout(&app, &app.state()).await.map_err(|e| e.error)?;

    // Resolve the app-data directory.
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data dir: {e}"))?;

    // Show an open dialog restricted to .ccar files.
    let open_path: Option<PathBuf> = app
        .dialog()
        .file()
        .set_title("Restore backup")
        .add_filter("App Archive", &["ccar"])
        .blocking_pick_file()
        .map(|fp: FilePath| fp.into_path().ok())
        .flatten();

    let src = match open_path {
        Some(p) => p,
        None => return Ok(()), // User cancelled – not an error.
    };

    // Open and validate the archive before touching the data dir.
    let zip_file = File::open(&src).map_err(|e| format!("Could not open archive: {e}"))?;
    let mut archive = ZipArchive::new(zip_file).map_err(|e| format!("Invalid archive: {e}"))?;

    // Wipe the current contents of the app-data directory.
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir).map_err(|e| format!("Failed to clear app data dir: {e}"))?;
    }
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to recreate app data dir: {e}"))?;

    // Extract every entry from the archive.
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry {i}: {e}"))?;

        // Sanitise the path to prevent directory-traversal attacks.
        let entry_path = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe path in archive entry {i}"))?
            .to_owned();

        let out_path = data_dir.join(&entry_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create directory {out_path:?}: {e}"))?;
        } else {
            // Ensure parent directories exist.
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir {parent:?}: {e}"))?;
            }
            let mut out_file = File::create(&out_path)
                .map_err(|e| format!("Failed to create file {out_path:?}: {e}"))?;
            io::copy(&mut entry, &mut out_file)
                .map_err(|e| format!("Failed to write file {out_path:?}: {e}"))?;
        }
    }

    app.emit("import_complete", "Import Complete")
        .map_err(|e| e.to_string())?;

    // Relaunch the application so the backend reopens the database against the
    // newly imported files. Spawn a new process for the current executable and
    // exit the current process. If spawning fails, still return Ok so the UI
    // receives the import_complete event.
    app.request_restart();

    Ok(())
}
