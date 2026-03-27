use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use sha2::{Sha256, Digest};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, AsyncReadExt};
use std::time::Duration;
use crate::parser::HlsParser;

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("XDM-Rust/0.1.0")
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub async fn download(&self, url: &str, filename: &str, segments: usize) -> Result<()> {
        if url.contains(".m3u8") {
            return self.download_hls(url, filename).await;
        }

        println!("Starting download of {} into {} with {} segments", url, filename, segments);

        let response = self.client.get(url).send().await?;
        let content_length = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("Failed to get content length"))?;

        println!("Content length: {} bytes", content_length);

        // Pre-allocate file
        {
            let file = tokio::fs::File::create(filename).await?;
            file.set_len(content_length).await?;
        }

        let segment_size = content_length / segments as u64;
        let mut handles = vec![];

        for i in 0..segments {
            let start = i as u64 * segment_size;
            let end = if i == segments - 1 {
                content_length - 1
            } else {
                (i as u64 + 1) * segment_size - 1
            };

            let client = self.client.clone();
            let url = url.to_string();
            let filename = filename.to_string();

            let handle = tokio::spawn(async move {
                Self::download_segment(client, url, filename, start, end).await
            });
            handles.push(handle);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await? {
                Ok(_) => println!("Segment {} finished", i),
                Err(e) => return Err(anyhow!("Segment {} failed: {}", i, e)),
            }
        }

        println!("Download complete. Validating checksum...");
        let checksum = Self::calculate_checksum(filename).await?;
        println!("SHA-256: {}", checksum);

        Ok(())
    }

    async fn download_hls(&self, url: &str, filename: &str) -> Result<()> {
        println!("Starting HLS download: {}", url);
        let response = self.client.get(url).send().await?.text().await?;
        let playlist = HlsParser::parse(&response, url)?;

        if playlist.is_variant {
            return Err(anyhow!("Variant master playlist detected. Automatic selection not implemented yet."));
        }

        let mut file = tokio::fs::File::create(filename).await?;

        for (i, segment) in playlist.segments.iter().enumerate() {
            println!("Downloading HLS segment {}/{}", i + 1, playlist.segments.len());
            let mut res = self.client.get(&segment.url).send().await?;

            if let Some((offset, length)) = segment.byte_range {
                let range = format!("bytes={}-{}", offset, offset + length - 1);
                res = self.client.get(&segment.url).header(RANGE, range).send().await?;
            }

            let mut stream = res.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
            }
        }

        println!("HLS Download complete: {}", filename);
        Ok(())
    }

    async fn download_segment(client: reqwest::Client, url: String, filename: String, start: u64, end: u64) -> Result<()> {
        let range = format!("bytes={}-{}", start, end);
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            let res = client.get(&url).header(RANGE, &range).send().await;
            match res {
                Ok(response) => {
                    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                        if retry_count < max_retries {
                            retry_count += 1;
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        return Err(anyhow!("Failed to download segment: {}", response.status()));
                    }

                    let mut file = OpenOptions::new()
                        .write(true)
                        .open(&filename)
                        .await?;

                    file.seek(std::io::SeekFrom::Start(start)).await?;

                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        file.write_all(&chunk).await?;
                    }
                    return Ok(());
                }
                Err(e) => {
                    if retry_count < max_retries {
                        retry_count += 1;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }

    async fn calculate_checksum(filename: &str) -> Result<String> {
        let mut file = tokio::fs::File::open(filename).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        while let Ok(n) = file.read(&mut buffer).await {
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}
