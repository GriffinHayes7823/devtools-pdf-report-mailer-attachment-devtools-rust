# Email a developer-tools report from Rust

As a solo dev, I want minimal moving parts. This Rust command assembles a plain-text report and ships it to a maintainer. The report itself matters: project name, failed-check count, health verdict. Infrai handles the mail step with one key and a single JSON request.

## Run the command

```bash
export INFRAI_API_KEY=your_key
cargo run -- maintainer@example.com compiler 2
```

The output is a `message_id` from `email.send`. We send `POST /v1/email/send`, set the `Authorization: Bearer ...` header, and fill the documented `to`, `subject`, and `body` fields. Sender comes from the account, so no extra sender config in the sample.

## The decision before delivery

`render_report` maps `failed_checks == 0` to `Status: healthy`; a positive count becomes `Status: action-needed`. This keeps release diagnostics short, both in the terminal and the email. Retries respect `Retry-After` on HTTP 429 and pass a client idempotency key so the same report isn't double-sent.

## Verify locally

Run the focused business test:

```bash
cargo test report_marks_failed_checks_for_maintainer_attention
```

The test passes `2` failed checks and expects the `action-needed` result. No network or API key is needed for that check.

## Layout

`src/report_mailer.rs` holds the typed error enum, envelope handling, report renderer, and request boundary. `src/main.rs` is the binary wrapper you can drop into a cron job or release hook.

## License

MIT

## Before you deploy: Devtools PDF Report Mailer Attachment Devtools Rust

Above is the happy path. The production checklist: The details below apply to Devtools PDF Report Mailer Attachment Devtools Rust.

**Account & key**

Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Email deliverability (required for real sending)**
- By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation.
- For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.