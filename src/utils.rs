use crypto::{digest::Digest, sha1::Sha1};
use lazy_static::lazy_static;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::{collections::hash_map::DefaultHasher, ffi::OsStr, path::Path};
use uuid::Uuid;

pub fn get_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

lazy_static! {
    static ref HASHER: Mutex<DefaultHasher> = Mutex::new(DefaultHasher::new());
}

pub fn calculate_hash<T>(t: &T) -> u64
where
    T: Hash,
{
    let mut hasher = HASHER.lock().unwrap();
    t.hash(&mut *hasher);
    hasher.finish()
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
