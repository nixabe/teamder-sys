use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::user::PortfolioItem;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

fn get_extension(filename: &str) -> String {
    filename
        .rsplit('.')
        .next()
        .unwrap_or("bin")
        .to_lowercase()
}

// ── Upload forms ────────────────────────────────────────────────────────────

#[derive(FromForm)]
pub struct AvatarUpload<'r> {
    pub file: TempFile<'r>,
}

#[derive(FromForm)]
pub struct PortfolioUpload<'r> {
    pub file: TempFile<'r>,
    #[field(default = String::new())]
    pub title: String,
    #[field(default = String::new())]
    pub kind: String,
}

#[derive(FromForm)]
pub struct ResumeUpload<'r> {
    pub file: TempFile<'r>,
}

#[derive(FromForm)]
pub struct BannerUpload<'r> {
    pub file: TempFile<'r>,
}

#[derive(FromForm)]
pub struct ApplicationUpload<'r> {
    pub file: TempFile<'r>,
}

// ── Response DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::post("/uploads/avatar", data = "<form>")]
pub async fn upload_avatar(
    state: &State<AppState>,
    auth: AuthUser,
    mut form: Form<AvatarUpload<'_>>,
) -> Result<Json<UploadResponse>, ApiError> {
    let raw_name = form
        .file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw().as_str().to_string())
        .unwrap_or_else(|| "avatar.png".to_string());
    let safe_name = sanitize_filename(&raw_name);
    let ext = get_extension(&safe_name);
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    let dir = format!("uploads/{}/avatar", auth.user_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let path = format!("{}/{}", dir, filename);
    form.file
        .persist_to(&path)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/avatar/{}", auth.user_id, filename);
    state.db.user_repo().update_avatar(&auth.user_id, &url).await?;

    Ok(Json(UploadResponse { url }))
}

#[rocket::post("/uploads/portfolio", data = "<form>")]
pub async fn upload_portfolio(
    state: &State<AppState>,
    auth: AuthUser,
    mut form: Form<PortfolioUpload<'_>>,
) -> Result<Json<UploadResponse>, ApiError> {
    let raw_name = form
        .file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw().as_str().to_string())
        .unwrap_or_else(|| "portfolio.bin".to_string());
    let safe_name = sanitize_filename(&raw_name);
    let ext = get_extension(&safe_name);
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    let dir = format!("uploads/{}/portfolio", auth.user_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let path = format!("{}/{}", dir, filename);
    form.file
        .persist_to(&path)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/portfolio/{}", auth.user_id, filename);

    let title = if form.title.is_empty() {
        safe_name.clone()
    } else {
        form.title.clone()
    };
    let kind = if form.kind.is_empty() {
        "file".to_string()
    } else {
        form.kind.clone()
    };

    let item = PortfolioItem {
        title,
        kind,
        description: None,
        url: Some(url.clone()),
    };

    state
        .db
        .user_repo()
        .append_portfolio(&auth.user_id, &item)
        .await?;

    Ok(Json(UploadResponse { url }))
}

#[rocket::post("/uploads/resume", data = "<form>")]
pub async fn upload_resume(
    state: &State<AppState>,
    auth: AuthUser,
    mut form: Form<ResumeUpload<'_>>,
) -> Result<Json<UploadResponse>, ApiError> {
    let raw_name = form
        .file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw().as_str().to_string())
        .unwrap_or_else(|| "resume.pdf".to_string());
    let safe_name = sanitize_filename(&raw_name);
    let ext = get_extension(&safe_name);
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    let dir = format!("uploads/{}/resume", auth.user_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let path = format!("{}/{}", dir, filename);
    form.file
        .persist_to(&path)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/resume/{}", auth.user_id, filename);
    state.db.user_repo().update_resume(&auth.user_id, &url).await?;

    Ok(Json(UploadResponse { url }))
}

#[rocket::post("/uploads/banner", data = "<form>")]
pub async fn upload_banner(
    auth: AuthUser,
    mut form: Form<BannerUpload<'_>>,
) -> Result<Json<UploadResponse>, ApiError> {
    let raw_name = form
        .file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw().as_str().to_string())
        .unwrap_or_else(|| "banner.png".to_string());
    let safe_name = sanitize_filename(&raw_name);
    let ext = get_extension(&safe_name);
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    let dir = format!("uploads/{}/banners", auth.user_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let path = format!("{}/{}", dir, filename);
    form.file
        .persist_to(&path)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/banners/{}", auth.user_id, filename);
    Ok(Json(UploadResponse { url }))
}

#[rocket::post("/uploads/application", data = "<form>")]
pub async fn upload_application(
    auth: AuthUser,
    mut form: Form<ApplicationUpload<'_>>,
) -> Result<Json<UploadResponse>, ApiError> {
    let raw_name = form
        .file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw().as_str().to_string())
        .unwrap_or_else(|| "application.bin".to_string());
    let safe_name = sanitize_filename(&raw_name);
    let ext = get_extension(&safe_name);
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    let dir = format!("uploads/{}/applications", auth.user_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let path = format!("{}/{}", dir, filename);
    form.file
        .persist_to(&path)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/applications/{}", auth.user_id, filename);
    Ok(Json(UploadResponse { url }))
}

#[rocket::delete("/uploads?<path>")]
pub async fn delete_upload(
    auth: AuthUser,
    path: String,
) -> Result<Json<SuccessResponse>, ApiError> {
    // Security: must not contain ".." and must belong to the user
    if path.contains("..") {
        return Err(TeamderError::Validation("Invalid path".into()).into());
    }

    // The path should be like /uploads/<user_id>/...
    let expected_prefix = format!("/uploads/{}/", auth.user_id);
    if !path.starts_with(&expected_prefix) {
        return Err(TeamderError::Forbidden("Cannot delete another user's files".into()).into());
    }

    // Convert URL path to filesystem path
    let fs_path = path.trim_start_matches('/');
    if tokio::fs::metadata(fs_path).await.is_ok() {
        tokio::fs::remove_file(fs_path)
            .await
            .map_err(|e| TeamderError::Internal(e.to_string()))?;
    }

    Ok(Json(SuccessResponse { success: true }))
}
