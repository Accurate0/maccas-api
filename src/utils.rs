use crypto::{digest::Digest, sha1::Sha1};
use std::{ffi::OsStr, path::Path};
use uuid::Uuid;

pub fn get_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

pub fn get_short_sha1(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(key);
    hasher.result_str()[..6].to_owned()
}

pub fn remove_ext(s: &str) -> &str {
    Path::new(s)
        .file_stem()
        .unwrap_or_else(|| OsStr::new(s))
        .to_str()
        .unwrap_or(s)
}
