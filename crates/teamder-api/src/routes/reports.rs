use rocket::{Route, State, serde::json::Json};
use serde::Deserialize;
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::report::{CreateReportRequest, Report, ReportResponse, ReportStatus},
};

use crate::{error::ApiResult, guards::{AdminUser, AuthUser}, state::AppState};

/// POST /api/v1/reports
#[post("/", data = "<req>")]
async fn create(
    req: Json<CreateReportRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    if req.reason.trim().is_empty() {
        return Err(TeamderError::Validation("reason is required".into()).into());
    }
    let r = Report::new(&auth.0.sub, req.0.entity_type, req.0.entity_id, req.0.reason, req.0.details);
    state.reports.create(&r).await?;
    Ok(Json(json!({ "id": r.id })))
}

/// GET /api/v1/reports  (admin only)
#[get("/")]
async fn list(_admin: AdminUser, state: &State<AppState>) -> ApiResult<Value> {
    let raw = state.reports.list_all().await?;
    let data: Vec<ReportResponse> = raw.into_iter().map(Into::into).collect();
    Ok(Json(json!({ "data": data })))
}

/// Admin review patch. `status` accepts the snake_case ReportStatus strings
/// (pending|reviewing|resolved|dismissed); `admin_notes` is free-form.
#[derive(Debug, Deserialize)]
struct UpdateReportRequest {
    #[serde(default)]
    status: Option<ReportStatus>,
    #[serde(default)]
    admin_notes: Option<String>,
}

/// PATCH /api/v1/reports/<id>  (admin only)
#[patch("/<id>", data = "<req>")]
async fn update(
    id: String,
    req: Json<UpdateReportRequest>,
    admin: AdminUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    state
        .reports
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Report {} not found", id)))?;

    state
        .reports
        .update_review(&id, req.0.status, &admin.0.sub, req.0.admin_notes)
        .await?;

    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![create, list, update]
}
