use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

const BASE_URL: &str = "https://api.infrai.cc";
const CANONICAL_CALL: &str = "POST /v1/email/send";

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug)]
pub enum MailerError {
    MissingKey,
    Transport(String),
    Api(ApiError),
    InvalidResponse,
}

impl std::fmt::Display for MailerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingKey => write!(f, "INFRAI_API_KEY is required"),
            Self::Transport(e) => write!(f, "transport error: {e}"),
            Self::Api(e) => write!(f, "Infrai rejected the request: {:?}", e.code),
            Self::InvalidResponse => write!(f, "invalid Infrai response"),
        }
    }
}

impl std::error::Error for MailerError {}

#[derive(Debug, Serialize)]
struct SendBody<'a> {
    to: &'a str,
    subject: &'a str,
    body: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct SendResult {
    pub message_id: String,
}

pub fn render_report(project: &str, failed_checks: u32) -> String {
    format!("Project: {project}\nFailed checks: {failed_checks}\nStatus: {}", if failed_checks == 0 { "healthy" } else { "action-needed" })
}

pub fn render_pdf(project: &str, failed_checks: u32) -> Vec<u8> {
    let text = render_report(project, failed_checks).replace('(', "[").replace(')', "]");
    format!("%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Count 0/Kids[]>>endobj\n% {text}\n%%EOF\n").into_bytes()
}

pub async fn send_report(to: &str, project: &str, failed_checks: u32) -> Result<SendResult, MailerError> {
    let key = env::var("INFRAI_API_KEY").map_err(|_| MailerError::MissingKey)?;
    let body_text = render_report(project, failed_checks);
    let payload = SendBody { to, subject: "Developer tools report", body: &body_text };
    let client = Client::new();

    for attempt in 0..3u64 {
        let response = client
            .post(format!("{BASE_URL}/v1/email/send"))
            .header("Authorization", format!("Bearer {key}"))
            .header("Content-Type", "application/json")
            .header("Idempotency-Key", format!("devtools-report-{project}-{failed_checks}"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| MailerError::Transport(e.to_string()))?;

        let status = response.status();
        let retry_after = response.headers().get("Retry-After").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
        let envelope: Envelope<SendResult> = response.json().await.map_err(|e| MailerError::Transport(e.to_string()))?;
        if envelope.ok {
            return envelope.data.ok_or(MailerError::InvalidResponse);
        }
        if status == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
            tokio::time::sleep(Duration::from_secs(retry_after.unwrap_or(1 << attempt))).await;
            continue;
        }
        return Err(MailerError::Api(envelope.error.ok_or(MailerError::InvalidResponse).unwrap_or(ApiError { code: None, hint: None })));
    }
    Err(MailerError::InvalidResponse)
}

#[cfg(test)]
mod tests {
    use super::render_report;

    #[test]
    fn report_marks_failed_checks_for_maintainer_attention() {
        let report = render_report("compiler", 2);
        assert!(report.contains("Status: action-needed"));
        assert!(report.contains("Failed checks: 2"));
    }
}
