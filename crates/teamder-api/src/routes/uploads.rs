use std::path::{Path, PathBuf};

use rocket::{
    Route, State,
    form::Form,
    fs::TempFile,
    serde::json::Json,
};
use serde_json::{Value, json};
use teamder_core::{error::TeamderError, models::user::PortfolioItem};
use uuid::Uuid;

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

const UPLOAD_ROOT: &str = "uploads";

#[derive(FromForm)]
struct FileUpload<'r> {
    file: TempFile<'r>,
    #[field(default = String::new())]
    title: String,
    #[field(default = String::new())]
    kind: String,
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "upload".into()
    } else {
        cleaned
    }
}

fn build_filename(stem: Option<&str>, ext: Option<String>) -> String {
    let stem = sanitize(stem.unwrap_or("upload"));
    let id = Uuid::new_v4().simple().to_string();
    match ext {
        Some(e) if !e.is_empty() => format!("{}-{}.{}", id, stem, e),
        _ => format!("{}-{}", id, stem),
    }
}

async fn persist(
    file: &mut TempFile<'_>,
    user_id: &str,
    subdir: &str,
) -> Result<(String, String), TeamderError> {
    let ext = file
        .content_type()
        .and_then(|c| c.extension())
        .map(|e| e.to_string());
    let filename = build_filename(file.name(), ext);

    let dir: PathBuf = Path::new(UPLOAD_ROOT).join(user_id).join(subdir);
    std::fs::create_dir_all(&dir).map_err(|e| TeamderError::Internal(e.to_string()))?;

    let dest = dir.join(&filename);
    file.persist_to(&dest)
        .await
        .map_err(|e| TeamderError::Internal(e.to_string()))?;

    let url = format!("/uploads/{}/{}/{}", user_id, subdir, filename);
    Ok((filename, url))
}

#[post("/portfolio", data = "<form>")]
async fn upload_portfolio(
    mut form: Form<FileUpload<'_>>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let display_name = form
        .file
        .name()
        .unwrap_or("upload")
        .to_string();
    let (filename, url) = persist(&mut form.file, &auth.0.sub, "portfolio").await?;

    let title = if form.title.trim().is_empty() {
        display_name
    } else {
        form.title.clone()
    };
    let kind = if form.kind.trim().is_empty() {
        "file".to_string()
    } else {
        form.kind.clone()
    };

    let item = PortfolioItem {
        title: title.clone(),
        kind: kind.clone(),
        description: None,
        url: Some(url.clone()),
    };
    state
        .users
        .push_portfolio_item(&auth.0.sub, &item)
        .await?;

    Ok(Json(json!({
        "url": url,
        "filename": filename,
        "title": title,
        "kind": kind,
    })))
}

#[post("/avatar", data = "<form>")]
async fn upload_avatar(
    mut form: Form<FileUpload<'_>>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let ct = form.file.content_type().map(|c| c.to_string()).unwrap_or_default();
    if !ct.starts_with("image/") {
        return Err(TeamderError::Validation("Only image files are accepted for avatars".into()).into());
    }
    let (filename, url) = persist(&mut form.file, &auth.0.sub, "avatar").await?;
    state.users.set_avatar_url(&auth.0.sub, Some(url.clone())).await?;
    Ok(Json(json!({ "url": url, "filename": filename })))
}

#[post("/resume", data = "<form>")]
async fn upload_resume(
    mut form: Form<FileUpload<'_>>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let (filename, url) = persist(&mut form.file, &auth.0.sub, "resume").await?;
    state
        .users
        .set_resume_url(&auth.0.sub, Some(url.clone()))
        .await?;
    Ok(Json(json!({ "url": url, "filename": filename })))
}

#[post("/banner", data = "<form>")]
async fn upload_banner(
    mut form: Form<FileUpload<'_>>,
    auth: AuthUser,
) -> ApiResult<Value> {
    let (filename, url) = persist(&mut form.file, &auth.0.sub, "banners").await?;
    Ok(Json(json!({ "url": url, "filename": filename })))
}

#[post("/application", data = "<form>")]
async fn upload_application_image(
    mut form: Form<FileUpload<'_>>,
    auth: AuthUser,
) -> ApiResult<Value> {
    let (filename, url) = persist(&mut form.file, &auth.0.sub, "applications").await?;
    Ok(Json(json!({ "url": url, "filename": filename })))
}

#[delete("/?<path>")]
async fn delete_upload(path: String, auth: AuthUser) -> ApiResult<Value> {
    let prefix = format!("/uploads/{}/", auth.0.sub);
    let relative = path
        .strip_prefix(&prefix)
        .ok_or(TeamderError::Forbidden)?;
    if relative.is_empty() || relative.contains("..") {
        return Err(TeamderError::Forbidden.into());
    }
    let fs_path = Path::new(UPLOAD_ROOT).join(&auth.0.sub).join(relative);
    if fs_path.exists() {
        std::fs::remove_file(&fs_path).map_err(|e| TeamderError::Internal(e.to_string()))?;
    }
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![upload_avatar, upload_portfolio, upload_resume, upload_banner, upload_application_image, delete_upload]
}
