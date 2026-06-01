use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use teamder_core::error::TeamderError;

/// Sends passwordless verification-code emails over SMTP.
///
/// When SMTP is not configured (no `SMTP_HOST`), the mailer runs in "dev mode":
/// it logs the code instead of sending and the route returns the code in the
/// response so the flow stays testable without infrastructure.
#[derive(Clone)]
pub struct Mailer {
    inner: Option<SmtpConfig>,
    /// Base URL of the frontend, used to build the magic verification link.
    pub base_url: String,
}

#[derive(Clone)]
struct SmtpConfig {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl Mailer {
    /// Build a mailer from environment variables:
    /// `SMTP_HOST`, `SMTP_PORT` (default 587), `SMTP_USERNAME`, `SMTP_PASSWORD`,
    /// `SMTP_FROM` (default = username), `APP_BASE_URL` (default localhost:3000).
    pub fn from_env() -> Self {
        let base_url = std::env::var("APP_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let host = match std::env::var("SMTP_HOST") {
            Ok(h) if !h.trim().is_empty() => h,
            _ => {
                tracing::warn!(
                    "SMTP_HOST not set — mailer runs in dev mode (codes logged, not emailed)"
                );
                return Self { inner: None, base_url };
            }
        };

        let port: u16 = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
        let password = std::env::var("SMTP_PASSWORD").unwrap_or_default();
        let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| username.clone());

        // STARTTLS on 587, implicit TLS on 465, plaintext otherwise.
        let builder = if port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        };

        let transport = match builder {
            Ok(b) => b
                .port(port)
                .credentials(Credentials::new(username, password))
                .build(),
            Err(e) => {
                tracing::error!("Invalid SMTP relay config ({e}); falling back to dev mode");
                return Self { inner: None, base_url };
            }
        };

        tracing::info!("SMTP mailer configured for host {host}:{port}");
        Self {
            inner: Some(SmtpConfig { transport, from }),
            base_url,
        }
    }

    /// True when real SMTP delivery is configured.
    pub fn is_live(&self) -> bool {
        self.inner.is_some()
    }

    /// Build the magic verification link the email links to.
    pub fn verify_link(&self, email: &str, code: &str, purpose: &str) -> String {
        format!(
            "{}/verify?email={}&code={}&purpose={}",
            self.base_url.trim_end_matches('/'),
            urlencode(email),
            code,
            purpose
        )
    }

    /// Send a verification code. In dev mode this logs and returns Ok.
    pub async fn send_code(
        &self,
        to: &str,
        code: &str,
        purpose: &str,
    ) -> Result<(), TeamderError> {
        let link = self.verify_link(to, code, purpose);

        let Some(cfg) = &self.inner else {
            tracing::info!("[dev mailer] verification code for {to} ({purpose}): {code} — link: {link}");
            return Ok(());
        };

        let action = match purpose {
            "register" => "complete your Teamder sign-up",
            "delete" => "confirm deleting your Teamder account",
            _ => "sign in to Teamder",
        };

        let text = format!(
            "Your Teamder verification code is: {code}\n\n\
             It expires in 10 minutes. Enter it to {action}.\n\n\
             Or open this link: {link}\n\n\
             If you didn't request this, you can ignore this email."
        );
        let html = format!(
            "<div style=\"font-family:sans-serif;color:#1F2A2F\">\
               <h2 style=\"color:#DD6E42\">Teamder</h2>\
               <p>Your verification code to {action}:</p>\
               <p style=\"font-size:30px;font-weight:700;letter-spacing:6px;color:#2C3E45\">{code}</p>\
               <p>It expires in 10 minutes.</p>\
               <p><a href=\"{link}\" style=\"color:#DD6E42\">Or click here to verify</a></p>\
               <p style=\"color:#8A99A0;font-size:12px\">If you didn't request this, ignore this email.</p>\
             </div>"
        );

        let email = Message::builder()
            .from(cfg.from.parse().map_err(|e| {
                TeamderError::Internal(format!("bad SMTP_FROM address: {e}"))
            })?)
            .to(to
                .parse()
                .map_err(|_| TeamderError::Validation("Invalid email address".into()))?)
            .subject("Your Teamder verification code")
            .multipart(
                lettre::message::MultiPart::alternative_plain_html(text, html),
            )
            .map_err(|e| TeamderError::Internal(e.to_string()))?;

        cfg.transport
            .send(email)
            .await
            .map_err(|e| TeamderError::Internal(format!("Failed to send email: {e}")))?;
        Ok(())
    }
}

/// Minimal percent-encoding for the email query param (enough for addresses).
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            '@' => "%40".to_string(),
            '+' => "%2B".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
