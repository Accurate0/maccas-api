use std::{ffi::OsStr, path::Path};
use uuid::Uuid;

pub fn get_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

pub fn remove_extension(s: &str) -> &str {
    Path::new(s)
        .file_stem()
        .unwrap_or_else(|| OsStr::new(s))
        .to_str()
        .unwrap_or(s)
}
