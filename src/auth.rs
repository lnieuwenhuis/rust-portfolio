use std::{collections::HashMap, sync::Arc};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::{HeaderMap, header};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::RwLock;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const SESSION_COOKIE: &str = "lars_portfolio_session";
const LOGIN_CSRF_COOKIE: &str = "lars_portfolio_login_csrf";
const SESSION_MAX_AGE_SECONDS: i64 = 60 * 60 * 24 * 7;

#[derive(Clone, Debug)]
pub struct SessionData {
    pub username: String,
    pub csrf_token: String,
}

#[derive(Clone, Default)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create(&self, username: impl Into<String>) -> (String, SessionData) {
        let id = random_token();
        let data = SessionData {
            username: username.into(),
            csrf_token: random_token(),
        };
        self.inner.write().await.insert(id.clone(), data.clone());
        (id, data)
    }

    pub async fn get(&self, id: &str) -> Option<SessionData> {
        self.inner.read().await.get(id).cloned()
    }

    pub async fn remove(&self, id: &str) {
        self.inner.write().await.remove(id);
    }
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn random_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub fn session_cookie(secret: &str, session_id: &str, secure: bool) -> String {
    let signature = sign(secret, session_id);
    build_cookie(
        SESSION_COOKIE,
        &format!("{session_id}.{signature}"),
        SESSION_MAX_AGE_SECONDS,
        "/",
        secure,
        true,
    )
}

pub fn expire_session_cookie(secure: bool) -> String {
    build_cookie(SESSION_COOKIE, "", 0, "/", secure, true)
}

pub fn login_csrf_cookie(token: &str, secure: bool) -> String {
    build_cookie(LOGIN_CSRF_COOKIE, token, 600, "/admin/login", secure, true)
}

pub fn expire_login_csrf_cookie(secure: bool) -> String {
    build_cookie(LOGIN_CSRF_COOKIE, "", 0, "/admin/login", secure, true)
}

pub fn session_id_from_headers(headers: &HeaderMap, secret: &str) -> Option<String> {
    let raw = cookie_value(headers, SESSION_COOKIE)?;
    let (session_id, signature) = raw.split_once('.')?;
    verify_signature(secret, session_id, signature).then(|| session_id.to_string())
}

pub fn login_csrf_from_headers(headers: &HeaderMap) -> Option<String> {
    cookie_value(headers, LOGIN_CSRF_COOKIE)
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(cookie_name, value)| (cookie_name == name).then(|| value.to_string()))
}

fn build_cookie(
    name: &str,
    value: &str,
    max_age: i64,
    path: &str,
    secure: bool,
    http_only: bool,
) -> String {
    let mut cookie = format!("{name}={value}; Path={path}; Max-Age={max_age}; SameSite=Lax");
    if http_only {
        cookie.push_str("; HttpOnly");
    }
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

fn sign(secret: &str, value: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(value.as_bytes());
    URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}

fn verify_signature(secret: &str, value: &str, signature: &str) -> bool {
    let Ok(signature) = URL_SAFE_NO_PAD.decode(signature.as_bytes()) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(value.as_bytes());
    mac.verify_slice(&signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{hash_password, session_cookie, session_id_from_headers, verify_password};
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn password_hashes_verify_only_matching_passwords() {
        let hash = hash_password("correct horse").expect("hash should be generated");

        assert!(verify_password(&hash, "correct horse"));
        assert!(!verify_password(&hash, "wrong password"));
    }

    #[test]
    fn signed_session_cookie_round_trips() {
        let secret = "0123456789012345678901234567890123456789012345678901234567890123";
        let cookie = session_cookie(secret, "session-id", false);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie.split(';').next().unwrap()).unwrap(),
        );

        assert_eq!(
            session_id_from_headers(&headers, secret).as_deref(),
            Some("session-id")
        );
    }
}
