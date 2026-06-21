use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub body_markdown: String,
    pub role: Option<String>,
    pub status: String,
    pub tech_stack: Json<Vec<String>>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub image_url: Option<String>,
    pub accent: Option<String>,
    pub published: bool,
    pub featured: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminCredentialRow {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct ProjectInput {
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub body_markdown: String,
    pub role: Option<String>,
    pub status: String,
    pub tech_stack: Vec<String>,
    pub github_url: Option<String>,
    pub live_url: Option<String>,
    pub image_url: Option<String>,
    pub accent: Option<String>,
    pub published: bool,
    pub featured: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProjectFormData {
    pub csrf_token: String,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub body_markdown: String,
    pub role: String,
    pub status: String,
    pub tech_stack: String,
    pub github_url: String,
    pub live_url: String,
    pub image_url: String,
    pub accent: String,
    pub published: Option<String>,
    pub featured: Option<String>,
}

impl ProjectFormData {
    pub fn into_input(self, generated_slug: String) -> ProjectInput {
        ProjectInput {
            title: self.title.trim().to_string(),
            slug: generated_slug,
            summary: self.summary.trim().to_string(),
            body_markdown: self.body_markdown.trim().to_string(),
            role: clean_optional(&self.role),
            status: clean_status(&self.status),
            tech_stack: parse_tech_stack(&self.tech_stack),
            github_url: clean_optional_url(&self.github_url),
            live_url: clean_optional_url(&self.live_url),
            image_url: clean_optional_url(&self.image_url),
            accent: clean_accent(&self.accent),
            published: self.published.is_some(),
            featured: self.featured.is_some(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteForm {
    pub csrf_token: String,
    pub confirm_delete: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublicProjectView {
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub body_html: String,
    pub role: String,
    pub has_role: bool,
    pub status: String,
    pub tech_stack: Vec<String>,
    pub github_url: String,
    pub has_github_url: bool,
    pub live_url: String,
    pub has_live_url: bool,
    pub image_url: String,
    pub has_image_url: bool,
    pub accent: String,
    pub featured: bool,
}

#[derive(Debug, Clone)]
pub struct AdminProjectView {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub body_markdown: String,
    pub role: String,
    pub status: String,
    pub tech_stack_text: String,
    pub github_url: String,
    pub live_url: String,
    pub image_url: String,
    pub accent: String,
    pub published: bool,
    pub featured: bool,
    pub display_order: i32,
    pub updated_at: String,
}

impl PublicProjectView {
    pub fn from_row(row: ProjectRow, body_html: String) -> Self {
        let role = row.role.unwrap_or_default();
        let github_url = row.github_url.unwrap_or_default();
        let live_url = row.live_url.unwrap_or_default();
        let image_url = row.image_url.unwrap_or_default();

        Self {
            title: row.title,
            slug: row.slug,
            summary: row.summary,
            body_html,
            has_role: !role.is_empty(),
            role,
            status: row.status,
            tech_stack: row.tech_stack.0,
            has_github_url: !github_url.is_empty(),
            github_url,
            has_live_url: !live_url.is_empty(),
            live_url,
            has_image_url: !image_url.is_empty(),
            image_url,
            accent: row.accent.unwrap_or_else(default_accent),
            featured: row.featured,
        }
    }
}

impl AdminProjectView {
    pub fn blank() -> Self {
        Self {
            id: Uuid::nil(),
            title: String::new(),
            slug: String::new(),
            summary: String::new(),
            body_markdown: String::new(),
            role: String::new(),
            status: "In progress".to_string(),
            tech_stack_text: String::new(),
            github_url: String::new(),
            live_url: String::new(),
            image_url: String::new(),
            accent: default_accent(),
            published: false,
            featured: false,
            display_order: 0,
            updated_at: String::new(),
        }
    }

    pub fn from_row(row: ProjectRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            slug: row.slug,
            summary: row.summary,
            body_markdown: row.body_markdown,
            role: row.role.unwrap_or_default(),
            status: row.status,
            tech_stack_text: row.tech_stack.0.join(", "),
            github_url: row.github_url.unwrap_or_default(),
            live_url: row.live_url.unwrap_or_default(),
            image_url: row.image_url.unwrap_or_default(),
            accent: row.accent.unwrap_or_else(default_accent),
            published: row.published,
            featured: row.featured,
            display_order: row.display_order,
            updated_at: row.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in input.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

pub fn clean_accent(input: &str) -> Option<String> {
    let value = input.trim();
    let hex = value.strip_prefix('#')?;
    let valid_length = hex.len() == 3 || hex.len() == 6;

    if valid_length && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        Some(format!("#{hex}"))
    } else {
        None
    }
}

pub fn default_accent() -> String {
    "#ff6fbd".to_string()
}

fn clean_optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn clean_optional_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with("https://") || value.starts_with("http://") {
        Some(value.to_string())
    } else {
        None
    }
}

fn clean_status(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "In progress".to_string()
    } else {
        value.to_string()
    }
}

fn parse_tech_stack(value: &str) -> Vec<String> {
    let mut stack = Vec::new();

    for item in value.split([',', '\n']) {
        let item = item.trim();
        if !item.is_empty() && !stack.iter().any(|existing| existing == item) {
            stack.push(item.to_string());
        }
    }

    stack
}

#[cfg(test)]
mod tests {
    use super::{clean_accent, slugify};

    #[test]
    fn slug_generation_is_stable_and_clean() {
        assert_eq!(slugify("Rust Portfolio!"), "rust-portfolio");
        assert_eq!(slugify("  Dealer / QR Workflow  "), "dealer-qr-workflow");
        assert_eq!(slugify("###"), "project");
    }

    #[test]
    fn accent_accepts_only_hex_colors() {
        assert_eq!(clean_accent("#ff88aa").as_deref(), Some("#ff88aa"));
        assert_eq!(clean_accent("background:red"), None);
    }
}
