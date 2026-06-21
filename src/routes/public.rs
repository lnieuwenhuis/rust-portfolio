use std::sync::Arc;

use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};

use crate::{
    AppState, config, db,
    error::{AppError, render},
    markdown,
    models::PublicProjectView,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/projects/{slug}", get(project_detail))
        .route("/health", get(health))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    projects: Vec<PublicProjectView>,
    has_projects: bool,
    contact_email: &'static str,
    mailto: String,
    github_url: &'static str,
    has_github_url: bool,
    linkedin_url: &'static str,
    has_linkedin_url: bool,
}

#[derive(Template)]
#[template(path = "project_detail.html")]
struct ProjectDetailTemplate {
    project: PublicProjectView,
    contact_email: &'static str,
    mailto: String,
}

async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>, AppError> {
    let projects = db::list_published_projects(&state.pool)
        .await?
        .into_iter()
        .map(|project| {
            let html = markdown::render_markdown(&project.body_markdown);
            PublicProjectView::from_row(project, html)
        })
        .collect::<Vec<_>>();

    render(IndexTemplate {
        has_projects: !projects.is_empty(),
        projects,
        contact_email: config::CONTACT_EMAIL,
        mailto: format!("mailto:{}", config::CONTACT_EMAIL),
        github_url: config::PUBLIC_GITHUB_URL,
        has_github_url: !config::PUBLIC_GITHUB_URL.is_empty(),
        linkedin_url: config::PUBLIC_LINKEDIN_URL,
        has_linkedin_url: !config::PUBLIC_LINKEDIN_URL.is_empty(),
    })
}

async fn project_detail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, AppError> {
    let Some(project) = db::get_published_project_by_slug(&state.pool, &slug).await? else {
        return Err(AppError::NotFound);
    };

    let project = PublicProjectView::from_row(
        project.clone(),
        markdown::render_markdown(&project.body_markdown),
    );

    render(ProjectDetailTemplate {
        project,
        contact_email: config::CONTACT_EMAIL,
        mailto: format!("mailto:{}", config::CONTACT_EMAIL),
    })
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"ok"}"#,
    )
}

async fn robots(State(state): State<Arc<AppState>>) -> Response {
    let body = format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
        state.config.base_url
    );
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
        .into_response()
}

async fn sitemap(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let projects = db::list_published_projects(&state.pool).await?;
    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    body.push_str(&format!(
        "  <url><loc>{}</loc><priority>1.0</priority></url>\n",
        xml_escape(&state.config.base_url)
    ));

    for project in projects {
        body.push_str(&format!(
            "  <url><loc>{}/projects/{}</loc><priority>0.8</priority></url>\n",
            xml_escape(&state.config.base_url),
            xml_escape(&project.slug)
        ));
    }

    body.push_str("</urlset>\n");

    let mut response = (StatusCode::OK, body).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    Ok(response)
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
