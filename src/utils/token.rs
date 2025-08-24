use actix_web::error::InternalError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, prelude::BASE64_STANDARD, DecodeError, Engine as _};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::{OsRng, RngCore};
use uuid::Uuid;
use crate::{db::postgres_service::PostgresService, types::token::TokenType};
use anyhow::{Context, Result as AResult};

pub fn new_id() -> Uuid {
    Uuid::new_v4()
}

pub fn new_token(tokentype: TokenType) -> String {
    let mut buf = [0u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut buf);
    format!("{}_{}", tokentype.to_string(), URL_SAFE_NO_PAD.encode(buf))
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

pub fn encrypt_to_base64(base_string: &str) -> String {
    BASE64_STANDARD.encode(base_string.as_bytes())
}

pub fn decrypt_from_base64(encoded: &str) -> AResult<String> {
    let bytes = BASE64_STANDARD.decode(encoded)?;
    let s = String::from_utf8(bytes)?;
    Ok(s)
}

pub async fn token_valid(db: &PostgresService, b64_token: &str) -> bool {
    println!("[+] starting token_valid with b64_token: {}", b64_token);

    let token = match decrypt_from_base64(b64_token) {
        Ok(token) => {
            println!("[+] successfully base64-decoded token: {}", token);
            token
        },
        Err(e) => {
            println!("[-] failed to decode base64 token: {:?}", e);
            return false;
        },
    };

    let mut parts = token.split('.');

    let id_str = match parts.nth(0) {
        Some(s) => {
            println!("[>] extracted id_str: {}", s);
            s
        },
        None => {
            println!("[-] token missing id part");
            return false;
        },
    };

    let id = match Uuid::parse_str(id_str) {
        Ok(id) => {
            println!("[+] parsed UUID: {}", id);
            id
        },
        Err(e) => {
            println!("[-] failed to parse id_str as UUID: {:?}", e);
            return false;
        },
    };

    let encrypted_token = match db.get_user_token(id).await {
        Ok(encrypted) => {
            println!("[+] retrieved encrypted_token for id {}", id);
            encrypted
        },
        Err(e) => {
            println!("[-] database lookup failed for id {}: {:?}", id, e);
            return false;
        },
    };

    let raw_token = match parts.nth(0) {
        Some(token) => {
            println!("[>] extracted raw_token: {}", token);
            token
        },
        None => {
            println!("[-] token missing raw part");
            return false;
        },
    };

    match verify(raw_token, &encrypted_token) {
        Ok(_) => {
            println!("[+] token verified successfully (match)");
            true
        },
        Err(e) => {
            println!("[-] token verification failed: {:?}", e);
            false
        },
    }
}

pub fn construct_token(user_id: &str, api_key: &str) -> String {
    return encrypt_to_base64(format!("{user_id}.{api_key}"))
}
