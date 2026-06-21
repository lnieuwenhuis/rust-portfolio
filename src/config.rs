use std::{env, net::SocketAddr};

use anyhow::{Context, Result, bail};
use url::Url;

pub const CONTACT_EMAIL: &str = "lnieuwenhuis48@icloud.com";

// Keep these empty until real public profiles are ready.
pub static PUBLIC_GITHUB_URL: &str = "";
pub static PUBLIC_LINKEDIN_URL: &str = "";

pub fn has_public_github_url() -> bool {
    !PUBLIC_GITHUB_URL.is_empty()
}

pub fn has_public_linkedin_url() -> bool {
    !PUBLIC_LINKEDIN_URL.is_empty()
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub admin_username: String,
    pub session_secret: String,
    pub base_url: String,
    pub rust_log: String,
    pub port: u16,
    pub secure_cookies: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = database_url_from_env()?;
        let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "lars".to_string());
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
        if admin_username.trim().is_empty() {
            bail!("ADMIN_USERNAME cannot be empty");
        }

        let railway_environment = env::var("RAILWAY_ENVIRONMENT").is_ok()
            || env::var("RAILWAY_PUBLIC_DOMAIN").is_ok()
            || env::var("RAILWAY_PRIVATE_DOMAIN").is_ok();

        Ok(Self {
            database_url,
            admin_username: admin_username.trim().to_string(),
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

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn database_url_from_env() -> Result<String> {
    let database_url = required_env("DATABASE_URL")?;
    let pguser = optional_env("PGUSER");
    let pgpassword = optional_env("PGPASSWORD");

    if pguser.is_none() && pgpassword.is_none() {
        return Ok(database_url);
    }

    let mut parsed = Url::parse(&database_url)
        .context("DATABASE_URL must be a valid URL when PGUSER or PGPASSWORD are set")?;

    if let Some(username) = pguser {
        parsed
            .set_username(&username)
            .map_err(|_| anyhow::anyhow!("PGUSER could not be encoded into DATABASE_URL"))?;
    }

    if let Some(password) = pgpassword {
        parsed
            .set_password(Some(&password))
            .map_err(|_| anyhow::anyhow!("PGPASSWORD could not be encoded into DATABASE_URL"))?;
    }

    Ok(parsed.to_string())
}
