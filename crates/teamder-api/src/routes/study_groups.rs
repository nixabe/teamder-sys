use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use std::collections::HashMap;
use serde::Deserialize;
use teamder_core::{
    error::TeamderError,
    models::notification::{Notification, NotificationKind},
    models::study_group::{CreateStudyGroupRequest, CreateStudyNoteRequest, GroupMember, GroupMemberEnriched, StudyGroup, StudyGroupDetail, StudyGroupResponse, StudyGroupStatus, StudyNote},
};
use chrono::Utc;

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

#[derive(Debug, Deserialize)]
struct UpdateStudyGroupRequest {
    name: Option<String>,
    goal: Option<String>,
    description: Option<String>,
    subject: Option<String>,
    tags: Option<Vec<String>>,
    schedule: Option<String>,
    max_members: Option<u8>,
    is_open: Option<bool>,
    join_mode: Option<String>,
    banner_image: Option<String>,
}

/// GET /api/v1/study-groups?limit=20&skip=0&page=1&is_open=true&subject=Frontend&search=rust
#[get("/?<limit>&<skip>&<page>&<open>&<is_open>&<subject>&<search>")]
async fn list_groups(
    limit: Option<i64>,
    skip: Option<u64>,
    page: Option<u64>,
    open: Option<bool>,
    is_open: Option<bool>,
    subject: Option<String>,
    search: Option<String>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let limit = limit.unwrap_or(20).min(100);
    let effective_open = is_open.or(open);
    let skip = if let Some(p) = page {
        skip.unwrap_or(0).max((p.saturating_sub(1)) * limit as u64)
    } else {
        skip.unwrap_or(0)
    };

    let subject_ref = subject.as_deref();
    let search_ref = search.as_deref();

    let groups: Vec<StudyGroupResponse> = state
        .study_groups
        .list_filtered(limit, skip, effective_open, subject_ref, search_ref)
        .await?
        .into_iter()
        .map(StudyGroupResponse::from)
        .collect();

    let total = state.study_groups.count_filtered(effective_open, subject_ref, search_ref).await?;

    Ok(Json(json!({
        "data": groups,
        "total": total,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/study-groups/<id>  — returns full detail with enriched members
#[get("/<id>")]
async fn get_group(id: String, state: &State<AppState>) -> ApiResult<Value> {
    let g = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    let mut member_ids: Vec<&str> = std::iter::once(g.created_by.as_str())
        .chain(g.members.iter().map(|m| m.user_id.as_str()))
        .collect();
    member_ids.sort_unstable(); member_ids.dedup();
    let users = state.users.find_many_by_ids(&member_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    let creator_name = names.get(g.created_by.as_str()).copied().unwrap_or("").to_string();
    let progress = g.progress_percent();
    let members: Vec<GroupMemberEnriched> = g.members.iter().map(|m| GroupMemberEnriched {
        user_id: m.user_id.clone(),
        name: names.get(m.user_id.as_str()).copied().unwrap_or("").to_string(),
        initials: m.initials.clone(),
        color: m.color.clone(),
        joined_at: m.joined_at,
        streak: m.streak,
    }).collect();

    let member_count = members.len();
    let detail = StudyGroupDetail {
        id: g.id, name: g.name, goal: g.goal, icon: g.icon, icon_bg: g.icon_bg,
        subject: g.subject, tags: g.tags, members, member_count, max_members: g.max_members,
        schedule: g.schedule, duration_weeks: g.duration_weeks,
        current_week: g.current_week, progress_percent: progress,
        is_open: g.is_open, status: g.status, join_mode: g.join_mode,
        banner_image: g.banner_image, notes: g.notes, description: g.description,
        admins: g.admins, created_by: g.created_by, creator_name, created_at: g.created_at,
    };

    Ok(Json(json!(detail)))
}

/// POST /api/v1/study-groups  (auth)
#[post("/", data = "<req>")]
async fn create_group(
    req: Json<CreateStudyGroupRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<StudyGroupResponse> {
    let mut group = StudyGroup::new(&req.name, &req.goal, &auth.0.sub);
    group.subject = req.subject.clone();
    group.tags = req.tags.clone();
    group.max_members = req.max_members.unwrap_or(6);
    group.schedule = req.schedule.clone();
    group.duration_weeks = req.duration_weeks;
    if let Some(v) = &req.icon { group.icon = v.clone(); }
    if let Some(v) = &req.icon_bg { group.icon_bg = v.clone(); }
    if let Some(v) = req.join_mode.clone() { group.join_mode = v; }
    if req.banner_image.is_some() { group.banner_image = req.banner_image.clone(); }
    if req.description.is_some() { group.description = req.description.clone(); }

    // Fetch creator info so they appear in the members list and can check in
    let creator = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
    let creator_member = GroupMember {
        user_id: auth.0.sub.clone(),
        initials: creator.initials,
        color: "#4F6D7A".into(),
        joined_at: Utc::now(),
        last_checkin: None,
        streak: 0,
    };
    group.members.push(creator_member);

    state.study_groups.create(&group).await?;
    Ok(Json(group.into()))
}

/// POST /api/v1/study-groups/<id>/join  (auth)
#[post("/<id>/join")]
async fn join_group(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if !group.is_open {
        return Err(TeamderError::Conflict("This study group is closed".into()).into());
    }
    if group.members.len() >= group.max_members as usize {
        return Err(TeamderError::Conflict("Study group is full".into()).into());
    }

    // Fetch user details for initials/color
    let user = state
        .users
        .find_by_id(&auth.0.sub)
        .await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let member = GroupMember {
        user_id: auth.0.sub,
        initials: user.initials,
        color: "#4F6D7A".into(),
        joined_at: Utc::now(),
        last_checkin: None,
        streak: 0,
    };

    state.study_groups.add_member(&id, &member).await?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/study-groups/<id>/checkin  (auth)
#[post("/<id>/checkin")]
async fn checkin(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    // If the user is the group creator but was never added to members (legacy groups),
    // auto-add them now so they can check in.
    if g.created_by == auth.0.sub && !g.members.iter().any(|m| m.user_id == auth.0.sub) {
        let creator = state.users.find_by_id(&auth.0.sub).await?
            .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;
        let creator_member = GroupMember {
            user_id: auth.0.sub.clone(),
            initials: creator.initials,
            color: "#4F6D7A".into(),
            joined_at: Utc::now(),
            last_checkin: None,
            streak: 0,
        };
        state.study_groups.add_member(&id, &creator_member).await?;
    }

    let g = state.study_groups.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    let member = g.members.iter().find(|m| m.user_id == auth.0.sub)
        .ok_or_else(|| TeamderError::Forbidden)?;

    // Enforce once-per-day: compare UTC date of last_checkin with today
    if let Some(last) = member.last_checkin {
        let today = Utc::now().date_naive();
        let last_date = last.date_naive();
        if last_date == today {
            return Err(TeamderError::Conflict("Already checked in today".into()).into());
        }
    }

    state.study_groups.checkin(&id, &auth.0.sub).await?;
    Ok(Json(json!({ "success": true, "message": "Check-in recorded!" })))
}

/// GET /api/v1/study-groups/joined  (auth — groups where user is a member)
#[get("/joined")]
async fn joined_groups(auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    use serde_json::json;
    let user_id = &auth.0.sub;

    // also include groups the user created
    let mut all: Vec<StudyGroup> = state.study_groups.list_by_member(user_id).await?;
    let created = state.study_groups.list_by_creator(user_id).await?;
    for g in created {
        if !all.iter().any(|x| x.id == g.id) { all.push(g); }
    }

    let mut member_ids: Vec<&str> = all.iter()
        .flat_map(|g| std::iter::once(g.created_by.as_str())
            .chain(g.members.iter().map(|m| m.user_id.as_str())))
        .collect();
    member_ids.sort_unstable(); member_ids.dedup();
    let users = state.users.find_many_by_ids(&member_ids).await?;
    let names: HashMap<&str, &str> = users.iter().map(|u| (u.id.as_str(), u.name.as_str())).collect();

    let data: Vec<StudyGroupDetail> = all.into_iter().map(|g| {
        let creator_name = names.get(g.created_by.as_str()).copied().unwrap_or("").to_string();
        let progress = g.progress_percent();
        let members: Vec<GroupMemberEnriched> = g.members.iter().map(|m| GroupMemberEnriched {
            user_id: m.user_id.clone(),
            name: names.get(m.user_id.as_str()).copied().unwrap_or("").to_string(),
            initials: m.initials.clone(),
            color: m.color.clone(),
            joined_at: m.joined_at,
            streak: m.streak,
        }).collect();
        let member_count = members.len();
        StudyGroupDetail {
            id: g.id, name: g.name, goal: g.goal, icon: g.icon, icon_bg: g.icon_bg,
            subject: g.subject, tags: g.tags, members, member_count, max_members: g.max_members,
            schedule: g.schedule, duration_weeks: g.duration_weeks,
            current_week: g.current_week, progress_percent: progress,
            is_open: g.is_open, status: g.status, join_mode: g.join_mode,
            banner_image: g.banner_image, notes: g.notes, description: g.description,
            admins: g.admins, created_by: g.created_by, creator_name, created_at: g.created_at,
        }
    }).collect();

    Ok(Json(json!({ "data": data })))
}

/// GET /api/v1/study-groups/<id>/notes
#[get("/<id>/notes")]
async fn list_notes(id: String, state: &State<AppState>) -> ApiResult<Value> {
    let g = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;
    Ok(Json(json!({ "data": g.notes })))
}

/// POST /api/v1/study-groups/<id>/notes  (auth — member or creator)
#[post("/<id>/notes", data = "<req>")]
async fn add_note(
    id: String,
    req: Json<CreateStudyNoteRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    let is_member = g.created_by == auth.0.sub || g.members.iter().any(|m| m.user_id == auth.0.sub);
    if !is_member {
        return Err(TeamderError::Forbidden.into());
    }

    let user = state.users.find_by_id(&auth.0.sub).await?
        .ok_or_else(|| TeamderError::NotFound("User not found".into()))?;

    let note = StudyNote {
        id: uuid::Uuid::new_v4().to_string(),
        author_id: auth.0.sub.clone(),
        author_name: user.name.clone(),
        title: req.title.clone(),
        body: req.body.clone(),
        created_at: Utc::now(),
    };

    state.study_groups.add_note(&id, &note).await?;

    Ok(Json(json!({ "success": true, "note": note })))
}

/// DELETE /api/v1/study-groups/<group_id>/notes/<note_id>  (auth — author or creator)
#[delete("/<group_id>/notes/<note_id>")]
async fn delete_note(
    group_id: String,
    note_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state
        .study_groups
        .find_by_id(&group_id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", group_id)))?;

    let note = g.notes.iter().find(|n| n.id == note_id)
        .ok_or_else(|| TeamderError::NotFound("Note not found".into()))?;

    if note.author_id != auth.0.sub && g.created_by != auth.0.sub {
        return Err(TeamderError::Forbidden.into());
    }

    state.study_groups.remove_note(&group_id, &note_id).await?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/study-groups/<id>/leave  (auth)
#[post("/<id>/leave")]
async fn leave_group(id: String, auth: AuthUser, state: &State<AppState>) -> ApiResult<Value> {
    let g = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if g.created_by == auth.0.sub {
        return Err(TeamderError::Conflict("Creator cannot leave the group".into()).into());
    }

    state.study_groups.remove_member(&id, &auth.0.sub).await?;

    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
struct UpdateProgressRequest {
    current_week: u8,
}

/// POST /api/v1/study-groups/<id>/progress  (auth — creator only)
#[post("/<id>/progress", data = "<req>")]
async fn update_progress(
    id: String,
    req: Json<UpdateProgressRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if group.created_by != auth.0.sub {
        return Err(TeamderError::Forbidden.into());
    }
    if group.status == StudyGroupStatus::Completed {
        return Err(TeamderError::Conflict("Study group is already completed".into()).into());
    }
    if req.current_week > group.duration_weeks {
        return Err(TeamderError::Validation("current_week cannot exceed duration_weeks".into()).into());
    }

    state.study_groups.update_progress(&id, req.current_week).await?;

    Ok(Json(json!({ "success": true, "current_week": req.current_week })))
}

/// POST /api/v1/study-groups/<id>/complete  (auth — creator only)
#[post("/<id>/complete")]
async fn complete_group(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if group.created_by != auth.0.sub {
        return Err(TeamderError::Forbidden.into());
    }
    if group.status == StudyGroupStatus::Completed {
        return Err(TeamderError::Conflict("Study group is already completed".into()).into());
    }

    state.study_groups.set_status(&id, "completed").await?;

    let creator_name = state.users.find_by_id(&auth.0.sub).await?
        .map(|u| u.name).unwrap_or_default();

    for member in &group.members {
        let n = Notification::new(
            &member.user_id,
            NotificationKind::System,
            format!("{} is completed!", group.name),
            format!("{} marked \"{}\" as completed. You can now leave reviews for your groupmates.", creator_name, group.name),
            Some(format!("/study-groups/{}", group.id)),
        );
        if let Err(e) = state.notifications.create(&n).await {
            tracing::warn!("failed to create completion notification: {e}");
        }
    }

    Ok(Json(json!({ "success": true })))
}

/// PATCH /api/v1/study-groups/<id>  (auth — creator only)
#[patch("/<id>", data = "<req>")]
async fn update_group(
    id: String,
    req: Json<UpdateStudyGroupRequest>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    use mongodb::bson::{doc, to_bson};

    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if group.created_by != auth.0.sub {
        return Err(TeamderError::Forbidden.into());
    }

    let mut fields = doc! {};
    if let Some(v) = &req.name { fields.insert("name", v.as_str()); }
    if let Some(v) = &req.goal { fields.insert("goal", v.as_str()); }
    if let Some(v) = &req.description { fields.insert("description", v.as_str()); }
    if let Some(v) = &req.subject { fields.insert("subject", v.as_str()); }
    if let Some(v) = &req.tags {
        let bson_tags = to_bson(v).map_err(|e| TeamderError::Internal(e.to_string()))?;
        fields.insert("tags", bson_tags);
    }
    if let Some(v) = &req.schedule { fields.insert("schedule", v.as_str()); }
    if let Some(v) = req.max_members { fields.insert("max_members", v as i32); }
    if let Some(v) = req.is_open { fields.insert("is_open", v); }
    if let Some(v) = &req.join_mode { fields.insert("join_mode", v.as_str()); }
    if let Some(v) = &req.banner_image { fields.insert("banner_image", v.as_str()); }

    if fields.is_empty() {
        return Ok(Json(json!({ "success": true })));
    }

    state.study_groups.update(&id, fields).await?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/study-groups/<id>  (auth — creator or admin)
#[delete("/<id>")]
async fn delete_group(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;

    if group.created_by != auth.0.sub && !auth.0.is_admin {
        return Err(TeamderError::Forbidden.into());
    }

    state.study_groups.delete(&id).await?;
    Ok(Json(json!({ "success": true })))
}

/// POST /api/v1/study-groups/<id>/admins  (auth — creator only)
/// Body: { user_id }
#[derive(serde::Deserialize)]
struct AdminBody { user_id: String }

#[post("/<id>/admins", data = "<body>")]
async fn add_admin(
    id: String,
    body: Json<AdminBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;
    if g.created_by != auth.0.sub { return Err(TeamderError::Forbidden.into()); }
    let is_member = g.members.iter().any(|m| m.user_id == body.user_id);
    if !is_member { return Err(TeamderError::Validation("User must be a member first".into()).into()); }
    state.study_groups.add_admin(&id, &body.user_id).await?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/v1/study-groups/<id>/admins/<user_id>  (auth — creator only)
#[delete("/<id>/admins/<user_id>")]
async fn remove_admin(
    id: String,
    user_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;
    if g.created_by != auth.0.sub { return Err(TeamderError::Forbidden.into()); }
    state.study_groups.remove_admin(&id, &user_id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_groups, get_group, create_group, join_group, checkin, joined_groups, list_notes, add_note, delete_note, leave_group, update_progress, complete_group, update_group, delete_group, add_admin, remove_admin]
}
