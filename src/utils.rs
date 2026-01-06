use regex::Regex;
use std::sync::OnceLock;

pub static THUMB_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn get_thumb_regex() -> &'static Regex {
    THUMB_REGEX.get_or_init(|| {
        Regex::new(r"^/wiki/thumb/(archive/)?([0-9a-f]/[0-9a-f][0-9a-f])/([^/]+)/([0-9]+)px-(.*)$").unwrap()
    })
}

#[derive(Debug)]
pub struct ThumbRequest {
    pub archive_prefix: Option<String>,
    pub hash_path: String,
    pub original_filename: String,
    pub width: u32,
    pub target_filename: String,
}

pub fn parse_thumb_path(path: &str) -> Option<ThumbRequest> {
    let re = get_thumb_regex();
    let caps = re.captures(path)?;

    Some(ThumbRequest {
        archive_prefix: caps.get(1).map(|m| m.as_str().to_string()),
        hash_path: caps.get(2).unwrap().as_str().to_string(),
        original_filename: caps.get(3).unwrap().as_str().to_string(),
        width: caps.get(4).unwrap().as_str().parse().ok()?,
        target_filename: caps.get(5).unwrap().as_str().to_string(),
    })
}
