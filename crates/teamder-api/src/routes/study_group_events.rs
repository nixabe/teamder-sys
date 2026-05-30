use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::study_group_event::{CreateEventBody, StudyGroupEvent},
};
use crate::{error::ApiResult, guards::AuthUser, state::AppState};

fn is_admin_or_creator(group_admins: &[String], created_by: &str, user_id: &str) -> bool {
    created_by == user_id || group_admins.contains(&user_id.to_string())
}

/// GET /api/v1/study-groups/:id/events
#[get("/<group_id>/events")]
async fn list_events(group_id: String, state: &State<AppState>) -> ApiResult<Value> {
    let events = state.sg_events.list_for_group(&group_id).await?;
    Ok(Json(json!({ "data": events })))
}

/// POST /api/v1/study-groups/:id/events  (auth — admin or creator)
#[post("/<group_id>/events", data = "<body>")]
async fn create_event(
    group_id: String,
    body: Json<CreateEventBody>,
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

    let mut event = StudyGroupEvent::new(
        &group_id, &auth.0.sub, &author.name, &body.title, &body.location, body.starts_at,
    );
    event.ends_at = body.ends_at;
    event.description = body.description.clone();
    state.sg_events.create(&event).await?;
    Ok(Json(json!({ "event": event })))
}

/// POST /api/v1/study-groups/:id/events/:event_id/rsvp  (auth — toggle attendance)
#[post("/<group_id>/events/<event_id>/rsvp")]
async fn rsvp_event(
    group_id: String,
    event_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    // Verify the user is a member of the group
    let g = state.study_groups.find_by_id(&group_id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;
    let is_member = g.created_by == auth.0.sub || g.members.iter().any(|m| m.user_id == auth.0.sub);
    if !is_member { return Err(TeamderError::Forbidden.into()); }

    let event = state.sg_events.find_by_id(&event_id).await?
        .ok_or_else(|| TeamderError::NotFound("Event not found".into()))?;

    if event.attendees.contains(&auth.0.sub) {
        state.sg_events.remove_attendee(&event_id, &auth.0.sub).await?;
        Ok(Json(json!({ "attending": false })))
    } else {
        state.sg_events.add_attendee(&event_id, &auth.0.sub).await?;
        Ok(Json(json!({ "attending": true })))
    }
}

/// DELETE /api/v1/study-groups/:id/events/:event_id  (auth — admin or creator)
#[delete("/<group_id>/events/<event_id>")]
async fn delete_event(
    group_id: String,
    event_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let g = state.study_groups.find_by_id(&group_id).await?
        .ok_or_else(|| TeamderError::NotFound("Group not found".into()))?;

    if !is_admin_or_creator(&g.admins, &g.created_by, &auth.0.sub) {
        return Err(TeamderError::Forbidden.into());
    }

    state.sg_events.delete(&event_id).await?;
    Ok(Json(json!({ "success": true })))
}

pub fn routes() -> Vec<Route> {
    routes![list_events, create_event, rsvp_event, delete_event]
}
