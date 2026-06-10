use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::types::CharacterCardFile;

// ─────────────────────────────────────────────
// 内部辅助：角色卡目录
// ─────────────────────────────────────────────

fn get_character_cards_dir(app: &AppHandle) -> PathBuf {
    let data_dir = crate::utils::get_st_data_dir(app);
    data_dir.join("default-user").join("characters")
}

// ─────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────

#[tauri::command]
pub async fn list_character_card_pngs(app: AppHandle) -> Result<Vec<CharacterCardFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_character_cards_dir(&app_clone);
        if !dir.exists() {
            return Err("DIR_NOT_FOUND".to_string());
        }

        let mut result = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = match entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            let file_type = match entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }
            let path = entry.path();
            let ext_ok = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("png"))
                .unwrap_or(false);
            if !ext_ok {
                continue;
            }
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(v) => v.to_string(),
                None => continue,
            };

            let meta = match entry.metadata() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let modified_ms = meta.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_millis() as i64)
            });

            result.push(CharacterCardFile {
                file_name,
                size: meta.len(),
                modified_ms,
                category: None,
            });
        }

        result.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_character_card_png(app: AppHandle, file_name: String) -> Result<Vec<u8>, String> {
    if file_name.trim().is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if !file_name.to_lowercase().ends_with(".png") {
        return Err("仅支持 .png 文件".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_character_cards_dir(&app_clone);
        let file_path = dir.join(&file_name);
        if !file_path.exists() {
            return Err("文件不存在".to_string());
        }
        fs::read(&file_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_character_cards(app: AppHandle, file_names: Vec<String>) -> Result<(), String> {
    if file_names.is_empty() {
        return Ok(());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_character_cards_dir(&app_clone);
        let mut errors = Vec::new();

        for file_name in file_names {
            if file_name.trim().is_empty()
                || file_name.contains("..")
                || file_name.contains('/')
                || file_name.contains('\\')
            {
                errors.push(format!("文件名不合法: {}", file_name));
                continue;
            }

            let file_path = dir.join(&file_name);
            if file_path.exists() {
                if let Err(e) = fs::remove_file(&file_path) {
                    errors.push(format!("无法删除 {}: {}", file_name, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_character_card(app: AppHandle, source_path: String) -> Result<(), String> {
    if source_path.trim().is_empty() {
        return Err("源路径不能为空".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let source = std::path::PathBuf::from(&source_path);
        if !source.exists() || !source.is_file() {
            return Err("源文件不存在或不是文件".to_string());
        }

        let ext = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "png" {
            return Err("只支持导入 png 格式的角色卡".to_string());
        }

        let file_name = source
            .file_name()
            .ok_or("无效的文件名")?
            .to_string_lossy()
            .to_string();

        let dir = get_character_cards_dir(&app_clone);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }

        let target_path = dir.join(&file_name);
        if target_path.exists() {
            return Err("同名角色卡已存在，请重命名后再导入".to_string());
        }

        fs::copy(&source, &target_path).map_err(|e| format!("复制文件失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_local_file(path: String) -> Result<Vec<u8>, String> {
    tokio::task::spawn_blocking(move || {
        let file_path = PathBuf::from(&path);
        if !file_path.exists() || !file_path.is_file() {
            return Err("文件不存在".to_string());
        }
        fs::read(&file_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn import_character_card_from_bytes(
    app: AppHandle,
    bytes: Vec<u8>,
    filename: String,
) -> Result<(), String> {
    // 1. 基本安全和格式校验
    if filename.trim().is_empty() {
        return Err("文件名不能为空".to_string());
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if !filename.to_lowercase().ends_with(".png") {
        return Err("只支持导入 png 格式的角色卡".to_string());
    }

    let app_clone = app.clone();

    // 2. 放入 blocking 线程池执行文件 I/O 操作
    tokio::task::spawn_blocking(move || {
        // 获取目标目录
        let dir = get_character_cards_dir(&app_clone);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // 拼接目标文件路径
        let target_path = dir.join(&filename);

        // 查重：防止覆盖现有角色卡
        if target_path.exists() {
            return Err("同名角色卡已存在，请重命名后再导入".to_string());
        }

        // 3. 将传入的字节流写入文件
        fs::write(&target_path, bytes).map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─────────────────────────────────────────────
// 预设资源（bundled presets）
// ─────────────────────────────────────────────

fn get_bundled_presets_dir(app: &AppHandle) -> PathBuf {
    let resource_dir = app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    resource_dir.join("presets")
}

fn get_user_presets_dir(app: &AppHandle) -> PathBuf {
    let data_dir = crate::utils::get_st_data_dir(app);
    data_dir.join("default-user").join("settings")
}

#[tauri::command]
pub async fn list_bundled_presets(app: AppHandle) -> Result<Vec<CharacterCardFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let base_dir = get_bundled_presets_dir(&app_clone);
        if !base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let category_entries = fs::read_dir(&base_dir).map_err(|e| e.to_string())?;

        for cat_entry in category_entries {
            let cat_entry = match cat_entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !cat_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let category = match cat_entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let cat_path = cat_entry.path();
            let file_entries = match fs::read_dir(&cat_path) {
                Ok(v) => v,
                Err(_) => continue,
            };

            for file_entry in file_entries {
                let file_entry = match file_entry {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if !file_entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let path = file_entry.path();
                let ext_ok = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case("json"))
                    .unwrap_or(false);
                if !ext_ok {
                    continue;
                }
                let file_name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(v) => v.to_string(),
                    None => continue,
                };
                let meta = match file_entry.metadata() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                result.push(CharacterCardFile {
                    file_name,
                    size: meta.len(),
                    modified_ms: None,
                    category: Some(category.clone()),
                });
            }
        }

        result.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_bundled_preset(
    app: AppHandle,
    category: String,
    file_name: String,
) -> Result<(), String> {
    if file_name.trim().is_empty() || category.trim().is_empty() {
        return Err("参数不能为空".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if category.contains("..") || category.contains('/') || category.contains('\\') {
        return Err("分类名不合法".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let bundled_dir = get_bundled_presets_dir(&app_clone);
        let source_path = bundled_dir.join(&category).join(&file_name);
        if !source_path.exists() {
            return Err("预设文件不存在".to_string());
        }

        let target_dir = get_user_presets_dir(&app_clone);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // Presets go to settings/AI配置名称.json
        let target_path = target_dir.join(&file_name);
        if target_path.exists() {
            return Err("同名预设已存在".to_string());
        }

        fs::copy(&source_path, &target_path).map_err(|e| format!("复制失败: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn get_bundled_cards_dir(app: &AppHandle) -> PathBuf {
    let resource_dir = app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    resource_dir.join("character-cards")
}

#[tauri::command]
pub async fn list_bundled_cards(app: AppHandle) -> Result<Vec<CharacterCardFile>, String> {
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let base_dir = get_bundled_cards_dir(&app_clone);
        if !base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let category_entries = fs::read_dir(&base_dir).map_err(|e| e.to_string())?;

        for cat_entry in category_entries {
            let cat_entry = match cat_entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !cat_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let category = match cat_entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let cat_path = cat_entry.path();
            let card_entries = match fs::read_dir(&cat_path) {
                Ok(v) => v,
                Err(_) => continue,
            };

            for card_entry in card_entries {
                let card_entry = match card_entry {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if !card_entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let path = card_entry.path();
                let ext_ok = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.eq_ignore_ascii_case("png") || s.eq_ignore_ascii_case("webp"))
                    .unwrap_or(false);
                if !ext_ok {
                    continue;
                }
                let file_name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(v) => v.to_string(),
                    None => continue,
                };
                let meta = match card_entry.metadata() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                result.push(CharacterCardFile {
                    file_name,
                    size: meta.len(),
                    modified_ms: None,
                    category: Some(category.clone()),
                });
            }
        }

        result.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_bundled_card_thumb(
    app: AppHandle,
    category: String,
    file_name: String,
) -> Result<Vec<u8>, String> {
    if file_name.trim().is_empty() || category.trim().is_empty() {
        return Err("参数不能为空".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if category.contains("..") || category.contains('/') || category.contains('\\') {
        return Err("分类名不合法".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let dir = get_bundled_cards_dir(&app_clone);
        let file_path = dir.join(&category).join(&file_name);
        if !file_path.exists() {
            return Err("文件不存在".to_string());
        }
        fs::read(&file_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_bundled_card(
    app: AppHandle,
    category: String,
    file_name: String,
) -> Result<(), String> {
    if file_name.trim().is_empty() || category.trim().is_empty() {
        return Err("参数不能为空".to_string());
    }
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err("文件名不合法".to_string());
    }
    if category.contains("..") || category.contains('/') || category.contains('\\') {
        return Err("分类名不合法".to_string());
    }

    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let bundled_dir = get_bundled_cards_dir(&app_clone);
        let source_path = bundled_dir.join(&category).join(&file_name);
        if !source_path.exists() {
            return Err("预设文件不存在".to_string());
        }

        let target_dir = get_character_cards_dir(&app_clone);
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let target_path = target_dir.join(&file_name);
        if target_path.exists() {
            return Err("同名角色卡已存在".to_string());
        }

        fs::copy(&source_path, &target_path).map_err(|e| format!("复制失败: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
