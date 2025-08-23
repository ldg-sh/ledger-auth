use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::{OsRng, RngCore};
use uuid::Uuid;

pub fn new_id() -> Uuid {
    Uuid::new_v4()
}

// imagine manually importing things


pub fn new_token() -> String {
    let mut buf = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut buf);
    format!("tok_{}", URL_SAFE_NO_PAD.encode(buf))
}

pub fn encrypt(token: &str) -> Result<String, argon2::password_hash::Error> {
    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);
    let hash = Argon2::default().hash_password(token.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify(token: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(token.as_bytes(), &parsed).is_ok())
}

// Okay thats not bad
// you ARGONna make this good?
