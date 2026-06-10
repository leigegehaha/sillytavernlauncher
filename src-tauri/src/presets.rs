use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

// ── 数据结构 ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresetFile {
    pub file_name: String,
    pub category: String,
    pub size: u64,
    pub modified_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegexFile {
    pub file_name: String,
    pub size: u64,
    pub modified_ms: Option<u64>,
}

// ── 预设目录 ─────────────────────────────────────────────

fn get_presets_dir(app: &AppHandle) -> PathBuf {
    let st_data = crate::utils::get_st_data_dir(app);
    st_data.join("presets")
}

fn get_regex_dir(app: &AppHandle) -> PathBuf {
    let st_data = crate::utils::get_st_data_dir(app);
    st_data.join("extensions").join("regex")
}

fn get_preset_category_dir(app: &AppHandle, category: &str) -> PathBuf {
    get_presets_dir(app).join(category)
}

/// 预设子目录列表（这些是 SillyTavern 的内置预设类型）
const PRESET_CATEGORIES: &[&str] = &[
    "context",
    "instruct",
    "sysprompt",
    "kobold",
    "novel",
    "openai",
    "textgen",
    "reasoning",
];

// ── 辅助：扫描目录中的 .json 文件 ────────────────────────

fn scan_json_files(dir: &PathBuf) -> Result<Vec<PresetFile>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("json") {
                    let meta = entry.metadata().ok();
                    result.push(PresetFile {
                        file_name: path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                        modified_ms: meta.and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as u64),
                        category: String::new(), // filled by caller
                    });
                }
            }
        }
    }
    result.sort_by(|a, b| b.modified_ms.unwrap_or(0).cmp(&a.modified_ms.unwrap_or(0)));
    Ok(result)
}

// ── Tauri Commands: 预设 ─────────────────────────────────

#[tauri::command]
pub async fn list_presets(app: AppHandle) -> Result<Vec<PresetFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let presets_dir = get_presets_dir(&app_clone);
        let mut all = Vec::new();

        for cat in PRESET_CATEGORIES {
            let cat_dir = presets_dir.join(cat);
            if cat_dir.exists() {
                if let Ok(mut files) = scan_json_files(&cat_dir) {
                    for f in &mut files {
                        f.category = cat.to_string();
                    }
                    all.append(&mut files);
                }
            }
        }
        Ok(all)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_preset_file(
    app: AppHandle,
    source_path: String,
    category: String,
    file_name: Option<String>,
) -> Result<(), String> {
    if !PRESET_CATEGORIES.contains(&category.as_str()) {
        return Err(format!("不支持的预设类型: {}", category));
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let src = PathBuf::from(&source_path);
        if !src.exists() {
            return Err("源文件不存在".to_string());
        }
        if src.extension().map(|e| e.to_ascii_lowercase()) != Some("json".into()) {
            return Err("只支持 .json 格式的预设文件".to_string());
        }

        let target_name = file_name.unwrap_or_else(|| {
            src.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let cat_dir = get_preset_category_dir(&app_clone, &category);
        if !cat_dir.exists() {
            fs::create_dir_all(&cat_dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let dest = cat_dir.join(&target_name);
        if dest.exists() {
            return Err(format!("同名预设 {} 已存在", target_name));
        }

        fs::copy(&src, &dest).map_err(|e| format!("复制文件失败: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_preset_file(
    app: AppHandle,
    category: String,
    file_name: String,
) -> Result<String, String> {
    if !PRESET_CATEGORIES.contains(&category.as_str()) {
        return Err("不支持的预设类型".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let path = get_preset_category_dir(&app_clone, &category).join(&file_name);
        if !path.exists() {
            return Err("文件不存在".to_string());
        }
        fs::read_to_string(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_presets(
    app: AppHandle,
    items: Vec<PresetFile>,
) -> Result<(), String> {
    if items.is_empty() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let mut errors = Vec::new();
        for item in items {
            if item.file_name.contains("..") || item.file_name.contains('/')
                || item.file_name.contains('\\')
            {
                errors.push(format!("跳过非法文件名: {}", item.file_name));
                continue;
            }
            let path = get_preset_category_dir(&app_clone, &item.category).join(&item.file_name);
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    errors.push(format!("删除 {} 失败: {}", item.file_name, e));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── Tauri Commands: 正则 ─────────────────────────────────

#[tauri::command]
pub async fn list_regex_scripts(app: AppHandle) -> Result<Vec<RegexFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_regex_dir(&app_clone);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("js") {
                        let meta = entry.metadata().ok();
                        result.push(RegexFile {
                            file_name: path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                            modified_ms: meta.and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_millis() as u64),
                        });
                    }
                }
            }
        }
        result.sort_by(|a, b| b.modified_ms.unwrap_or(0).cmp(&a.modified_ms.unwrap_or(0)));
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_regex_script(
    app: AppHandle,
    source_path: String,
    file_name: Option<String>,
) -> Result<(), String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let src = PathBuf::from(&source_path);
        if !src.exists() {
            return Err("源文件不存在".to_string());
        }
        let ext = src.extension()
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        if ext != "json" && ext != "js" {
            return Err("只支持 .json 或 .js 格式的正则脚本".to_string());
        }

        let target_name = file_name.unwrap_or_else(|| {
            src.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let dir = get_regex_dir(&app_clone);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let dest = dir.join(&target_name);
        if dest.exists() {
            return Err(format!("同名正则脚本 {} 已存在", target_name));
        }

        fs::copy(&src, &dest).map_err(|e| format!("复制文件失败: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_regex_scripts(
    app: AppHandle,
    file_names: Vec<String>,
) -> Result<(), String> {
    if file_names.is_empty() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_regex_dir(&app_clone);
        let mut errors = Vec::new();
        for name in file_names {
            if name.contains("..") || name.contains('/') || name.contains('\\') {
                errors.push(format!("跳过非法文件名: {}", name));
                continue;
            }
            let path = dir.join(&name);
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    errors.push(format!("删除 {} 失败: {}", name, e));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
