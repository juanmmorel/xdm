use anyhow::{Result, anyhow};
use url::Url;

#[derive(Debug, Clone)]
pub struct HlsSegment {
    pub url: String,
    pub duration: f64,
    pub byte_range: Option<(u64, u64)>, // (offset, length)
}

#[derive(Debug, Clone)]
pub struct HlsPlaylist {
    pub segments: Vec<HlsSegment>,
    pub total_duration: f64,
    pub is_variant: bool,
}

pub struct HlsParser;

impl HlsParser {
    pub fn parse(manifest: &str, base_url: &str) -> Result<HlsPlaylist> {
        let base = Url::parse(base_url)?;
        let mut segments = Vec::new();
        let mut total_duration = 0.0;
        let mut current_duration = 0.0;
        let mut current_byte_range = None;
        let mut is_variant = false;
        let mut last_offset = 0;

        let lines = manifest.lines();
        let mut sig_found = false;

        for line in lines {
            let line = line.trim();
            if line.is_empty() { continue; }

            if !sig_found {
                if line == "#EXTM3U" {
                    sig_found = true;
                    continue;
                } else {
                    return Err(anyhow!("Invalid HLS manifest signature"));
                }
            }

            if line.starts_with("#EXT-X-STREAM-INF") {
                is_variant = true;
            } else if line.starts_with("#EXTINF:") {
                let parts: Vec<&str> = line["#EXTINF:".len()..].split(',').collect();
                if let Ok(d) = parts[0].parse::<f64>() {
                    current_duration = d;
                }
            } else if line.starts_with("#EXT-X-BYTERANGE:") {
                let val = &line["#EXT-X-BYTERANGE:".len()..];
                let parts: Vec<&str> = val.split('@').collect();
                let length = parts[0].parse::<u64>()?;
                let offset = if parts.len() == 2 {
                    parts[1].parse::<u64>()?
                } else {
                    last_offset
                };
                current_byte_range = Some((offset, length));
                last_offset = offset + length;
            } else if !line.starts_with('#') {
                let segment_url = if Url::parse(line).is_ok() {
                    line.to_string()
                } else {
                    base.join(line)?.to_string()
                };

                segments.push(HlsSegment {
                    url: segment_url,
                    duration: current_duration,
                    byte_range: current_byte_range.take(),
                });
                total_duration += current_duration;
            }
        }

        Ok(HlsPlaylist {
            segments,
            total_duration,
            is_variant,
        })
    }
}
