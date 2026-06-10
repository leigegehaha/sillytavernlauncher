use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

fn get_secrets_path(app: &AppHandle) -> PathBuf {
    let data_dir = crate::utils::get_st_data_dir(app);
    data_dir.join("default-user").join("secrets.json")
}

#[tauri::command]
pub async fn read_secrets(app: AppHandle) -> Result<Value, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_secrets_path(&app_clone);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&path, "{}").ok();
            return Ok(serde_json::json!({}));
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let val: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(val)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn write_secrets(app: AppHandle, secrets: Value) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_secrets_path(&app_clone);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Read existing secrets to merge
        let mut merged: serde_json::Map<String, Value> = if path.exists() {
            let existing = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&existing).unwrap_or(serde_json::Map::new())
        } else {
            serde_json::Map::new()
        };

        // Helper: write a value in SillyTavern's structured format
        // [{ "id": "uuid", "value": "...", "label": "Tavern Launcher", "active": true }]
        let make_secret = |value: &str| -> Value {
            serde_json::json!([{
                "id": uuid::Uuid::new_v4().to_string(),
                "value": value,
                "label": "Tavern Launcher",
                "active": true,
            }])
        };

        // Merge our custom provider config into SillyTavern format (structured array)
        // When chat_completion_source = "custom", SillyTavern reads:
        //   - api_key_custom  (from secrets.json)
        //   - custom_url      (from settings.json → oai_settings.custom_url)
        //   - custom_model    (from settings.json → oai_settings.custom_model)
        let mut last_active_endpoint: Option<String> = None;
        let mut last_active_model: Option<String> = None;

        if let Some(providers) = secrets.as_object() {
            for (key, cfg) in providers {
                if let (Some(api_key), Some(endpoint), Some(model)) = (
                    cfg.get("apiKey").and_then(|v| v.as_str()),
                    cfg.get("endpoint").and_then(|v| v.as_str()),
                    cfg.get("model").and_then(|v| v.as_str()),
                ) {
                    if !api_key.is_empty() {
                        // CRITICAL: Check for masked keys (contain asterisks like "NhP4****KCOs")
                        // The frontend may pass masked keys from token lists. Reject them.
                        if api_key.contains('*') {
                            tracing::warn!(
                                "Skipping masked API key for provider '{}': value contains asterisks. \
                                 This key was likely copied from a token list that masks the full key.",
                                key
                            );
                            // Don't write this masked key to SillyTavern secrets
                            // Also don't update deepseek_tavern.apiKey with the masked value
                            continue;
                        }
                        if key == "deepseek_tavern" {
                            // DeepSeek Tavern uses OpenAI-compatible API → "custom" source in SillyTavern
                            let endpoint_url = format!("{}/v1", endpoint.trim_end_matches('/').trim_end_matches("/v1"));
                            merged.insert("api_key_custom".to_string(), make_secret(api_key));
                            last_active_endpoint = Some(endpoint_url);
                            last_active_model = if model.is_empty() { None } else { Some(model.to_string()) };
                        } else if key == "openai" {
                            // Standard OpenAI → "openai" source
                            merged.insert("api_key_openai".to_string(), make_secret(api_key));
                        }
                    }
                }
            }
        }

        // Also preserve our custom provider config for the launcher UI
        for (key, value) in secrets.as_object().unwrap_or(&serde_json::Map::new()) {
            merged.insert(key.clone(), value.clone());
        }

        let content = serde_json::to_string_pretty(&merged).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;

        // ─── Update settings.json → oai_settings for the "custom" API source ───
        // SillyTavern reads connection config from settings.json, NOT secrets.json.
        // We must write to oai_settings.chat_completion_source / custom_url / custom_model.
        // CRITICAL: These go INSIDE the oai_settings object, NOT at the top level.
        if let Some(endpoint_url) = last_active_endpoint {
            let model_str = last_active_model.as_deref().unwrap_or("");
            if let Some(parent) = path.parent() {
                let settings_path = parent.join("settings.json");
                if settings_path.exists() {
                    if let Ok(settings_content) = fs::read_to_string(&settings_path) {
                        if let Ok(mut settings) = serde_json::from_str::<serde_json::Map<String, Value>>(&settings_content) {
                            let mut changed = false;

                            // Ensure main_api is "openai" (SillyTavern's chat completions mode)
                            if settings.get("main_api").and_then(|v| v.as_str()) != Some("openai") {
                                settings.insert("main_api".to_string(), Value::String("openai".to_string()));
                                changed = true;
                            }

                            // Update oai_settings (nested!)
                            if let Some(oai_val) = settings.get_mut("oai_settings") {
                                if let Some(oai) = oai_val.as_object_mut() {
                                    if oai.get("chat_completion_source").and_then(|v| v.as_str()) != Some("custom") {
                                        oai.insert("chat_completion_source".to_string(), Value::String("custom".to_string()));
                                        changed = true;
                                    }
                                    if oai.get("custom_url").and_then(|v| v.as_str()) != Some(&endpoint_url) {
                                        oai.insert("custom_url".to_string(), Value::String(endpoint_url.clone()));
                                        changed = true;
                                    }
                                    if !model_str.is_empty() && oai.get("custom_model").and_then(|v| v.as_str()) != Some(model_str) {
                                        oai.insert("custom_model".to_string(), Value::String(model_str.to_string()));
                                        changed = true;
                                    }
                                }
                            }

                            if changed {
                                if let Ok(new_content) = serde_json::to_string_pretty(&settings) {
                                    let _ = fs::write(&settings_path, new_content);
                                    tracing::info!("Updated settings.json oai_settings for custom API source");
                                }
                            }
                        }
                    }
                } else {
                    // settings.json doesn't exist — create from default template
                    tracing::warn!("settings.json missing, creating from default template");
                    crate::sillytavern::generate_default_settings_for_version(&app_clone, "1.18.0")?;

                    // Now update the newly created settings.json with our custom source config
                    if let Ok(settings_content) = fs::read_to_string(&settings_path) {
                        if let Ok(mut settings) = serde_json::from_str::<serde_json::Map<String, Value>>(&settings_content) {
                            settings.insert("main_api".to_string(), Value::String("openai".to_string()));
                            if let Some(oai_val) = settings.get_mut("oai_settings") {
                                if let Some(oai) = oai_val.as_object_mut() {
                                    oai.insert("chat_completion_source".to_string(), Value::String("custom".to_string()));
                                    oai.insert("custom_url".to_string(), Value::String(endpoint_url.clone()));
                                    if !model_str.is_empty() {
                                        oai.insert("custom_model".to_string(), Value::String(model_str.to_string()));
                                    }
                                }
                            }
                            if let Ok(new_content) = serde_json::to_string_pretty(&settings) {
                                let _ = fs::write(&settings_path, new_content);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn test_api_connection(endpoint: String, api_key: String) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let status = resp.status();
    if status.is_success() || status.as_u16() == 401 {
        // 401 means the endpoint is reachable but key is wrong - still a successful connection
        Ok(serde_json::json!({ "ok": true, "msg": "API 端点可访问" }))
    } else {
        Ok(serde_json::json!({ "ok": false, "msg": format!("HTTP {}", status.as_u16()) }))
    }
}

#[tauri::command]
pub async fn fetch_model_list(endpoint: String, api_key: String) -> Result<Value, String> {
    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {} — 无法获取模型列表", status.as_u16()));
    }

    let body: Value = resp.json().await.map_err(|e| format!("解析响应失败: {}", e))?;

    // OpenAI-compatible format: { data: [{ id: "model-name", ... }] }
    let models: Vec<String> = if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        data.iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
            .collect()
    } else {
        return Err("未识别的模型列表格式".to_string());
    };

    if models.is_empty() {
        return Err("端点返回空模型列表".to_string());
    }

    Ok(serde_json::json!({ "models": models }))
}
