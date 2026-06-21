use sqlx::{PgPool, postgres::PgPoolOptions, types::Json};
use uuid::Uuid;

use crate::models::{AdminCredentialRow, ProjectInput, ProjectRow, slugify};

pub type DbPool = PgPool;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn count_projects(pool: &DbPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await
}

pub async fn get_admin_credential(
    pool: &DbPool,
    username: &str,
) -> Result<Option<AdminCredentialRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminCredentialRow>(
        r#"
        SELECT username, password_hash
        FROM admin_credentials
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn create_admin_credential(
    pool: &DbPool,
    username: &str,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO admin_credentials (username, password_hash)
        VALUES ($1, $2)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub async fn list_published_projects(pool: &DbPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT *
        FROM projects
        WHERE published = true
        ORDER BY featured DESC, display_order ASC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_admin_projects(pool: &DbPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT *
        FROM projects
        ORDER BY display_order ASC, created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn latest_admin_projects(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT *
        FROM projects
        ORDER BY updated_at DESC, created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_project_by_id(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_published_project_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE slug = $1 AND published = true")
        .bind(slug)
        .fetch_optional(pool)
        .await
}

pub async fn create_project(pool: &DbPool, input: ProjectInput) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let display_order = next_display_order(pool).await?;

    sqlx::query(
        r#"
        INSERT INTO projects (
            id, title, slug, summary, body_markdown, role, status, tech_stack,
            github_url, live_url, image_url, accent, published, featured, display_order
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
    )
    .bind(id)
    .bind(input.title)
    .bind(input.slug)
    .bind(input.summary)
    .bind(input.body_markdown)
    .bind(input.role)
    .bind(input.status)
    .bind(Json(input.tech_stack))
    .bind(input.github_url)
    .bind(input.live_url)
    .bind(input.image_url)
    .bind(input.accent)
    .bind(input.published)
    .bind(input.featured)
    .bind(display_order)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn insert_seed_project(
    pool: &DbPool,
    input: ProjectInput,
    display_order: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO projects (
            id, title, slug, summary, body_markdown, role, status, tech_stack,
            github_url, live_url, image_url, accent, published, featured, display_order
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(input.title)
    .bind(input.slug)
    .bind(input.summary)
    .bind(input.body_markdown)
    .bind(input.role)
    .bind(input.status)
    .bind(Json(input.tech_stack))
    .bind(input.github_url)
    .bind(input.live_url)
    .bind(input.image_url)
    .bind(input.accent)
    .bind(input.published)
    .bind(input.featured)
    .bind(display_order)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_project(
    pool: &DbPool,
    id: Uuid,
    input: ProjectInput,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE projects
        SET title = $2,
            slug = $3,
            summary = $4,
            body_markdown = $5,
            role = $6,
            status = $7,
            tech_stack = $8,
            github_url = $9,
            live_url = $10,
            image_url = $11,
            accent = $12,
            published = $13,
            featured = $14
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(input.title)
    .bind(input.slug)
    .bind(input.summary)
    .bind(input.body_markdown)
    .bind(input.role)
    .bind(input.status)
    .bind(Json(input.tech_stack))
    .bind(input.github_url)
    .bind(input.live_url)
    .bind(input.image_url)
    .bind(input.accent)
    .bind(input.published)
    .bind(input.featured)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_project(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn toggle_published(pool: &DbPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE projects SET published = NOT published WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn move_project(
    pool: &DbPool,
    id: Uuid,
    direction: MoveDirection,
) -> Result<(), sqlx::Error> {
    let Some(current) = get_project_by_id(pool, id).await? else {
        return Ok(());
    };

    let neighbor = match direction {
        MoveDirection::Up => {
            sqlx::query_as::<_, ProjectRow>(
                r#"
                SELECT *
                FROM projects
                WHERE display_order < $1
                ORDER BY display_order DESC
                LIMIT 1
                "#,
            )
            .bind(current.display_order)
            .fetch_optional(pool)
            .await?
        }
        MoveDirection::Down => {
            sqlx::query_as::<_, ProjectRow>(
                r#"
                SELECT *
                FROM projects
                WHERE display_order > $1
                ORDER BY display_order ASC
                LIMIT 1
                "#,
            )
            .bind(current.display_order)
            .fetch_optional(pool)
            .await?
        }
    };

    if let Some(neighbor) = neighbor {
        let mut tx = pool.begin().await?;
        sqlx::query("UPDATE projects SET display_order = $1 WHERE id = $2")
            .bind(neighbor.display_order)
            .bind(current.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE projects SET display_order = $1 WHERE id = $2")
            .bind(current.display_order)
            .bind(neighbor.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
}

pub async fn unique_slug(
    pool: &DbPool,
    requested_slug: &str,
    fallback_title: &str,
    exclude_id: Option<Uuid>,
) -> Result<String, sqlx::Error> {
    let base = if requested_slug.trim().is_empty() {
        slugify(fallback_title)
    } else {
        slugify(requested_slug)
    };

    let mut candidate = base.clone();
    let mut suffix = 2;

    while slug_exists(pool, &candidate, exclude_id).await? {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }

    Ok(candidate)
}

async fn slug_exists(
    pool: &DbPool,
    slug: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let count = if let Some(exclude_id) = exclude_id {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE slug = $1 AND id <> $2")
            .bind(slug)
            .bind(exclude_id)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM projects WHERE slug = $1")
            .bind(slug)
            .fetch_one(pool)
            .await?
    };

    Ok(count > 0)
}

async fn next_display_order(pool: &DbPool) -> Result<i32, sqlx::Error> {
    let max_order = sqlx::query_scalar::<_, Option<i32>>("SELECT MAX(display_order) FROM projects")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

    Ok(max_order + 10)
}
