use std::{ffi::OsStr, path::Path};

pub fn remove_extension(s: &str) -> &str {
    Path::new(s)
        .file_stem()
        .unwrap_or_else(|| OsStr::new(s))
        .to_str()
        .unwrap_or(s)
}
