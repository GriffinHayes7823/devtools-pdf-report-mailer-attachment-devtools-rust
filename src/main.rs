mod report_mailer;

use report_mailer::{render_pdf, send_report};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let to = args.next().ok_or("usage: cargo run -- <recipient> [project] [failed-checks]")?;
    let project = args.next().unwrap_or_else(|| "developer-tools".to_string());
    let failed_checks = args.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let _pdf = render_pdf(&project, failed_checks);
    let result = send_report(&to, &project, failed_checks).await?;
    println!("sent report message_id={}", result.message_id);
    Ok(())
}
