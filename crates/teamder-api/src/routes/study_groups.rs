use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use std::collections::HashMap;
use teamder_core::{
    error::TeamderError,
    models::study_group::{CreateStudyGroupRequest, GroupMember, GroupMemberEnriched, StudyGroup, StudyGroupDetail, StudyGroupResponse},
};
use chrono::Utc;

use crate::{error::ApiResult, guards::AuthUser, state::AppState};

/// GET /api/v1/study-groups?limit=20&skip=0&open=true
#[get("/?<limit>&<skip>&<open>")]
async fn list_groups(
    limit: Option<i64>,
    skip: Option<u64>,
    open: Option<bool>,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let limit = limit.unwrap_or(20).min(100);
    let skip = skip.unwrap_or(0);

    let groups: Vec<StudyGroupResponse> = if open.unwrap_or(false) {
        state
            .study_groups
            .list_open()
            .await?
            .into_iter()
            .map(StudyGroupResponse::from)
            .collect()
    } else {
        state
            .study_groups
            .list(limit, skip)
            .await?
            .into_iter()
            .map(StudyGroupResponse::from)
            .collect()
    };

    let total = state.study_groups.count().await?;

    Ok(Json(json!({
        "data": groups,
        "meta": { "total": total, "limit": limit, "skip": skip }
    })))
}

/// GET /api/v1/study-groups/<id>
#[get("/<id>")]
async fn get_group(id: String, state: &State<AppState>) -> ApiResult<StudyGroupResponse> {
    let group = state
        .study_groups
        .find_by_id(&id)
        .await?
        .ok_or_else(|| TeamderError::NotFound(format!("Study group {} not found", id)))?;
    Ok(Json(group.into()))
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
        StudyGroupDetail {
            id: g.id, name: g.name, goal: g.goal, icon: g.icon, icon_bg: g.icon_bg,
            subject: g.subject, tags: g.tags, members, max_members: g.max_members,
            schedule: g.schedule, duration_weeks: g.duration_weeks,
            current_week: g.current_week, progress_percent: progress,
            is_open: g.is_open, join_mode: g.join_mode,
            created_by: g.created_by, creator_name, created_at: g.created_at,
        }
    }).collect();

    Ok(Json(json!({ "data": data })))
}

pub fn routes() -> Vec<Route> {
    routes![list_groups, get_group, create_group, join_group, checkin, joined_groups]
}
