use std::sync::OnceLock;

use jsonwebtoken::{DecodingKey, EncodingKey};
use sha2::{Sha256, Digest};
use base64ct::{Base64Bcrypt, Encoding};
use serde::{Serialize, Deserialize};

pub fn do_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Base64Bcrypt::encode_string(&hasher.finalize())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
    pub sub: String,
    pub exp: usize,
}

static SECRET: &'static [u8] = std::include_bytes!("../rissue.secret");

static DECODE_KEY: OnceLock<DecodingKey> = OnceLock::new();

pub fn decode_key() -> &'static DecodingKey {
    DECODE_KEY.get_or_init(|| {
        DecodingKey::from_secret(SECRET)
    })
}

static ENCODE_KEY: OnceLock<EncodingKey> = OnceLock::new();

pub fn encode_key() -> &'static EncodingKey {
    ENCODE_KEY.get_or_init(|| {
        EncodingKey::from_secret(SECRET)
    })
}
