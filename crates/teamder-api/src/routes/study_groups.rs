use chrono::Utc;
use mongodb::bson;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::guards::AuthUser;
use crate::state::AppState;
use teamder_core::error::TeamderError;
use teamder_core::models::notification::Notification;
use teamder_core::models::study_group::{GroupMember, StudyGroup, StudyNote};

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedStudyGroups {
    pub study_groups: Vec<StudyGroup>,
    pub total: u64,
    pub page: u64,
    pub limit: i64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudyGroupBody {
    pub name: String,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub max_members: Option<u8>,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub duration_weeks: Option<u8>,
    #[serde(default)]
    pub is_open: Option<bool>,
    #[serde(default)]
    pub join_mode: Option<String>,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudyGroupBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_bg: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub max_members: Option<u8>,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub duration_weeks: Option<u8>,
    #[serde(default)]
    pub is_open: Option<bool>,
    #[serde(default)]
    pub join_mode: Option<String>,
    #[serde(default)]
    pub banner_image: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NoteBody {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ProgressBody {
    pub current_week: u8,
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[rocket::get("/study-groups?<page>&<limit>&<open_only>")]
pub async fn list_study_groups(
    state: &State<AppState>,
    page: Option<u64>,
    limit: Option<i64>,
    open_only: Option<bool>,
) -> Result<Json<PaginatedStudyGroups>, ApiError> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);
    let skip = (page.saturating_sub(1)) * (limit as u64);
    let open_only = open_only.unwrap_or(false);

    let (groups, total) = state
        .db
        .study_group_repo()
        .list(open_only, skip, limit)
        .await?;

    Ok(Json(PaginatedStudyGroups {
        study_groups: groups,
        total,
        page,
        limit,
    }))
}

#[rocket::get("/study-groups/joined")]
pub async fn joined_study_groups(
    state: &State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<StudyGroup>>, ApiError> {
    let groups = state
        .db
        .study_group_repo()
        .find_joined(&auth.user_id)
        .await?;
    Ok(Json(groups))
}

#[rocket::get("/study-groups/<id>")]
pub async fn get_study_group(
    state: &State<AppState>,
    id: &str,
) -> Result<Json<StudyGroup>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;
    Ok(Json(group))
}

#[rocket::post("/study-groups", data = "<body>")]
pub async fn create_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    body: Json<CreateStudyGroupBody>,
) -> Result<Json<StudyGroup>, ApiError> {
    let req = body.into_inner();
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let creator_member = GroupMember {
        user_id: auth.user_id.clone(),
        initials: user.initials.clone(),
        color: user.gradient.clone(),
        joined_at: now,
        last_checkin: None,
        streak: 0,
    };

    let group = StudyGroup {
        id: id.clone(),
        name: req.name,
        goal: req.goal.unwrap_or_default(),
        icon: req.icon.unwrap_or_else(|| "Sg".to_string()),
        icon_bg: req.icon_bg.unwrap_or_default(),
        subject: req.subject.unwrap_or_else(|| "General".to_string()),
        tags: req.tags.unwrap_or_default(),
        members: vec![creator_member],
        max_members: req.max_members.unwrap_or(6),
        schedule: req.schedule.unwrap_or_default(),
        duration_weeks: req.duration_weeks.unwrap_or(0),
        current_week: 1,
        is_open: req.is_open.unwrap_or(true),
        status: "active".to_string(),
        join_mode: req.join_mode.unwrap_or_else(|| "direct".to_string()),
        banner_image: req.banner_image,
        notes: vec![],
        description: req.description,
        created_by: auth.user_id,
        created_at: now,
        updated_at: now,
    };

    state.db.study_group_repo().create(&group).await?;

    Ok(Json(group))
}

#[rocket::post("/study-groups/<id>/join")]
pub async fn join_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    // Check already a member
    if group.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(TeamderError::Conflict("Already a member".into()).into());
    }

    // Check capacity
    if group.members.len() >= group.max_members as usize {
        return Err(TeamderError::Validation("Group is full".into()).into());
    }

    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let member = GroupMember {
        user_id: auth.user_id,
        initials: user.initials,
        color: user.gradient,
        joined_at: Utc::now(),
        last_checkin: None,
        streak: 0,
    };

    state.db.study_group_repo().add_member(id, &member).await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/study-groups/<id>/checkin")]
pub async fn checkin(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if !group.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(TeamderError::Forbidden("Not a member".into()).into());
    }

    state
        .db
        .study_group_repo()
        .checkin(id, &auth.user_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/study-groups/<id>/notes", data = "<body>")]
pub async fn add_note(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<NoteBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if !group.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(TeamderError::Forbidden("Not a member".into()).into());
    }

    let user = state
        .db
        .user_repo()
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let req = body.into_inner();
    let note = StudyNote {
        id: Uuid::new_v4().to_string(),
        author_id: auth.user_id,
        author_name: user.name,
        title: req.title,
        body: req.body,
        created_at: Utc::now(),
    };

    state.db.study_group_repo().add_note(id, &note).await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::delete("/study-groups/<id>/notes/<note_id>")]
pub async fn delete_note(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    note_id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    // Check: author or creator
    let note = group
        .notes
        .iter()
        .find(|n| n.id == note_id)
        .ok_or_else(|| TeamderError::NotFound("Note not found".into()))?;

    if note.author_id != auth.user_id && group.created_by != auth.user_id {
        return Err(TeamderError::Forbidden("Not authorized".into()).into());
    }

    state
        .db
        .study_group_repo()
        .delete_note(id, note_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/study-groups/<id>/leave")]
pub async fn leave_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if group.created_by == auth.user_id {
        return Err(TeamderError::Validation("Creator cannot leave the group".into()).into());
    }

    state
        .db
        .study_group_repo()
        .remove_member(id, &auth.user_id)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/study-groups/<id>/progress", data = "<body>")]
pub async fn set_progress(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<ProgressBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if group.created_by != auth.user_id {
        return Err(TeamderError::Forbidden("Only the creator can set progress".into()).into());
    }

    state
        .db
        .study_group_repo()
        .update_progress(id, body.current_week)
        .await?;

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::post("/study-groups/<id>/complete")]
pub async fn complete_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if group.created_by != auth.user_id {
        return Err(TeamderError::Forbidden("Only the creator can complete the group".into()).into());
    }

    state
        .db
        .study_group_repo()
        .set_status(id, "completed")
        .await?;

    // Notify all members
    let now = Utc::now();
    for member in &group.members {
        if member.user_id != auth.user_id {
            let notif = Notification {
                id: Uuid::new_v4().to_string(),
                user_id: member.user_id.clone(),
                kind: "system".to_string(),
                title: format!("{} has been completed!", group.name),
                body: String::new(),
                link: Some(format!("/study-groups/{}", id)),
                read: false,
                created_at: now,
            };
            let _ = state.db.notification_repo().create(&notif).await;
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}

#[rocket::patch("/study-groups/<id>", data = "<body>")]
pub async fn update_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
    body: Json<UpdateStudyGroupBody>,
) -> Result<Json<StudyGroup>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if group.created_by != auth.user_id {
        return Err(TeamderError::Forbidden("Only the creator can edit the group".into()).into());
    }

    let req = body.into_inner();
    let mut update = bson::doc! {};

    if let Some(v) = &req.name { update.insert("name", v.as_str()); }
    if let Some(v) = &req.goal { update.insert("goal", v.as_str()); }
    if let Some(v) = &req.icon { update.insert("icon", v.as_str()); }
    if let Some(v) = &req.icon_bg { update.insert("icon_bg", v.as_str()); }
    if let Some(v) = &req.subject { update.insert("subject", v.as_str()); }
    if let Some(v) = &req.tags {
        update.insert("tags", bson::to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?);
    }
    if let Some(v) = req.max_members { update.insert("max_members", v as i32); }
    if let Some(v) = &req.schedule { update.insert("schedule", v.as_str()); }
    if let Some(v) = req.duration_weeks { update.insert("duration_weeks", v as i32); }
    if let Some(v) = req.is_open { update.insert("is_open", v); }
    if let Some(v) = &req.join_mode { update.insert("join_mode", v.as_str()); }
    if let Some(v) = &req.banner_image { update.insert("banner_image", v.as_str()); }
    if let Some(v) = &req.description { update.insert("description", v.as_str()); }

    update.insert("updated_at", bson::DateTime::from_chrono(Utc::now()));

    state.db.study_group_repo().update(id, update).await?;

    let updated = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    Ok(Json(updated))
}

#[rocket::delete("/study-groups/<id>")]
pub async fn delete_study_group(
    state: &State<AppState>,
    auth: AuthUser,
    id: &str,
) -> Result<Json<SuccessResponse>, ApiError> {
    let group = state
        .db
        .study_group_repo()
        .find_by_id(id)
        .await?
        .ok_or_else(|| TeamderError::NotFound("Study group not found".into()))?;

    if group.created_by != auth.user_id {
        let caller = state.db.user_repo().find_by_id(&auth.user_id).await?;
        if !caller.map(|u| u.is_admin).unwrap_or(false) {
            return Err(TeamderError::Forbidden("Not authorized".into()).into());
        }
    }

    state.db.study_group_repo().delete(id).await?;
    Ok(Json(SuccessResponse { success: true }))
}
