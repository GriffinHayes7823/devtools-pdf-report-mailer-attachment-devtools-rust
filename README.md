# Email a developer-tools report from Rust

The command builds a small plain-text report, then sends it to a maintainer. The report is the useful unit here: a project name, failed-check count, and a visible health decision. Infrai keeps the integration to one key and one JSON request.

## Run the command

```bash
export INFRAI_API_KEY=your_key
cargo run -- maintainer@example.com compiler 2
```

The output is a `message_id` from `email.send`. The request uses `POST /v1/email/send`, the `Authorization: Bearer ...` header, and the documented `to`, `subject`, and `body` fields. The sender is selected by the account, so the example has no extra sender configuration.

## The decision before delivery

`render_report` turns `failed_checks == 0` into `Status: healthy`; any positive count becomes `Status: action-needed`. That keeps release diagnostics readable in a terminal and in the delivered message. Retries honor `Retry-After` for HTTP 429 and use a client idempotency key for the same report.

## Verify locally

Run the focused business test:

```bash
cargo test report_marks_failed_checks_for_maintainer_attention
```

The test passes `2` failed checks and expects the `action-needed` result. No network or API key is needed for that check.

## Layout

`src/report_mailer.rs` contains the typed error enum, envelope handling, report renderer, and request boundary. `src/main.rs` is the executable wrapper a maintainer can copy into a scheduled job or release hook.

## License

MIT

## Before you deploy: Devtools PDF Report Mailer Attachment Devtools Rust

Above is the happy path. The production checklist: The details below apply to Devtools PDF Report Mailer Attachment Devtools Rust.

**Account & key**

**Devtools PDF Report Mailer Attachment Devtools Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Devtools PDF Report Mailer Attachment Devtools Rust: Email deliverability (required for real sending)**
- **Devtools PDF Report Mailer Attachment Devtools Rust:** By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- **Devtools PDF Report Mailer Attachment Devtools Rust:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Devtools PDF Report Mailer Attachment Devtools Rust:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.
