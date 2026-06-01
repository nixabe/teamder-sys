//! Standalone SMTP credential / connection validator.
//!
//! Reads the same `SMTP_*` env vars as the API and sends one test verification
//! email through the production `Mailer` (identical TLS 1.2 settings), so you
//! can validate SMTP config without running the whole app or the frontend.
//!
//! Usage (from the teamder-sys workspace root, where `.env` lives):
//!   cargo run -p teamder-api --bin smtp_check -- [recipient@example.com]
//!
//! If no recipient is given it falls back to `SMTP_FROM` (i.e. emails you).
//! Exit codes: 0 = sent OK, 1 = send failed, 2 = SMTP not configured.

use teamder_api::mailer::Mailer;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let host = std::env::var("SMTP_HOST").unwrap_or_default();
    let port = std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".into());
    let username = std::env::var("SMTP_USERNAME").unwrap_or_default();
    let password = std::env::var("SMTP_PASSWORD").unwrap_or_default();
    let from = std::env::var("SMTP_FROM")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| username.clone());

    let to = std::env::args().nth(1).unwrap_or_else(|| from.clone());

    println!("SMTP config:");
    println!("  host     = {host}");
    println!("  port     = {port}");
    println!("  username = {username}");
    println!(
        "  password = {} (length {})",
        if password.is_empty() { "<empty>" } else { "<set>" },
        password.len()
    );
    println!("  from     = {from}");
    println!("  to       = {to}");
    println!();

    let mailer = Mailer::from_env();
    if !mailer.is_live() {
        eprintln!("✗ SMTP is not configured (SMTP_HOST is empty) — the mailer is in dev mode.");
        eprintln!("  Set SMTP_HOST + credentials in .env to test real delivery.");
        std::process::exit(2);
    }

    println!("Sending test verification email to {to} …");
    match mailer.send_code(&to, "123456", "login").await {
        Ok(()) => {
            println!("✓ SUCCESS — the server accepted the message. Check the inbox for {to}.");
        }
        Err(e) => {
            eprintln!("✗ FAILED — {e}");
            eprintln!();
            eprintln!("Hints:");
            eprintln!("  • 535 Authentication Failed → the username/token was rejected. For");
            eprintln!("    ZeptoMail: username must be 'emailapikey' and password the ACTIVE");
            eprintln!("    Send Mail Token from the SAME region host shown in the console");
            eprintln!("    (smtp.zeptomail.com / .eu / .in). The token must belong to the Mail");
            eprintln!("    Agent that owns the verified sender domain.");
            eprintln!("  • connection refused / timeout → wrong host or port, or a firewall.");
            eprintln!("  • invalid peer certificate → host/region or system root certs.");
            std::process::exit(1);
        }
    }
}
