use rocket::{Route, State, serde::json::Json};
use serde::{Deserialize, Serialize};
use teamder_core::{
    error::TeamderError,
    models::{auth_code::AuthCode, user::User},
};

use crate::{auth, error::ApiResult, state::AppState};

/// How long a verification code stays valid.
const CODE_TTL_MINUTES: i64 = 10;

/// Default domain allowed to create new accounts (Fu Jen student cloud mail).
/// Override with the `REGISTER_EMAIL_DOMAIN` env var; set it to `*` (or empty)
/// to disable the restriction. Login is never restricted, so existing/seeded
/// accounts keep working.
const ALLOWED_REGISTER_DOMAIN: &str = "cloud.fju.edu.tw";

fn allowed_register_domain() -> Option<String> {
    let configured = std::env::var("REGISTER_EMAIL_DOMAIN")
        .unwrap_or_else(|_| ALLOWED_REGISTER_DOMAIN.to_string());
    let configured = configured.trim().to_lowercase();
    if configured.is_empty() || configured == "*" {
        None
    } else {
        Some(configured)
    }
}

fn register_domain_allowed(email: &str) -> bool {
    match allowed_register_domain() {
        None => true,
        Some(allowed) => matches!(email.rsplit_once('@'), Some((_, domain)) if domain == allowed),
    }
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    token: String,
    user: teamder_core::models::user::UserResponse,
}

#[derive(Debug, Deserialize)]
struct RequestCodeRequest {
    email: String,
    /// "register" | "login" | "delete"
    purpose: String,
}

#[derive(Debug, Serialize)]
struct RequestCodeResponse {
    success: bool,
    /// Only populated when SMTP isn't configured (dev mode) so the flow stays
    /// testable without an email server. `null` in production.
    dev_code: Option<String>,
}

fn is_valid_email(email: &str) -> bool {
    // Minimal sanity check: exactly one '@', non-empty local + domain, a dot in domain.
    let mut parts = email.split('@');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(local), Some(domain), None) => {
            !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        }
        _ => false,
    }
}

fn generate_code() -> String {
    // 6-digit numeric code. Uuid gives us entropy without pulling in `rand`.
    let n = uuid::Uuid::new_v4().as_u128() % 1_000_000;
    format!("{n:06}")
}

/// POST /api/v1/auth/request-code
/// Issues a verification code and emails it (or logs it in dev mode).
#[post("/request-code", data = "<req>")]
async fn request_code(
    req: Json<RequestCodeRequest>,
    state: &State<AppState>,
) -> ApiResult<RequestCodeResponse> {
    let email = req.email.trim().to_lowercase();
    let purpose = req.purpose.trim();

    if !is_valid_email(&email) {
        return Err(TeamderError::Validation("Please enter a valid email address".into()).into());
    }
    if !matches!(purpose, "register" | "login" | "delete") {
        return Err(TeamderError::Validation("Unknown verification purpose".into()).into());
    }

    let existing = state.users.find_by_email(&email).await?;
    match purpose {
        "register" => {
            if !register_domain_allowed(&email) {
                return Err(TeamderError::Validation(format!(
                    "Registration is restricted to @{ALLOWED_REGISTER_DOMAIN} email addresses"
                ))
                .into());
            }
            if existing.is_some() {
                return Err(TeamderError::Conflict(format!(
                    "{email} is already registered — sign in instead"
                ))
                .into());
            }
        }
        "login" | "delete" => {
            if existing.is_none() {
                return Err(TeamderError::NotFound(
                    "No account is registered with that email".into(),
                )
                .into());
            }
            if let Some(u) = &existing {
                if u.is_banned {
                    return Err(
                        TeamderError::Suspended("Your account has been suspended.".into()).into(),
                    );
                }
            }
        }
        _ => unreachable!(),
    }

    let code = generate_code();
    let record = AuthCode::new(&email, &code, purpose, CODE_TTL_MINUTES);
    state.auth_codes.set_code(&record).await?;

    state.mailer.send_code(&email, &code, purpose).await?;

    Ok(Json(RequestCodeResponse {
        success: true,
        dev_code: if state.mailer.is_live() { None } else { Some(code) },
    }))
}

#[derive(Debug, Deserialize)]
struct VerifyCodeRequest {
    email: String,
    code: String,
    /// "register" | "login"
    purpose: String,
}

/// POST /api/v1/auth/verify-code
/// Consumes a code. For "register" it creates the account; for "login" it
/// returns a token for the existing account. Either way you end up logged in.
#[post("/verify-code", data = "<req>")]
async fn verify_code(req: Json<VerifyCodeRequest>, state: &State<AppState>) -> ApiResult<AuthResponse> {
    let email = req.email.trim().to_lowercase();
    let purpose = req.purpose.trim();
    let code = req.code.trim();

    if !matches!(purpose, "register" | "login") {
        return Err(TeamderError::Validation("Unknown verification purpose".into()).into());
    }

    let record = state
        .auth_codes
        .find(&email, purpose)
        .await?
        .ok_or(TeamderError::Unauthorized)?;

    if record.is_expired() || record.code != code {
        return Err(TeamderError::Unauthorized.into());
    }

    // Single-use: clear the code regardless of branch below.
    state.auth_codes.delete(&email, purpose).await?;

    let user = match purpose {
        "register" => {
            // Defense in depth — the domain is also gated at request-code time.
            if !register_domain_allowed(&email) {
                return Err(TeamderError::Validation(format!(
                    "Registration is restricted to @{ALLOWED_REGISTER_DOMAIN} email addresses"
                ))
                .into());
            }
            // Guard against a race / double-submit creating a duplicate.
            if state.users.find_by_email(&email).await?.is_some() {
                return Err(TeamderError::Conflict(format!(
                    "{email} is already registered"
                ))
                .into());
            }
            // Seed a placeholder name from the email local part; the onboarding
            // wizard fills in the real name, role and department afterwards.
            let placeholder_name = email.split('@').next().unwrap_or("New user");
            let user = User::new(&email, placeholder_name, "", "");
            state.users.create(&user).await?;
            user
        }
        "login" => state
            .users
            .find_by_email(&email)
            .await?
            .ok_or(TeamderError::Unauthorized)?,
        _ => unreachable!(),
    };

    if user.is_banned {
        return Err(TeamderError::Suspended("Your account has been suspended.".into()).into());
    }

    let token = auth::create_token(
        &user.id,
        &user.email,
        user.is_admin,
        user.is_publisher,
        &state.jwt_secret,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub fn routes() -> Vec<Route> {
    routes![request_code, verify_code]
}
