use crate::{
    db::{self, DbPool},
    models::{ProjectInput, clean_accent},
};

pub async fn seed_if_empty(pool: &DbPool) -> Result<(), sqlx::Error> {
    if db::count_projects(pool).await? > 0 {
        return Ok(());
    }

    let projects = [
        ProjectInput {
            title: "Dealer Banden Hotel / QR workflow system".to_string(),
            slug: "dealer-banden-hotel-qr-workflow".to_string(),
            summary: "A full-stack workflow for tire-set intake, request states, QR scanning, and warehouse-friendly handoffs.".to_string(),
            body_markdown: "Built around practical service-desk and warehouse flows: scan a QR code, find the right tire set, update request state, and keep the next action clear. The work touches React, TypeScript, Laravel, GraphQL, and operational UI design.".to_string(),
            role: Some("Full-stack development".to_string()),
            status: "Draft case study".to_string(),
            tech_stack: vec![
                "React".to_string(),
                "TypeScript".to_string(),
                "Laravel".to_string(),
                "GraphQL".to_string(),
                "QR workflows".to_string(),
            ],
            github_url: None,
            live_url: None,
            image_url: None,
            accent: clean_accent("#7ec8ff"),
            published: false,
            featured: true,
        },
        ProjectInput {
            title: "Landstede Scrum".to_string(),
            slug: "landstede-scrum".to_string(),
            summary: "A Jira-like scrum board with cards, workflow states, and burndown graph functionality.".to_string(),
            body_markdown: "A school-focused planning tool exploring how teams move work from idea to done. The project combines Laravel and Vue with board interactions, card management, and sprint progress visualization.".to_string(),
            role: Some("Application development".to_string()),
            status: "Draft case study".to_string(),
            tech_stack: vec![
                "Laravel".to_string(),
                "Vue".to_string(),
                "Scrum board".to_string(),
                "Burndown graph".to_string(),
            ],
            github_url: None,
            live_url: None,
            image_url: None,
            accent: clean_accent("#ff8fcf"),
            published: false,
            featured: false,
        },
        ProjectInput {
            title: "Rust Portfolio".to_string(),
            slug: "rust-portfolio".to_string(),
            summary: "This portfolio, built as a Rust-first server-rendered app with Axum, Askama, SQLx, and PostgreSQL.".to_string(),
            body_markdown: "A personal portfolio designed to stay fast, editable, and deployment-friendly without a JavaScript framework or frontend build step. It uses Rust for routing, templates, persistence, authentication, and deployment ergonomics.".to_string(),
            role: Some("Rust-first product build".to_string()),
            status: "In progress".to_string(),
            tech_stack: vec![
                "Rust".to_string(),
                "Axum".to_string(),
                "Askama".to_string(),
                "SQLx".to_string(),
                "PostgreSQL".to_string(),
                "Railway".to_string(),
            ],
            github_url: None,
            live_url: None,
            image_url: None,
            accent: clean_accent("#8a8cff"),
            published: false,
            featured: true,
        },
    ];

    for (index, project) in projects.into_iter().enumerate() {
        db::insert_seed_project(pool, project, ((index + 1) * 10) as i32).await?;
    }

    Ok(())
}
