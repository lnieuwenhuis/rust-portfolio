use std::{env, net::SocketAddr};

use anyhow::{Context, Result, bail};

pub const CONTACT_EMAIL: &str = "lnieuwenhuis48@icloud.com";

// Keep these empty until real public profiles are ready.
pub const PUBLIC_GITHUB_URL: &str = "";
pub const PUBLIC_LINKEDIN_URL: &str = "";

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub admin_username: String,
    pub admin_password_hash: String,
    pub session_secret: String,
    pub base_url: String,
    pub rust_log: String,
    pub port: u16,
    pub secure_cookies: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = required_env("DATABASE_URL")?;
        let admin_username = required_env("ADMIN_USERNAME")?;
        let admin_password_hash = required_env("ADMIN_PASSWORD_HASH")?;
        let session_secret = required_env("SESSION_SECRET")?;
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("PORT must be a valid TCP port")?;

        if session_secret.len() < 32 {
            bail!("SESSION_SECRET must be at least 32 characters; 64+ is recommended");
        }

        let railway_environment = env::var("RAILWAY_ENVIRONMENT").is_ok()
            || env::var("RAILWAY_PUBLIC_DOMAIN").is_ok()
            || env::var("RAILWAY_PRIVATE_DOMAIN").is_ok();

        Ok(Self {
            database_url,
            admin_username,
            admin_password_hash,
            session_secret,
            secure_cookies: base_url.starts_with("https://") || railway_environment,
            base_url: base_url.trim_end_matches('/').to_string(),
            rust_log,
            port,
        })
    }

    pub fn bind_addr(&self) -> Result<SocketAddr> {
        format!("0.0.0.0:{}", self.port)
            .parse()
            .context("failed to build bind address")
    }
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}
