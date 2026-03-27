mod downloader;
mod parser;

use warp::Filter;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::downloader::Downloader;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    enabled: bool,
    #[serde(rename = "fileExts")]
    file_exts: Vec<String>,
    #[serde(rename = "blockedHosts")]
    blocked_hosts: Vec<String>,
    #[serde(rename = "requestFileExts")]
    request_file_exts: Vec<String>,
    #[serde(rename = "mediaTypes")]
    media_types: Vec<String>,
    #[serde(rename = "tabsWatcher")]
    tabs_watcher: Vec<String>,
    #[serde(rename = "videoList")]
    video_list: Vec<VideoInfo>,
    #[serde(rename = "matchingHosts")]
    matching_hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VideoInfo {
    id: String,
    text: String,
    info: String,
    #[serde(rename = "tabId")]
    tab_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadMessage {
    url: String,
    filename: Option<String>,
    cookie: Option<String>,
    #[serde(rename = "requestHeaders")]
    request_headers: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "responseHeaders")]
    response_headers: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "fileSize")]
    file_size: Option<u64>,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    segments: Option<usize>,
}

type AppState = Arc<Mutex<Config>>;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(Config {
        enabled: true,
        file_exts: vec!["MKV".to_string(), "MP4".to_string(), "ZIP".to_string()],
        blocked_hosts: vec![],
        request_file_exts: vec!["MP4".to_string(), "MP3".to_string()],
        media_types: vec!["audio/".to_string(), "video/".to_string()],
        tabs_watcher: vec![".youtube.".to_string(), "/watch?v=".to_string()],
        video_list: vec![],
        matching_hosts: vec!["googlevideo".to_string()],
    }));

    let state_filter = warp::any().map(move || state.clone());
    let downloader = Arc::new(Downloader::new());
    let downloader_filter = warp::any().map(move || downloader.clone());

    let sync_route = warp::path("sync")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(handle_sync);

    let download_route = warp::path("download")
        .and(warp::post())
        .and(warp::body::json())
        .and(downloader_filter)
        .and_then(handle_download);

    let routes = sync_route.or(download_route);

    println!("XDM Rust Server starting on 127.0.0.1:8597");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8597))
        .await;
}

async fn handle_sync(state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    let config = state.lock().await;
    Ok(warp::reply::json(&*config))
}

async fn handle_download(msg: DownloadMessage, downloader: Arc<Downloader>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received download request: {:?}", msg);

    let url = msg.url.clone();
    let filename = msg.filename.clone().unwrap_or_else(|| "downloaded_file".to_string());
    let segments = msg.segments.unwrap_or(32);

    tokio::spawn(async move {
        match downloader.download(&url, &filename, segments).await {
            Ok(_) => println!("Download complete: {}", filename),
            Err(e) => eprintln!("Download failed: {}", e),
        }
    });

    Ok(warp::reply::with_status("Download queued", warp::http::StatusCode::ACCEPTED))
}
