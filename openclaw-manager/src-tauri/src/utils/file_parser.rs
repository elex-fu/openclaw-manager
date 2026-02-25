use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown file format")]
    UnknownFormat,
    #[error("Invalid filename")]
    InvalidFilename,
}

/// Parse file name and extract structured information
pub fn parse_file_name(file_name: &str) -> Result<Value, ParseError> {
    let ext = file_name
        .rsplitn(2, '.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let base_name = file_name
        .rsplitn(2, '.')
        .nth(1)
        .unwrap_or(file_name);

    let result = match ext.as_str() {
        "mp4" | "avi" | "mkv" | "mov" | "wmv" => parse_video_filename(base_name),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => parse_image_filename(base_name),
        "mp3" | "wav" | "flac" | "aac" | "ogg" => parse_audio_filename(base_name),
        "txt" | "md" | "doc" | "docx" | "pdf" => parse_document_filename(base_name),
        _ => parse_generic_filename(base_name),
    };

    Ok(json!({
        "original_name": file_name,
        "extension": ext,
        "base_name": base_name,
        "parsed": result,
    }))
}

fn parse_video_filename(base_name: &str) -> Value {
    // Common patterns in video filenames
    let mut info = HashMap::new();

    // Resolution patterns
    let resolution_re = Regex::new(r"(?i)(\d{3,4})[pP]|(\d+)x(\d+)").unwrap();
    if let Some(cap) = resolution_re.captures(base_name) {
        let resolution = cap.get(1)
            .map(|m| format!("{}p", m.as_str()))
            .or_else(|| cap.get(2).map(|m| format!("{}x{}", m.as_str(), cap.get(3).unwrap().as_str())));
        if let Some(res) = resolution {
            info.insert("resolution", res);
        }
    }

    // Date patterns (YYYY-MM-DD, YYYYMMDD)
    let date_re = Regex::new(r"(\d{4})[-_]?(\d{2})[-_]?(\d{2})").unwrap();
    if let Some(cap) = date_re.captures(base_name) {
        info.insert("date", format!("{}-{}-{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
            cap.get(3).unwrap().as_str()
        ));
    }

    // Episode/Season patterns
    let episode_re = Regex::new(r"(?i)[Ss](\d+)[Ee](\d+)").unwrap();
    if let Some(cap) = episode_re.captures(base_name) {
        info.insert("season", cap.get(1).unwrap().as_str().to_string());
        info.insert("episode", cap.get(2).unwrap().as_str().to_string());
    }

    // Quality/source indicators
    let quality_keywords = vec!["HDR", "SDR", "BluRay", "WEB-DL", "HDTV", "DVD", "Remux"];
    for keyword in quality_keywords {
        if base_name.to_uppercase().contains(&keyword.to_uppercase()) {
            info.insert("quality", keyword.to_string());
            break;
        }
    }

    // Codec indicators
    let codec_keywords = vec!["H264", "H265", "HEVC", "AV1", "VP9", "MPEG"];
    for codec in codec_keywords {
        if base_name.to_uppercase().contains(&codec.to_uppercase()) {
            info.insert("codec", codec.to_string());
            break;
        }
    }

    json!(info)
}

fn parse_image_filename(base_name: &str) -> Value {
    let mut info = HashMap::new();

    // Date patterns
    let date_re = Regex::new(r"(\d{4})[-_]?(\d{2})[-_]?(\d{2})").unwrap();
    if let Some(cap) = date_re.captures(base_name) {
        info.insert("date", format!("{}-{}-{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
            cap.get(3).unwrap().as_str()
        ));
    }

    // Camera/Phone patterns (IMG_, DSC_, etc.)
    let camera_re = Regex::new(r"(?i)^(IMG|DSC|IMG_|DSC_)[-_]?(\d+)").unwrap();
    if let Some(cap) = camera_re.captures(base_name) {
        info.insert("camera_prefix", cap.get(1).unwrap().as_str().to_string());
        info.insert("camera_number", cap.get(2).unwrap().as_str().to_string());
    }

    // Screenshot patterns
    if base_name.to_lowercase().contains("screenshot")
        || base_name.to_lowercase().contains("screen shot")
        || base_name.starts_with("Screenshot") {
        info.insert("is_screenshot", "true".to_string());
    }

    json!(info)
}

fn parse_audio_filename(base_name: &str) -> Value {
    let mut info = HashMap::new();

    // Artist - Title patterns
    let artist_title_re = Regex::new(r"^(.+?)\s*[-–—]\s*(.+)$").unwrap();
    if let Some(cap) = artist_title_re.captures(base_name) {
        info.insert("artist", cap.get(1).unwrap().as_str().trim().to_string());
        info.insert("title", cap.get(2).unwrap().as_str().trim().to_string());
    }

    // Track number patterns
    let track_re = Regex::new(r"^(\d+)[\s.-]+(.+)$").unwrap();
    if let Some(cap) = track_re.captures(base_name) {
        info.insert("track_number", cap.get(1).unwrap().as_str().to_string());
    }

    json!(info)
}

fn parse_document_filename(base_name: &str) -> Value {
    let mut info = HashMap::new();

    // Version patterns (v1, v2, v1.0, etc.)
    let version_re = Regex::new(r"(?i)[vV](\d+(?:\.\d+)?)").unwrap();
    if let Some(cap) = version_re.captures(base_name) {
        info.insert("version", cap.get(1).unwrap().as_str().to_string());
    }

    // Date patterns
    let date_re = Regex::new(r"(\d{4})[-_]?(\d{2})[-_]?(\d{2})").unwrap();
    if let Some(cap) = date_re.captures(base_name) {
        info.insert("date", format!("{}-{}-{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
            cap.get(3).unwrap().as_str()
        ));
    }

    // Draft/Final indicators
    if base_name.to_lowercase().contains("draft") {
        info.insert("is_draft", "true".to_string());
    }
    if base_name.to_lowercase().contains("final") {
        info.insert("is_final", "true".to_string());
    }

    json!(info)
}

fn parse_generic_filename(base_name: &str) -> Value {
    let mut info = HashMap::new();

    // Date patterns
    let date_re = Regex::new(r"(\d{4})[-_]?(\d{2})[-_]?(\d{2})").unwrap();
    if let Some(cap) = date_re.captures(base_name) {
        info.insert("date", format!("{}-{}-{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
            cap.get(3).unwrap().as_str()
        ));
    }

    json!(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_video_filename() {
        let result = parse_file_name("Movie.2024.1080p.BluRay.x264.mp4").unwrap();
        assert!(result["parsed"]["resolution"].as_str().is_some());
    }

    #[test]
    fn test_parse_image_filename() {
        let result = parse_file_name("IMG_20240225_123456.jpg").unwrap();
        assert!(result["parsed"]["camera_prefix"].as_str().is_some());
    }

    #[test]
    fn test_parse_audio_filename() {
        let result = parse_file_name("Artist Name - Song Title.mp3").unwrap();
        assert!(result["parsed"]["artist"].as_str().is_some());
        assert!(result["parsed"]["title"].as_str().is_some());
    }
}
