use crate::types::mail::SendEmail;
use reqwest::{Client, ClientBuilder};
use std::time::Instant;
use crate::config::config;

pub async fn send_email(email: SendEmail) -> Result<String, String> {
    let api = "https://api.resend.com/emails";
    let api_key = &config().resend_key;

    // Pre-serialize for logging + request body
    let payload = serde_json::to_string_pretty(&email)
        .map_err(|e| format!("serialize email failed: {e}"))?;

    println!("\n[mail] -> POST {api}");
    println!("[mail] payload:\n{payload}");

    let client: Client = ClientBuilder::new()
        .user_agent("ledger/1.0 (+reqwest)")
        .tcp_nodelay(true)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("build client failed: {e}"))?;

    // Build request so we can log it before send
    let req = client
        .post(api)
        .bearer_auth(api_key) // do NOT log full key
        .header("Content-Type", "application/json")
        .body(payload.clone())
        .build()
        .map_err(|e| format!("build request failed: {e}"))?;

    println!("[mail] request headers:");
    for (k, v) in req.headers().iter() {
        let vv = if k.as_str().eq_ignore_ascii_case("authorization") {
            "<redacted>"
        } else {
            v.to_str().unwrap_or("<non-utf8>")
        };
        println!("  {k}: {vv}");
    }
    println!("[mail] body bytes: {}", req.body().map(|b| b.as_bytes().map(|b| b.len()).unwrap_or(0)).unwrap_or(0));

    let t0 = Instant::now();
    let res = client.execute(req).await.map_err(|e| format!("send failed: {e}"))?;
    let dt = t0.elapsed();

    let status = res.status();
    let resp_headers = res.headers().clone();
    let body = res.text().await.map_err(|e| format!("read body failed: {e}"))?;

    println!("[mail] <- status: {status} in {} ms", dt.as_millis());
    println!("[mail] response headers:");
    for (k, v) in resp_headers.iter() {
        println!("  {k}: {}", v.to_str().unwrap_or("<non-utf8>"));
    }
    println!("[mail] response body:\n{body}");

    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("Resend API error: HTTP {status}: {body}"))
    }
}
