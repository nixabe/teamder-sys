use rocket::{Route, State, serde::json::Json};
use serde_json::{Value, json};
use teamder_core::{
    error::TeamderError,
    models::{
        contact_exchange::{
            ContactExchange, ContactExchangeResponse, ContactExchangeStatus,
            CreateContactExchangeBody, RespondContactExchangeBody,
        },
        message::Message,
    },
};

use crate::{error::ApiResult, guards::AuthUser, routes::chat::build_msg_response, state::AppState};

fn enrich(
    ex: ContactExchange,
    from_name: String,
    to_name: String,
) -> ContactExchangeResponse {
    ContactExchangeResponse {
        id: ex.id,
        from_user_id: ex.from_user_id,
        from_user_name: from_name,
        to_user_id: ex.to_user_id,
        to_user_name: to_name,
        status: ex.status,
        expires_at: ex.expires_at,
        created_at: ex.created_at,
    }
}

async fn resolve_names(
    state: &AppState,
    id_a: &str,
    id_b: &str,
) -> (String, String) {
    let users = state.users.find_many_by_ids(&[id_a, id_b]).await.unwrap_or_default();
    let name_a = users.iter().find(|u| u.id == id_a).map(|u| u.name.clone()).unwrap_or_default();
    let name_b = users.iter().find(|u| u.id == id_b).map(|u| u.name.clone()).unwrap_or_default();
    (name_a, name_b)
}

/// POST /api/v1/contact-exchange  (auth)
/// Initiates a contact info exchange request with another user.
#[post("/", data = "<body>")]
async fn create_exchange(
    body: Json<CreateContactExchangeBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let from_id = &auth.0.sub;
    let to_id = &body.to_user_id;

    if from_id == to_id {
        return Err(TeamderError::Conflict("Cannot exchange with yourself".into()).into());
    }

    // Check for existing active exchange
    let existing = state.contact_exchanges.find_between(from_id, to_id).await?;
    if let Some(ex) = &existing {
        match ex.status {
            ContactExchangeStatus::Pending =>
                return Err(TeamderError::Conflict("Request already pending".into()).into()),
            ContactExchangeStatus::Accepted if !ex.is_expired() =>
                return Err(TeamderError::Conflict("Already sharing contact info".into()).into()),
            _ => {} // declined/revoked/expired — allow a new request
        }
    }

    let exchange = ContactExchange::new(from_id, to_id);
    state.contact_exchanges.create(&exchange).await?;

    // System message so the chat history shows the request
    let msg = Message::system(from_id, to_id, &exchange.id, "contact_request");
    state.messages.create(&msg).await?;

    let (from_name, to_name) = resolve_names(state, from_id, to_id).await;
    let resp = build_msg_response(&msg, &from_name);
    if let Ok(json_str) = serde_json::to_string(&resp) {
        state.chat.send_to(to_id, json_str.clone()).await;
        state.chat.send_to(from_id, json_str).await;
    }

    Ok(Json(json!({
        "exchange": enrich(exchange, from_name, to_name)
    })))
}

/// POST /api/v1/contact-exchange/<id>/respond  (auth — must be recipient)
#[post("/<id>/respond", data = "<body>")]
async fn respond_exchange(
    id: String,
    body: Json<RespondContactExchangeBody>,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user_id = &auth.0.sub;
    let exchange = state.contact_exchanges.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Exchange not found".into()))?;

    if exchange.to_user_id != *user_id {
        return Err(TeamderError::Forbidden.into());
    }
    if exchange.status != ContactExchangeStatus::Pending {
        return Err(TeamderError::Conflict("Already responded".into()).into());
    }

    let (new_status, expires_at) = if body.accept {
        (ContactExchangeStatus::Accepted, Some(ContactExchange::accept_expiry()))
    } else {
        (ContactExchangeStatus::Declined, None)
    };
    state.contact_exchanges.update_status(&id, &new_status, expires_at).await?;

    // System message
    let kind = if body.accept { "contact_accepted" } else { "contact_declined" };
    let msg = Message::system(user_id, &exchange.from_user_id, &id, kind);
    state.messages.create(&msg).await?;

    let (responder_name, _) = resolve_names(state, user_id, &exchange.from_user_id).await;
    let resp = build_msg_response(&msg, &responder_name);
    if let Ok(json_str) = serde_json::to_string(&resp) {
        state.chat.send_to(&exchange.from_user_id, json_str.clone()).await;
        state.chat.send_to(user_id, json_str).await;
    }

    Ok(Json(json!({ "success": true, "status": if body.accept { "accepted" } else { "declined" } })))
}

/// POST /api/v1/contact-exchange/<id>/revoke  (auth — either party)
#[post("/<id>/revoke")]
async fn revoke_exchange(
    id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user_id = &auth.0.sub;
    let exchange = state.contact_exchanges.find_by_id(&id).await?
        .ok_or_else(|| TeamderError::NotFound("Exchange not found".into()))?;

    if exchange.from_user_id != *user_id && exchange.to_user_id != *user_id {
        return Err(TeamderError::Forbidden.into());
    }
    if exchange.status != ContactExchangeStatus::Accepted {
        return Err(TeamderError::Conflict("Can only revoke accepted exchanges".into()).into());
    }

    state.contact_exchanges.update_status(&id, &ContactExchangeStatus::Revoked, None).await?;

    // Broadcast revoke event to both parties via WS
    let partner_id = if exchange.from_user_id == *user_id {
        &exchange.to_user_id
    } else {
        &exchange.from_user_id
    };
    if let Ok(event) = serde_json::to_string(&json!({
        "type": "contact_revoked",
        "exchange_id": id
    })) {
        state.chat.send_to(partner_id, event.clone()).await;
        state.chat.send_to(user_id, event).await;
    }

    Ok(Json(json!({ "success": true })))
}

/// GET /api/v1/contact-exchange/with/<user_id>  (auth)
/// Returns the current exchange status between the caller and the given user.
#[get("/with/<partner_id>")]
async fn get_with(
    partner_id: String,
    auth: AuthUser,
    state: &State<AppState>,
) -> ApiResult<Value> {
    let user_id = &auth.0.sub;
    let exchange = state.contact_exchanges.find_between(user_id, &partner_id).await?;

    match exchange {
        None => Ok(Json(json!({ "found": false, "exchange": null }))),
        Some(mut ex) => {
            // Auto-expire if time passed
            if ex.status == ContactExchangeStatus::Accepted && ex.is_expired() {
                let _ = state.contact_exchanges.update_status(
                    &ex.id, &ContactExchangeStatus::Revoked, None
                ).await;
                ex.status = ContactExchangeStatus::Revoked;
            }
            let (from_name, to_name) = resolve_names(state, &ex.from_user_id, &ex.to_user_id).await;
            Ok(Json(json!({
                "found": true,
                "exchange": enrich(ex, from_name, to_name)
            })))
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![create_exchange, respond_exchange, revoke_exchange, get_with]
}
