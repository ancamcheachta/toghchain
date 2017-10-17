extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;
use std::env::current_dir;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_cwd_name() -> String {
    let cwd = current_dir().unwrap();
    
    cwd.iter()
        .next_back()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn epoch() -> String {
    format!("{}", SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs())
}
    
pub fn mini_hash() -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(&epoch());
    let unique: &str = &hasher.result_str()[0..5];
    unique.to_string()
}