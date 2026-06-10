// ─────────────────────────────────────────────────────────────────────────────
// 模块声明
// ─────────────────────────────────────────────────────────────────────────────
pub mod character;
pub mod chat;
pub mod config;
pub mod elevation;
pub mod extensions;
pub mod finderst;
pub mod git;
pub mod node;
pub mod presets;
pub mod sillytavern;
pub mod types;
pub mod utils;
pub mod worldinfo;
pub mod secrets;
pub mod tavern_api;

// ─────────────────────────────────────────────────────────────────────────────
// 顶层 use
// ─────────────────────────────────────────────────────────────────────────────
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::config::{apply_saved_window_position, setup_window_position_tracking};
use crate::types::{InstallState, ProcessState};
use crate::utils::{ensure_standard_layout, init_logger};
#[cfg(target_os = "macos")]
use crate::utils::migrate_macos_data_if_needed;

/// 在 setup 中提前构建、通过 manage 传递给 run 回调，解决生命周期问题
struct OwnedArcs {
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    git_child_pid: Arc<Mutex<Option<u32>>>,
}

#[allow(unused_variables)]
fn resolve_app_working_dir(app: &tauri::AppHandle) -> PathBuf {
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取可执行文件路径失败: {e}，回退到当前工作目录");
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    };

    #[cfg(target_os = "macos")]
    {
        let exe_str = exe_path.to_string_lossy();
        if exe_str.contains(".app/Contents/MacOS/") {
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                return app_data_dir;
            }
        }
    }

    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let looks_like_target_build = {
        let components: Vec<String> = exe_dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .collect();
        components
            .windows(2)
            .any(|pair| pair[0] == "target" && (pair[1] == "debug" || pair[1] == "release"))
    };

    if cfg!(debug_assertions) || looks_like_target_build {
        let mut cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if cwd.ends_with("src-tauri") {
            cwd.pop();
        }
        cwd
    } else {
        exe_dir
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 应用入口
// ─────────────────────────────────────────────────────────────────────────────
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let git_child_pid_arc = Arc::new(Mutex::new(None::<u32>));
            let cancel_flag_arc = Arc::new(std::sync::atomic::AtomicBool::new(false));

            app.manage(ProcessState {
                kill_tx: Arc::new(Mutex::new(None)),
                child_pid: Arc::new(Mutex::new(None)),
            });
            app.manage(InstallState {
                cancel_flag: cancel_flag_arc.clone(),
                git_child_pid: git_child_pid_arc.clone(),
            });

            // 把 Arc 存到 app state 中供 run 回调捕获（用 OwnedArcs 包装）
            app.manage(OwnedArcs {
                cancel_flag: cancel_flag_arc,
                git_child_pid: git_child_pid_arc,
            });

            let handle = app.handle().clone();
            let path = resolve_app_working_dir(&handle);

            #[cfg(target_os = "macos")]
            if let Err(e) = migrate_macos_data_if_needed(&handle, &path) {
                tracing::error!("macOS 数据迁移失败: {}", e);
            }

            // 如果目录不存在，先创建
            if !path.exists() {
                if let Err(e) = std::fs::create_dir_all(&path) {
                    eprintln!("创建应用数据目录失败: {e}");
                }
            }

            if let Err(e) = std::env::set_current_dir(&path) {
                eprintln!("设置工作目录失败: {e}");
            }

            if let Err(e) = ensure_standard_layout(&path) {
                #[cfg(target_os = "windows")]
                #[cfg(target_os = "windows")]
                {
                    if e.kind() == std::io::ErrorKind::PermissionDenied && !elevation::is_elevated()
                    {
                        tracing::warn!("检测到无法写入应用目录且未提权，尝试自动请求管理员权限...");
                        let _ = elevation::elevate_process(app.handle().clone());
                    }
                }
                return Err(Box::new(e));
            }

            // 初始化日志
            init_logger(&path.join("data"));
            tracing::info!("应用启动");

            // 自动配置内置酒馆和 Node.js
            let handle = app.handle().clone();
            let data_path = path.clone();
            tauri::async_runtime::spawn(async move {
                // 1. 自动配置内置 Node.js
                let node_to = data_path.join("node");
                let node_exe = if cfg!(target_os = "windows") {
                    node_to.join("node.exe")
                } else {
                    node_to.join("bin").join("node")
                };
                if !node_exe.exists() {
                    // 从资源目录复制内置 Node
                    #[cfg(not(dev))]
                    let resource_base = handle.path().resource_dir().unwrap_or_default();
                    #[cfg(dev)]
                    let resource_base = {
                        let mut p = std::env::current_dir().unwrap_or_default();
                        if !p.ends_with("src-tauri") { p.push("src-tauri"); }
                        p.join("resources")
                    };
                    // 查找 node-v* 目录
                    if let Ok(entries) = std::fs::read_dir(&resource_base) {
                        for entry in entries.flatten() {
                            let name = entry.file_name();
                            let name_str = name.to_string_lossy();
                            if name_str.starts_with("node-v") {
                                let _ = std::fs::remove_dir_all(&node_to);
                                // 递归复制
                                fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
                                    std::fs::create_dir_all(dst)?;
                                    for entry in std::fs::read_dir(src)? {
                                        let entry = entry?;
                                        let ty = entry.file_type()?;
                                        let dst_path = dst.join(entry.file_name());
                                        if ty.is_dir() {
                                            copy_dir(&entry.path(), &dst_path)?;
                                        } else {
                                            std::fs::copy(entry.path(), &dst_path)?;
                                        }
                                    }
                                    Ok(())
                                }
                                if copy_dir(&entry.path(), &node_to).is_ok() {
                                    tracing::info!("已复制内置 Node.js 到数据目录");
                                }
                                break;
                            }
                        }
                    }
                }

                // 2. 自动配置内置酒馆
                if let Ok(bundled_path) = sillytavern::get_bundled_tavern_path(handle.clone()) {
                    let mut config = config::read_app_config_from_disk(&handle);
                    let needs_update = config.sillytavern.version.version.is_empty()
                        || !config.initial_setup_completed
                        || config.sillytavern.version.path != bundled_path;
                    if needs_update {
                        config.sillytavern.version = crate::types::LocalTavernItem {
                            version: "1.18.0".to_string(),
                            path: bundled_path,
                            has_node_modules: true,
                        };
                        if !config.initial_setup_completed {
                            config.initial_setup_completed = true;
                        }
                        config.setup_checkpoint = Some("DONE".to_string());
                        let _ = config::write_app_config_to_disk(&handle, &config);
                        tracing::info!("已自动配置内置酒馆 v1.18.0, path: {}", config.sillytavern.version.path);
                    }
                }
            });

            let handle = app.handle().clone();
            apply_saved_window_position(&handle);
            setup_window_position_tracking(&handle);

            // 监听主窗口关闭事件：主窗口关闭时强制关掉 sillytavern-desktop 子窗口
            if let Some(main_win) = app.get_webview_window("main") {
                let handle2 = app.handle().clone();
                main_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::Destroyed = event {
                        if let Some(desktop_win) = handle2.get_webview_window("sillytavern-desktop")
                        {
                            tracing::info!("主窗口已销毁，强制关闭 sillytavern-desktop 子窗口");
                            let _ = desktop_win.close();
                        }
                    }
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // 通用
            config::greet,
            config::get_app_config,
            config::save_app_config,
            config::get_app_version,
            config::open_directory,
            config::fetch_github_proxies,
            config::get_system_cpu_cores,
            config::test_network_proxy,
            config::test_github_connection,
            config::test_github_multi,
            config::test_download_speed,
            config::get_system_proxy_info,
            // Node.js / npm
            node::check_nodejs,
            node::check_nodejs_both,
            node::check_npm,
            node::install_nodejs,
            // Git
            git::check_git,
            git::check_git_both,
            git::install_git,
            git::cancel_git_node_install,
            // SillyTavern 版本管理
            sillytavern::get_bundled_tavern_path,
            sillytavern::fetch_sillytavern_releases,
            sillytavern::get_installed_sillytavern_versions,
            sillytavern::get_installed_versions_info,
            sillytavern::switch_sillytavern_version,
            sillytavern::link_existing_sillytavern,
            sillytavern::install_sillytavern_version,
            sillytavern::install_sillytavern_dependencies,
            sillytavern::check_local_tavern_dependencies,
            sillytavern::cancel_install,
            sillytavern::delete_sillytavern_version,
            sillytavern::check_sillytavern_empty,
            sillytavern::get_tavern_version,
            // SillyTavern 配置
            sillytavern::read_sillytavern_config,
            sillytavern::write_sillytavern_config,
            sillytavern::get_sillytavern_config_path,
            sillytavern::get_sillytavern_config_options,
            sillytavern::update_sillytavern_config_options,
            sillytavern::open_sillytavern_config_file,
            // 全局配置操作（新版本）
            sillytavern::get_sillytavern_global_config_options,
            sillytavern::update_sillytavern_global_config_options,
            sillytavern::open_sillytavern_global_config_file,
            // 配置迁移
            sillytavern::list_config_migration_sources,
            sillytavern::migrate_tavern_config,
            // 资源迁移
            sillytavern::list_resource_migration_sources,
            sillytavern::scan_migration_conflicts,
            sillytavern::execute_resource_migration,
            // SillyTavern 进程
            sillytavern::start_sillytavern,
            sillytavern::stop_sillytavern,
            sillytavern::check_sillytavern_status,
            sillytavern::open_tavern_desktop_window,
            sillytavern::get_local_ip_addresses,
            sillytavern::get_public_ip_addresses,
            sillytavern::check_network_availability,
            sillytavern::repair_missing_deps,
            // 扩展管理
            extensions::get_extensions,
            extensions::toggle_extension_enable,
            extensions::delete_extension,
            extensions::toggle_extension_auto_update,
            extensions::open_extension_folder,
            extensions::install_extension_git,
            extensions::repair_extension_git,
            extensions::open_specific_extension_folder,
            extensions::verify_extension_zip,
            extensions::install_extension_zip,
            extensions::verify_extension_zip_from_bytes,
            extensions::install_extension_zip_from_bytes,
            // 角色卡
            character::list_character_card_pngs,
            character::read_character_card_png,
            character::delete_character_cards,
            character::import_character_card,
            character::read_local_file,
            character::import_character_card_from_bytes,
            // 预设角色卡
            character::list_bundled_cards,
            character::read_bundled_card_thumb,
            character::import_bundled_card,
            // 预设资源
            character::list_bundled_presets,
            character::import_bundled_preset,
            // secrets.json
            secrets::read_secrets,
            secrets::write_secrets,
            secrets::test_api_connection,
            secrets::fetch_model_list,
            // 酒馆 API
            tavern_api::tavern_register,
            tavern_api::tavern_login,
            tavern_api::tavern_send_verification_code,
            tavern_api::tavern_get_self,
            tavern_api::tavern_get_tokens,
            tavern_api::tavern_create_token,
            tavern_api::tavern_delete_token,
            tavern_api::tavern_update_token_status,
            tavern_api::tavern_get_token_by_name,
            tavern_api::tavern_topup,
            tavern_api::tavern_calc_amount,
            tavern_api::tavern_create_payment,
            tavern_api::tavern_get_models,
            tavern_api::tavern_get_token_detail,
            tavern_api::open_tavern_key_webview,
            // 世界书
            worldinfo::list_world_infos,
            worldinfo::read_world_info,
            worldinfo::delete_world_infos,
            worldinfo::import_world_info,
            worldinfo::import_world_info_from_bytes,
            // 预设
            presets::list_presets,
            presets::import_preset_file,
            presets::read_preset_file,
            presets::delete_presets,
            // 正则
            presets::list_regex_scripts,
            presets::import_regex_script,
            presets::delete_regex_scripts,
            // 对话历史
            chat::list_chats,
            chat::read_chat,
            chat::delete_chats,
            // 提权支持
            elevation::is_elevated,
            elevation::elevate_process,
            // 本地酒馆扫描
            finderst::scan_local_sillytavern,
            finderst::cancel_scan_local_sillytavern,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } => {
                let app = app_handle.clone();
                let _ = std::thread::spawn(move || {
                    // 关闭桌面程序模式子窗口（如果存在）
                    if let Some(desktop_win) = app.get_webview_window("sillytavern-desktop") {
                        let _ = desktop_win.close();
                    }

                    // 在进入 async 之前，从 state 里取出 Arc（同步代码，无生命周期问题）
                    let git_child_pid_arc: Arc<Mutex<Option<u32>>> = {
                        let owned = app.state::<OwnedArcs>();
                        owned
                            .cancel_flag
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                        // 将 Arc 内容通过 unsafe transmute_copy 延长生命周期 —— 实际上
                        // 我们只需要让编译器知道这个 Arc 可以独立存活，而 Arc 本身是
                        // 引用计数安全的。用更安全的方式：将内部指针重建为独立 Arc。
                        // 实际上最简单的做法是直接用 Arc<AtomicU32> 重建，但因为类型固定，
                        // 我们直接用 unsafe 重新 clone 一个独立 Arc。
                        // —— 改用更安全方式：把 raw pointer 重新包装
                        let raw = Arc::as_ptr(&owned.git_child_pid);
                        unsafe {
                            Arc::increment_strong_count(raw);
                            Arc::from_raw(raw)
                        }
                    };

                    if let Ok(rt) = tokio::runtime::Runtime::new() {
                        rt.block_on(async move {
                            // 停止本地酒馆扫描
                            let _ = finderst::cancel_scan_local_sillytavern().await;

                            // 获取 ProcessState 并停止酒馆（同时还原 git config）
                            let state = app.state::<ProcessState>();
                            let _ = crate::sillytavern::stop_sillytavern(app.clone(), state).await;

                            // kill 正在运行的 git 子进程（如 git clone / git fetch）
                            if let Some(pid) = git_child_pid_arc.lock().await.take() {
                                tracing::info!("程序退出：正在终止 git 子进程 PID={}", pid);
                                #[cfg(target_os = "windows")]
                                {
                                    let _ = std::process::Command::new("taskkill")
                                        .args(["/F", "/PID", &pid.to_string(), "/T"])
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .status();
                                }
                                #[cfg(not(target_os = "windows"))]
                                {
                                    let _ = std::process::Command::new("kill")
                                        .args(["-9", &pid.to_string()])
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .status();
                                }
                            }
                        });
                    }
                })
                .join();
            }
            _ => {}
        });
}
