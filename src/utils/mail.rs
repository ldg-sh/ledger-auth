use crate::types::mail::SendEmail;
use reqwest::{Client, ClientBuilder};
use crate::config::config;

pub async fn send_email(email: SendEmail) -> Result<String, String> {
    let api = "https://api.resend.com/emails";
    let api_key = &config().resend_key;

    // Pre-serialize for logging + request body
    let payload = serde_json::to_string_pretty(&email)
        .map_err(|e| format!("serialize email failed: {e}"))?;

    let client: Client = ClientBuilder::new()
        .user_agent("ledger/1.0 (+reqwest)")
        .tcp_nodelay(true)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("build client failed: {e}"))?;

    let req = client
        .post(api)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .body(payload.clone())
        .build()
        .map_err(|e| format!("build request failed: {e}"))?;

    let res = client.execute(req).await.map_err(|e| format!("send failed: {e}"))?;

    let status = res.status();
    let body = res.text().await.map_err(|e| format!("read body failed: {e}"))?;

    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("Resend API error: HTTP {status}: {body}"))
    }
}
