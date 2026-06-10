use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};

use serde_json::Value;

const TAVERN_BASE: &str = "https://deepseektavern.com";

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ─────────────────────────────────────────────
// Auth helpers — deepseektavern.com uses session cookies, not Bearer tokens
// ─────────────────────────────────────────────

fn auth_headers(session_cookie: &str, user_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&format!("session={}", session_cookie)).unwrap(),
    );
    headers.insert(
        "New-Api-User",
        HeaderValue::from_str(user_id).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

// ─────────────────────────────────────────────
// Public endpoints (no auth required)
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn tavern_register(
    username: String,
    password: String,
    email: String,
    verification_code: String,
    aff_code: Option<String>,
) -> Result<Value, String> {
    let mut body = serde_json::json!({
        "username": username,
        "password": password,
        "email": email,
        "verification_code": verification_code,
    });
    if let Some(code) = aff_code {
        body["aff_code"] = serde_json::Value::String(code);
    }

    let resp = client()
        .post(format!("{}/api/user/register", TAVERN_BASE))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("注册失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_login(username: String, password: String) -> Result<Value, String> {
    let body = serde_json::json!({
        "username": username,
        "password": password,
    });

    let resp = client()
        .post(format!("{}/api/user/login", TAVERN_BASE))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    // Extract session cookie from Set-Cookie header before consuming response body
    let session_cookie = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|h| {
            let val = h.to_str().ok()?;
            if val.starts_with("session=") {
                val.split(';').next().and_then(|s| s.strip_prefix("session="))
            } else {
                None
            }
        })
        .unwrap_or("")
        .to_string();

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
    let body: Value = serde_json::from_str(&text).map_err(|e| format!("解析响应失败: {}", e))?;

    if status.is_success() && body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        let mut result = body.clone();
        if let Some(obj) = result.as_object_mut() {
            obj.insert("session_cookie".to_string(), serde_json::Value::String(session_cookie));
        }
        Ok(result)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("登录失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_send_verification_code(email: String) -> Result<Value, String> {
    let encoded = urlencoding(&email);
    let url = format!("{}/api/verification?email={}", TAVERN_BASE, encoded);
    tracing::info!("发送验证码: {} -> {}", email, url);

    let resp = client()
        .get(&url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
    tracing::info!("验证码响应: status={}, body={}", status.as_u16(), body);

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("发送验证码失败");
        Err(msg.to_string())
    }
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b'@' => result.push_str("%40"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

// ─────────────────────────────────────────────
// Authenticated endpoints (use session cookie + user_id)
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn tavern_get_self(session_cookie: String, user_id: String) -> Result<Value, String> {
    let resp = client()
        .get(format!("{}/api/user/self", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        Err("获取用户信息失败".to_string())
    }
}

#[tauri::command]
pub async fn tavern_get_tokens(
    session_cookie: String,
    user_id: String,
    page: Option<u32>,
    size: Option<u32>,
) -> Result<Value, String> {
    let p = page.unwrap_or(1);
    let s = size.unwrap_or(50);
    let url = format!("{}/api/token/?p={}&size={}", TAVERN_BASE, p, s);

    let resp = client()
        .get(&url)
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        Err("获取 Token 列表失败".to_string())
    }
}

#[tauri::command]
pub async fn tavern_create_token(
    session_cookie: String,
    user_id: String,
    name: String,
    remain_quota: Option<f64>,
    unlimited_quota: Option<bool>,
) -> Result<Value, String> {
    let mut body = serde_json::json!({
        "name": name,
    });
    if let Some(q) = remain_quota {
        body["remain_quota"] = serde_json::Value::Number(serde_json::Number::from_f64(q).unwrap());
    }
    if let Some(u) = unlimited_quota {
        body["unlimited_quota"] = serde_json::Value::Bool(u);
    }

    let resp = client()
        .post(format!("{}/api/token/", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        // Extract the full API key from the creation response
        // The Tavern API may return it in various nested paths.
        // Log the full body for debugging.
        tracing::info!("Tavern create token raw response: {}", serde_json::to_string(&body).unwrap_or_default());

        let extracted_key: String = {
            // Try common paths using dot notation (e.g., "data.key")
            let from_path = ["data.key", "data.sk", "data.token", "data.api_key", "key", "sk", "token"]
                .iter()
                .find_map(|path| {
                    let parts: Vec<&str> = path.split('.').collect();
                    let mut val = &body;
                    for part in parts {
                        val = val.get(part)?;
                    }
                    val.as_str()
                })
                .filter(|s| s.len() >= 30 && !s.contains('*'))
                .map(|s| s.to_string());

            from_path.unwrap_or_else(|| {
                // Last resort: walk the JSON tree to find any string that looks like a key
                fn find_key(v: &Value) -> Option<String> {
                    match v {
                        Value::String(s) if s.len() >= 30 && !s.contains('*') => Some(s.clone()),
                        Value::Array(arr) => arr.iter().find_map(find_key),
                        Value::Object(map) => map.values().find_map(find_key),
                        _ => None,
                    }
                }
                find_key(&body).unwrap_or_default()
            })
        };

        let mut enriched = body.clone();
        if let Some(obj) = enriched.as_object_mut() {
            obj.insert("extracted_key".to_string(), Value::String(extracted_key.to_string()));
        }
        tracing::info!("Token created, extracted key length: {}, starts_with: {}",
            extracted_key.len(),
            if extracted_key.len() >= 8 { &extracted_key[..8] } else { "" }
        );
        Ok(enriched)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("创建 Token 失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_delete_token(
    session_cookie: String,
    user_id: String,
    key_id: u64,
) -> Result<Value, String> {
    let resp = client()
        .delete(format!("{}/api/token/{}", TAVERN_BASE, key_id))
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("删除 Token 失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_update_token_status(
    session_cookie: String,
    user_id: String,
    key_id: u64,
    status: u8,
) -> Result<Value, String> {
    let body = serde_json::json!({
        "id": key_id,
        "status": status,
    });

    let resp = client()
        .put(format!("{}/api/token/?status_only=true", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("更新失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_get_token_by_name(
    session_cookie: String,
    user_id: String,
    name: String,
) -> Result<Value, String> {
    let url = format!(
        "{}/api/token/search?keyword={}",
        TAVERN_BASE,
        urlencoding(&name)
    );

    let resp = client()
        .get(&url)
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        Err("搜索 Token 失败".to_string())
    }
}

#[tauri::command]
pub async fn tavern_topup(
    session_cookie: String,
    user_id: String,
    key: String,
) -> Result<Value, String> {
    let body = serde_json::json!({ "key": key });

    let resp = client()
        .post(format!("{}/api/user/topup", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("兑换失败");
        Err(msg.to_string())
    }
}

// ─────────────────────────────────────────────
// Payment endpoints
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn tavern_calc_amount(
    session_cookie: String,
    user_id: String,
    amount: u64,
) -> Result<Value, String> {
    let body = serde_json::json!({ "amount": amount });

    let resp = client()
        .post(format!("{}/api/user/amount", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("计算失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_create_payment(
    session_cookie: String,
    user_id: String,
    amount: u64,
    payment_method: String,
) -> Result<Value, String> {
    let body = serde_json::json!({
        "amount": amount,
        "payment_method": payment_method,
    });
    tracing::info!("创建支付: amount={}, method={}", amount, payment_method);

    let resp = client()
        .post(format!("{}/api/user/pay", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;
    tracing::info!("支付响应: url={}, message={:?}", 
        body.get("url").and_then(|u| u.as_str()).unwrap_or("none"),
        body.get("message").and_then(|m| m.as_str()).unwrap_or("none"));

    let is_success = body.get("success").and_then(|s| s.as_bool()).unwrap_or(false)
        || body.get("message").and_then(|m| m.as_str()).map(|m| m == "success").unwrap_or(false)
        || body.get("url").is_some();

    if is_success {
        // Write payment form to temp HTML and open in browser
        if let (Some(url), Some(data)) = (body.get("url").and_then(|u| u.as_str()), body.get("data")) {
            let mut html = String::from(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>支付跳转</title></head><body>"
            );
            html.push_str(&format!("<form id=\"pay\" action=\"{}\" method=\"POST\">", url));
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    let val = v.as_str().unwrap_or("");
                    html.push_str(&format!("<input type=\"hidden\" name=\"{}\" value=\"{}\">", k, val));
                }
            }
            html.push_str("</form><script>document.getElementById('pay').submit();</script></body></html>");

            let temp_dir = std::env::temp_dir();
            let file_path = temp_dir.join("tavern-payment.html");
            if std::fs::write(&file_path, html).is_ok() {
                // Open in default browser via OS command
                #[cfg(target_os = "macos")]
                { let _ = std::process::Command::new("open").arg(&file_path).spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn(); }
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("cmd").args(["/c", "start", ""]).arg(&file_path).spawn(); }
                tracing::info!("支付页面已打开: {}", file_path.display());
            }
        }
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty() && *m != "success").unwrap_or("创建支付失败");
        Err(msg.to_string())
    }
}

#[tauri::command]
pub async fn tavern_get_token_detail(
    session_cookie: String,
    user_id: String,
    key_id: u64,
) -> Result<Value, String> {
    let resp = client()
        .get(format!("{}/api/token/{}", TAVERN_BASE, key_id))
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        tracing::info!("Token detail response: {}", serde_json::to_string(&body).unwrap_or_default());
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("获取 Token 详情失败");
        Err(msg.to_string())
    }
}

/// Open deepseektavern.com/console in the system default browser.
/// Simpler and more reliable than an embedded webview — cookies persist naturally.
#[tauri::command]
pub async fn open_tavern_key_webview() -> Result<(), String> {
    let url = format!("{}/console", TAVERN_BASE);
    tracing::info!("Opening Tavern Console in browser: {}", url);

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", ""])
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn tavern_get_models(
    session_cookie: String,
    user_id: String,
) -> Result<Value, String> {
    let resp = client()
        .get(format!("{}/api/user/models", TAVERN_BASE))
        .headers(auth_headers(&session_cookie, &user_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    if body.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        Ok(body)
    } else {
        let msg = body.get("message").and_then(|m| m.as_str()).filter(|m| !m.is_empty()).unwrap_or("获取模型列表失败");
        Err(msg.to_string())
    }
}
