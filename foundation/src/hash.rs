use crypto::{digest::Digest, sha1::Sha1};
use lazy_static::lazy_static;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

lazy_static! {
    static ref HASHER: Mutex<DefaultHasher> = Mutex::new(DefaultHasher::new());
    static ref SHA1_HASHER: Mutex<Sha1> = Mutex::new(Sha1::new());
}

pub fn calculate_default_hash<T>(t: &T) -> u64
where
    T: Hash,
{
    let mut hasher = HASHER.lock().unwrap();
    t.hash(&mut *hasher);
    hasher.finish()
}

pub fn get_short_sha1(key: &str) -> String {
    let mut hasher = SHA1_HASHER.lock().unwrap();
    hasher.input_str(key);
    let output = hasher.result_str()[..6].to_owned();
    hasher.reset();

    output
}

pub fn get_sha1(key: &str) -> String {
    let mut hasher = SHA1_HASHER.lock().unwrap();
    hasher.input_str(key);
    let output = hasher.result_str();
    hasher.reset();

    output
}
