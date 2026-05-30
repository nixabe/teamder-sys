use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::study_group_announcement::{CreateAnnouncementBody, StudyGroupAnnouncement},
};
use crate::{error::ApiResult, guards::AuthUser, state::AppState};

fn is_admin_or_creator(group_admins: &[String], created_by: &str, user_id: &str) -> bool {
    created_by == user_id || group_admins.contains(&user_id.to_string())
}

/// GET /api/v1/study-groups/:id/announcements
#[get("/<group_id>/announcements")]
async fn list_announcements(group_id: String, state: &State<AppState>) -> ApiResult<Value> {
    let announcements = state.sg_announcements.list_for_group(&group_id).await?;
    Ok(Json(json!({ "data": announcements })))
}

/// POST /api/v1/study-groups/:id/announcements  (auth — admin or creator)
#[post("/<group_id>/announcements", data = "<body>")]
async fn create_announcement(
    group_id: String,
    body: Json<CreateAnnouncementBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&group_id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;

    if !is_admin_or_creator(&g.admins, &g.created_by, &auth.0.sub) {
        return Err(TeamderError::Forbidden.into());
    }

    let author = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let mut ann = StudyGroupAnnouncement::new(
        &group_id, &auth.0.sub, &author.name, &body.title, &body.content,
    );
    ann.pinned = body.pinned;
    state.sg_announcements.create(&ann).await?;
    Ok(Json(json!({ "announcement": ann })))
}

/// DELETE /api/v1/study-groups/:id/announcements/:ann_id  (auth — admin or creator)
#[delete("/<group_id>/announcements/<ann_id>")]
async fn delete_announcement(
    group_id: String,
    ann_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&group_id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;

    if !is_admin_or_creator(&g.admins, &g.created_by, &auth.0.sub) {
        return Err(TeamderError::Forbidden.into());
    }

    state.sg_announcements.delete(&ann_id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_announcements, create_announcement, delete_announcement]
}
