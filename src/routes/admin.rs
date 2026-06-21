use std::sync::Arc;

use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppState, auth, csrf, db,
    db::MoveDirection,
    error::{AppError, render},
    models::{AdminProjectView, CsrfForm, DeleteForm, ProjectFormData},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/login", get(login_form).post(login))
        .route("/admin/logout", get(logout_get).post(logout))
        .route("/admin", get(dashboard))
        .route("/admin/projects", get(projects).post(create_project))
        .route("/admin/projects/new", get(new_project))
        .route("/admin/projects/{id}/edit", get(edit_project))
        .route("/admin/projects/{id}", post_update_project())
        .route("/admin/projects/{id}/delete", post_delete_project())
        .route(
            "/admin/projects/{id}/toggle-published",
            post_toggle_published(),
        )
        .route("/admin/projects/{id}/move-up", post_move_up())
        .route("/admin/projects/{id}/move-down", post_move_down())
}

fn post_update_project() -> axum::routing::MethodRouter<Arc<AppState>> {
    axum::routing::post(update_project)
}

fn post_delete_project() -> axum::routing::MethodRouter<Arc<AppState>> {
    axum::routing::post(delete_project)
}

fn post_toggle_published() -> axum::routing::MethodRouter<Arc<AppState>> {
    axum::routing::post(toggle_published)
}

fn post_move_up() -> axum::routing::MethodRouter<Arc<AppState>> {
    axum::routing::post(move_up)
}

fn post_move_down() -> axum::routing::MethodRouter<Arc<AppState>> {
    axum::routing::post(move_down)
}

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    csrf_token: String,
    error: String,
    has_error: bool,
    admin_username: String,
    needs_setup: bool,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    username: String,
    csrf_token: String,
    projects: Vec<AdminProjectView>,
    total_projects: usize,
    published_projects: usize,
    draft_projects: usize,
}

#[derive(Template)]
#[template(path = "admin/projects.html")]
struct ProjectsTemplate {
    username: String,
    csrf_token: String,
    projects: Vec<AdminProjectView>,
    has_projects: bool,
}

#[derive(Template)]
#[template(path = "admin/project_form.html")]
struct ProjectFormTemplate {
    username: String,
    csrf_token: String,
    heading: String,
    action: String,
    cancel_url: String,
    project: AdminProjectView,
    is_edit: bool,
    delete_action: String,
}

#[derive(Deserialize)]
struct LoginForm {
    csrf_token: String,
    username: String,
    password: String,
    confirm_password: Option<String>,
}

#[derive(Clone)]
struct ActiveAdmin {
    session_id: String,
    username: String,
    csrf_token: String,
}

async fn login_form(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let needs_setup = db::get_admin_credential(&state.pool, &state.config.admin_username)
        .await?
        .is_none();
    render_login(&state, None, needs_setup)
}

async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let credential = db::get_admin_credential(&state.pool, &state.config.admin_username).await?;
    let needs_setup = credential.is_none();
    let expected = auth::login_csrf_from_headers(&headers).unwrap_or_default();
    if csrf::validate(&expected, &form.csrf_token).is_err() {
        return render_login(
            &state,
            Some("The form expired. Please try again."),
            needs_setup,
        );
    }

    if form.username.trim() != state.config.admin_username {
        return render_login(
            &state,
            Some("Use the configured admin username."),
            needs_setup,
        );
    }

    if let Some(credential) = credential {
        if !auth::verify_password(&credential.password_hash, &form.password) {
            return render_login(
                &state,
                Some("The username or password was not right."),
                false,
            );
        }
    } else {
        if form.password.len() < 8 {
            return render_login(
                &state,
                Some("Choose a password with at least 8 characters."),
                true,
            );
        }

        let confirmation = form.confirm_password.unwrap_or_default();
        if form.password != confirmation {
            return render_login(&state, Some("The passwords did not match."), true);
        }

        let password_hash = auth::hash_password(&form.password).map_err(|error| {
            tracing::error!(?error, "failed to hash admin password");
            AppError::BadRequest("Could not save the password. Please try again.".to_string())
        })?;
        let created =
            db::create_admin_credential(&state.pool, &state.config.admin_username, &password_hash)
                .await?;
        if !created {
            return render_login(
                &state,
                Some("The admin password already exists. Sign in with it."),
                false,
            );
        }
    }

    start_admin_session(&state).await
}

async fn start_admin_session(state: &Arc<AppState>) -> Result<Response, AppError> {
    let (session_id, _) = state
        .sessions
        .create(state.config.admin_username.clone())
        .await;
    let mut response = Redirect::to("/admin").into_response();
    append_cookie(
        &mut response,
        auth::session_cookie(
            &state.config.session_secret,
            &session_id,
            state.config.secure_cookies,
        ),
    );
    append_cookie(
        &mut response,
        auth::expire_login_csrf_cookie(state.config.secure_cookies),
    );
    Ok(no_store(response))
}

async fn logout_get() -> Redirect {
    Redirect::to("/admin")
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<CsrfForm>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    state.sessions.remove(&admin.session_id).await;

    let mut response = Redirect::to("/admin/login").into_response();
    append_cookie(
        &mut response,
        auth::expire_session_cookie(state.config.secure_cookies),
    );
    Ok(no_store(response))
}

async fn dashboard(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    let rows = db::list_admin_projects(&state.pool).await?;
    let total_projects = rows.len();
    let published_projects = rows.iter().filter(|project| project.published).count();
    let projects = rows
        .into_iter()
        .take(5)
        .map(AdminProjectView::from_row)
        .collect::<Vec<_>>();

    template_response(DashboardTemplate {
        username: admin.username,
        csrf_token: admin.csrf_token,
        projects,
        total_projects,
        published_projects,
        draft_projects: total_projects.saturating_sub(published_projects),
    })
}

async fn projects(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    let projects = db::list_admin_projects(&state.pool)
        .await?
        .into_iter()
        .map(AdminProjectView::from_row)
        .collect::<Vec<_>>();

    template_response(ProjectsTemplate {
        username: admin.username,
        csrf_token: admin.csrf_token,
        has_projects: !projects.is_empty(),
        projects,
    })
}

async fn new_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    template_response(ProjectFormTemplate {
        username: admin.username,
        csrf_token: admin.csrf_token,
        heading: "New project".to_string(),
        action: "/admin/projects".to_string(),
        cancel_url: "/admin/projects".to_string(),
        project: AdminProjectView::blank(),
        is_edit: false,
        delete_action: String::new(),
    })
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<ProjectFormData>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    validate_project_form(&form)?;
    let slug = db::unique_slug(&state.pool, &form.slug, &form.title, None).await?;
    let id = db::create_project(&state.pool, form.into_input(slug)).await?;

    Ok(Redirect::to(&format!("/admin/projects/{id}/edit")).into_response())
}

async fn edit_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    let Some(project) = db::get_project_by_id(&state.pool, id).await? else {
        return Err(AppError::NotFound);
    };

    template_response(ProjectFormTemplate {
        username: admin.username,
        csrf_token: admin.csrf_token,
        heading: format!("Edit {}", project.title),
        action: format!("/admin/projects/{id}"),
        cancel_url: "/admin/projects".to_string(),
        project: AdminProjectView::from_row(project),
        is_edit: true,
        delete_action: format!("/admin/projects/{id}/delete"),
    })
}

async fn update_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Form(form): Form<ProjectFormData>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    validate_project_form(&form)?;
    let slug = db::unique_slug(&state.pool, &form.slug, &form.title, Some(id)).await?;
    db::update_project(&state.pool, id, form.into_input(slug)).await?;

    Ok(Redirect::to("/admin/projects").into_response())
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Form(form): Form<DeleteForm>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    if form.confirm_delete.as_deref() != Some("yes") {
        return Err(AppError::BadRequest(
            "Confirm deletion before removing the project.".to_string(),
        ));
    }

    db::delete_project(&state.pool, id).await?;
    Ok(Redirect::to("/admin/projects").into_response())
}

async fn toggle_published(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Form(form): Form<CsrfForm>,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    db::toggle_published(&state.pool, id).await?;

    Ok(Redirect::to("/admin/projects").into_response())
}

async fn move_up(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Form(form): Form<CsrfForm>,
) -> Result<Response, AppError> {
    move_project(state, headers, id, form, MoveDirection::Up).await
}

async fn move_down(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Form(form): Form<CsrfForm>,
) -> Result<Response, AppError> {
    move_project(state, headers, id, form, MoveDirection::Down).await
}

async fn move_project(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Uuid,
    form: CsrfForm,
    direction: MoveDirection,
) -> Result<Response, AppError> {
    let Some(admin) = current_admin(&state, &headers).await else {
        return Ok(Redirect::to("/admin/login").into_response());
    };

    csrf::validate(&admin.csrf_token, &form.csrf_token)?;
    db::move_project(&state.pool, id, direction).await?;

    Ok(Redirect::to("/admin/projects").into_response())
}

async fn current_admin(state: &Arc<AppState>, headers: &HeaderMap) -> Option<ActiveAdmin> {
    let session_id = auth::session_id_from_headers(headers, &state.config.session_secret)?;
    let session = state.sessions.get(&session_id).await?;

    Some(ActiveAdmin {
        session_id,
        username: session.username,
        csrf_token: session.csrf_token,
    })
}

fn render_login(
    state: &AppState,
    error: Option<&str>,
    needs_setup: bool,
) -> Result<Response, AppError> {
    let token = auth::random_token();
    let mut response = render(LoginTemplate {
        csrf_token: token.clone(),
        error: error.unwrap_or_default().to_string(),
        has_error: error.is_some(),
        admin_username: state.config.admin_username.clone(),
        needs_setup,
    })?
    .into_response();

    append_cookie(
        &mut response,
        auth::login_csrf_cookie(&token, state.config.secure_cookies),
    );

    Ok(no_store(response))
}

fn template_response<T: Template>(template: T) -> Result<Response, AppError> {
    Ok(no_store(render(template)?.into_response()))
}

fn validate_project_form(form: &ProjectFormData) -> Result<(), AppError> {
    if form.title.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Project title is required.".to_string(),
        ));
    }
    if form.summary.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Project summary is required.".to_string(),
        ));
    }
    Ok(())
}

fn append_cookie(response: &mut Response, cookie: String) {
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("cookie value should be valid"),
    );
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private"),
    );
    response
}
