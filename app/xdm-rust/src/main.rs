use warp::Filter;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs::File;
use std::io::copy;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    enabled: bool,
    file_exts: Vec<String>,
    blocked_hosts: Vec<String>,
    request_file_exts: Vec<String>,
    media_types: Vec<String>,
    tabs_watcher: Vec<String>,
    video_list: Vec<VideoInfo>,
    matching_hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VideoInfo {
    id: String,
    text: String,
    info: String,
    tab_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadMessage {
    url: String,
    filename: Option<String>,
    cookie: Option<String>,
    request_headers: Option<HashMap<String, Vec<String>>>,
    response_headers: Option<HashMap<String, Vec<String>>>,
    file_size: Option<u64>,
    mime_type: Option<String>,
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

    let sync_route = warp::path("sync")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(handle_sync);

    let download_route = warp::path("download")
        .and(warp::post())
        .and(warp::body::json())
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

async fn handle_download(msg: DownloadMessage) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Received download request: {:?}", msg);

    let url = msg.url.clone();
    let filename = msg.filename.clone().unwrap_or_else(|| "downloaded_file".to_string());

    tokio::spawn(async move {
        match download_file(&url, &filename).await {
            Ok(_) => println!("Download complete: {}", filename),
            Err(e) => eprintln!("Download failed: {}", e),
        }
    });

    Ok(warp::reply::with_status("Download queued", warp::http::StatusCode::ACCEPTED))
}

async fn download_file(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::get(url).await?;
    let mut dest = File::create(filename)?;
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    Ok(())
}
