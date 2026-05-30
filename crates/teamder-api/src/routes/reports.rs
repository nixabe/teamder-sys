use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::report::{CreateReportRequest, Report, ReportResponse},
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

pub fn routes() -> Vec<Route> {
    routes![create, list]
}
