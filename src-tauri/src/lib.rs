use futures_util::StreamExt;
use reqwest::Client;
use rfd::FileDialog;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use std::path::Path;
use std::{env, fs};
use std::{fs::File, io::Write, path::PathBuf};
use sysinfo::Disks;
use tauri::Emitter;
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct FileInfo {
    name: String,
    path: String,
    full_path: String,
    is_dir: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[allow(dead_code)]
struct SupabaseRecord {
    id: i64,
    distro: String,
    version: String,
    download_url: Option<String>,
    sha256_hash: String,
    checksum_url: Option<String>,
    is_lts: bool,
    size: Option<u64>,
}

#[derive(serde::Serialize)]
struct IsoResult {
    distro: String,
    version: String,
}

#[derive(serde::Deserialize)]
struct DistroRow {
    distro: String,
}

#[derive(serde::Serialize, Clone)]
struct ProgressPayload {
    downloaded: u64,
    total: u64,
}

#[tauri::command]
async fn list_isos(path: String) -> Result<Vec<FileInfo>, String> {
    let mut isos = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path_buf = entry.path();
        let metadata = entry.metadata().map_err(|e| e.to_string())?;

        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let is_dir = metadata.is_dir();

        if is_dir || name.to_lowercase().ends_with(".iso") {
            isos.push(FileInfo {
                name: name.clone(),
                path: name,
                full_path: path_buf.to_string_lossy().to_string(),
                is_dir,
            });
        }
    }

    Ok(isos)
}

#[tauri::command]
async fn download_iso(app: tauri::AppHandle, url: String, dest: String) -> Result<(), String> {
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let total_size = resp.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut content = resp.bytes_stream();
    let mut downloaded = 0;
    while let Some(chunk) = content.next().await {
        let data = chunk.map_err(|e| e.to_string())?;
        file.write_all(&data).map_err(|e| e.to_string())?;
        downloaded += data.len() as u64;
        let _ = app.emit(
            "download_progress",
            ProgressPayload {
                downloaded,
                total: total_size,
            },
        );
    }
    Ok(())
}

#[tauri::command]
async fn select_folder() -> Option<PathBuf> {
    FileDialog::new().pick_folder()
}

#[tauri::command]
async fn obtain_hash(path: String) -> Result<String, String> {
    let iso = File::open(&path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(iso);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024 * 1024];

    loop {
        let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let iso_hash = hasher.finalize();
    Ok(format!("{:x}", iso_hash))
}

#[tauri::command]
async fn delete_iso(path: String, isos_folder: String) -> Result<(), String> {
    let path_buf = Path::new(&path);

    if !path.starts_with(&isos_folder) {
        return Err("The specified file is not in the working directory".to_string());
    }

    if path_buf.extension().and_then(|s| s.to_str()) != Some("iso") {
        return Err("Only .iso files can be deleted".to_string());
    }

    if !path_buf.exists() {
        return Err("The specified file does not exist".to_string());
    }

    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn verify_hash(hash: String) -> Result<Option<IsoResult>, String> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .unwrap_or_default()
        .to_string();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .unwrap_or_default()
        .to_string();
    let tabla_supabase = "all_isos_versions";
    let columna = "sha256_hash";

    let url = format!(
        "{}/rest/v1/{}?{}=eq.{}",
        supabase_url, tabla_supabase, columna, hash
    );
    let client = Client::new();

    let res = client
        .get(&url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let records: Vec<SupabaseRecord> = res.json().await.map_err(|e| e.to_string())?;

        if !records.is_empty() {
            Ok(Some(IsoResult {
                distro: records[0].distro.clone(),
                version: records[0].version.clone(),
            }))
        } else {
            Ok(None)
        }
    } else {
        let error_text = res.text().await.unwrap_or_default();
        Err(format!("Error of Supabase: {}", error_text))
    }
}

#[tauri::command]
async fn fetch_unique_distros() -> Result<Vec<String>, String> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .unwrap_or_default()
        .to_string();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .unwrap_or_default()
        .to_string();
    let tabla_supabase = "vw_distinct_distros";

    let url = format!("{}/rest/v1/{}", supabase_url, tabla_supabase);
    let client = Client::new();

    let res = client
        .get(&url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let records: Vec<DistroRow> = res.json().await.map_err(|e| e.to_string())?;

        let unique_distros: Vec<String> = records.into_iter().map(|r| r.distro).collect();
        Ok(unique_distros)
    } else {
        let error_text = res.text().await.unwrap_or_default();
        Err(format!("Error of Supabase: {}", error_text))
    }
}

#[tauri::command]
async fn fetch_versions_for_distro(distro: String) -> Result<Vec<SupabaseRecord>, String> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .unwrap_or_default()
        .to_string();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .unwrap_or_default()
        .to_string();
    let tabla_supabase = "all_isos_versions";
    let safe_distro = distro.replace(" ", "%20");

    let url = format!(
        "{}/rest/v1/{}?select=*&distro=eq.{}",
        supabase_url, tabla_supabase, safe_distro
    );
    let client = Client::new();

    let res = client
        .get(&url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let records: Vec<SupabaseRecord> = res.json().await.map_err(|e| e.to_string())?;

        Ok(records)
    } else {
        let error_text: String = res.text().await.unwrap_or_default();
        Err(format!("Error of Supabase: {}", error_text))
    }
}

#[tauri::command]
async fn fetch_disk_space(path: String) -> Option<(u64, u64, u64, f64)> {
    let path = Path::new(&path);

    let disks = Disks::new_with_refreshed_list();

    let disk = disks
        .list()
        .iter()
        .filter(|d| {
            let mount = d.mount_point();
            path.starts_with(mount) || mount == Path::new("/")
        })
        .max_by_key(|d| d.mount_point().components().count());

    if let Some(disk) = disk {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);

        let used_percent = if total == 0 {
            0.0
        } else {
            (used as f64 / total as f64) * 100.0
        };

        let used_percent = (used_percent * 100.0).round() / 100.0;

        Some((total, used, available, used_percent))
    } else {
        None
    }
}

#[tauri::command]
async fn submit_suggestion(name: String, message: String) -> Result<(), String> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .unwrap_or_default()
        .to_string();
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .unwrap_or_default()
        .to_string();
    let bucket_name = "suggestions";
    let folder_name = "pending";
    let suggestion_id = Uuid::new_v4();
    let file_name = format!("{}_{}.txt", name, suggestion_id);
    let url = reqwest::Url::parse_with_params(
        &format!(
            "{}/storage/v1/object/{}/{}/{}",
            supabase_url, bucket_name, folder_name, file_name
        ),
        &[("name", file_name.as_str())],
    )
    .map_err(|e| e.to_string())?;
    let client = Client::new();
    let res = client
        .post(url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "text/plain")
        .body(message)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let error_text: String = res.text().await.unwrap_or_default();
        Err(format!("Error of Supabase: {}", error_text))
    }
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let env_content = include_str!("../../.env");
    dotenvy::from_read(env_content.as_bytes()).ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_isos,
            download_iso,
            select_folder,
            obtain_hash,
            delete_iso,
            verify_hash,
            fetch_unique_distros,
            fetch_versions_for_distro,
            fetch_disk_space,
            submit_suggestion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
