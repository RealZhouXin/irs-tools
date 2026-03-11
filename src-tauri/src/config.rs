use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{Manager, path::BaseDirectory};
use tracing::{info, warn};

use crate::models::{
    ApplyTestsConfigUpdateResult, BaseConfig, TestConfig, TestGroup, TestsConfigUpdateStatus,
};
use crate::types::{AppError, CommandResult};

const BASE_CONFIG_RELATIVE_PATH: &str = "config/threshold.toml";
const TESTS_CONFIG_RELATIVE_PATH: &str = "config/tests.yaml";
const TESTS_CONFIG_STATE_RELATIVE_PATH: &str = "config/tests.state.json";

pub fn init_default_configs(app: &tauri::AppHandle) {
    if let Err(err) =
        copy_default_resource_if_missing(app, BASE_CONFIG_RELATIVE_PATH, "默认基础配置文件")
    {
        warn!("Failed to initialize base config: {}", err);
    }

    if let Err(err) = reconcile_tests_config(app) {
        warn!("Failed to initialize tests config: {}", err);
    }
}

#[derive(Deserialize)]
struct TestsConfig {
    #[serde(default)]
    stages: Vec<String>,
    tests: Vec<TestGroup>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TestsConfigState {
    #[serde(default)]
    last_applied_default_hash: Option<String>,
    #[serde(default)]
    last_applied_default_version: Option<String>,
    #[serde(default)]
    pending_default_hash: Option<String>,
    #[serde(default)]
    pending_default_version: Option<String>,
    #[serde(default)]
    ignored_default_hash: Option<String>,
}

#[derive(Debug, Clone)]
struct TestsConfigPaths {
    active_path: PathBuf,
    pending_path: PathBuf,
    state_path: PathBuf,
}

pub fn read_config(app: &tauri::AppHandle) -> CommandResult<TestConfig> {
    let base_config = read_base_config(app)?;
    let tests_config = read_tests_config_file(app)?;
    let stages = resolve_stage_order(&tests_config);
    let tests = tests_config.tests;

    Ok(TestConfig {
        connection: base_config.connection,
        read_timeout_ms: base_config.read_timeout_ms,
        stages,
        tests,
    })
}

pub fn read_base_config(app: &tauri::AppHandle) -> CommandResult<BaseConfig> {
    let base_path = resolve_readable_config_path(app, BASE_CONFIG_RELATIVE_PATH, "配置文件")?;
    info!("Using config at {}", base_path.display());
    let base_data =
        fs::read_to_string(&base_path).map_err(|err| AppError::io("无法读取配置文件", err))?;
    toml::from_str(&base_data).map_err(|err| AppError::toml_de("配置文件解析失败", err))
}

pub fn read_test_stages(app: &tauri::AppHandle) -> CommandResult<Vec<String>> {
    let tests_config = read_tests_config_file(app)?;
    Ok(resolve_stage_order(&tests_config))
}

pub fn read_tests_config_update_status(
    app: &tauri::AppHandle,
) -> CommandResult<TestsConfigUpdateStatus> {
    reconcile_tests_config(app)
}

pub fn apply_tests_config_update(
    app: &tauri::AppHandle,
) -> CommandResult<ApplyTestsConfigUpdateResult> {
    let _ = reconcile_tests_config(app)?;
    let paths = resolve_tests_config_paths(app)?;
    let backup_path = apply_pending_tests_config_update_with_paths(
        &paths,
        &app.package_info().version.to_string(),
    )?;
    let status = reconcile_tests_config(app)?;

    Ok(ApplyTestsConfigUpdateResult {
        backup_path: backup_path.display().to_string(),
        status,
    })
}

pub fn ignore_tests_config_update(
    app: &tauri::AppHandle,
) -> CommandResult<TestsConfigUpdateStatus> {
    let status = reconcile_tests_config(app)?;
    if !status.new_default_available {
        return Ok(status);
    }

    let paths = resolve_tests_config_paths(app)?;
    let mut state = read_tests_config_state(&paths.state_path);
    state.ignored_default_hash = state.pending_default_hash.clone();
    write_tests_config_state(&paths.state_path, &state)?;

    reconcile_tests_config(app)
}

pub fn write_base_config(app: &tauri::AppHandle, config: &BaseConfig) -> CommandResult<()> {
    let path = resolve_writable_config_path(app, BASE_CONFIG_RELATIVE_PATH, "配置文件")?;
    let data = toml::to_string_pretty(config)
        .map_err(|err| AppError::toml_ser("配置文件序列化失败", err))?;
    ensure_parent_dir(&path)?;
    fs::write(&path, data).map_err(|err| AppError::io("无法写入配置文件", err))?;
    info!("Saved config at {}", path.display());
    Ok(())
}

fn reconcile_tests_config(app: &tauri::AppHandle) -> CommandResult<TestsConfigUpdateStatus> {
    let paths = resolve_tests_config_paths(app)?;
    let default_content =
        read_default_resource_text(app, TESTS_CONFIG_RELATIVE_PATH, "默认测试项配置文件")?;
    reconcile_tests_config_with_content(
        &paths,
        &default_content,
        &app.package_info().version.to_string(),
    )
}

fn reconcile_tests_config_with_content(
    paths: &TestsConfigPaths,
    default_content: &str,
    app_version: &str,
) -> CommandResult<TestsConfigUpdateStatus> {
    ensure_parent_dir(&paths.active_path)?;
    ensure_parent_dir(&paths.state_path)?;

    let default_hash = hash_bytes(default_content.as_bytes());
    let mut state = read_tests_config_state(&paths.state_path);

    if !paths.active_path.exists() {
        write_text_file(&paths.active_path, default_content)?;
        remove_file_if_exists(&paths.pending_path)?;
        update_last_applied_default(&mut state, &default_hash, app_version);
        clear_pending_default(&mut state);
        write_tests_config_state(&paths.state_path, &state)?;
        return Ok(build_tests_config_update_status(paths, false, &state));
    }

    let active_content = fs::read_to_string(&paths.active_path)
        .map_err(|err| AppError::io("无法读取测试项配置文件", err))?;
    let active_hash = hash_bytes(active_content.as_bytes());

    if active_hash == default_hash {
        remove_file_if_exists(&paths.pending_path)?;
        update_last_applied_default(&mut state, &default_hash, app_version);
        clear_pending_default(&mut state);
        write_tests_config_state(&paths.state_path, &state)?;
        return Ok(build_tests_config_update_status(paths, false, &state));
    }

    let active_matches_last_applied =
        state.last_applied_default_hash.as_deref() == Some(active_hash.as_str());
    if active_matches_last_applied {
        write_text_file(&paths.active_path, default_content)?;
        remove_file_if_exists(&paths.pending_path)?;
        update_last_applied_default(&mut state, &default_hash, app_version);
        clear_pending_default(&mut state);
        write_tests_config_state(&paths.state_path, &state)?;
        return Ok(build_tests_config_update_status(paths, false, &state));
    }

    if state.pending_default_hash.as_deref() != Some(default_hash.as_str())
        || !paths.pending_path.exists()
    {
        write_text_file(&paths.pending_path, default_content)?;
    }
    state.pending_default_hash = Some(default_hash);
    state.pending_default_version = Some(app_version.to_string());
    if state.ignored_default_hash != state.pending_default_hash {
        state.ignored_default_hash = None;
    }
    write_tests_config_state(&paths.state_path, &state)?;

    Ok(build_tests_config_update_status(paths, true, &state))
}

fn apply_pending_tests_config_update_with_paths(
    paths: &TestsConfigPaths,
    app_version: &str,
) -> CommandResult<PathBuf> {
    let mut state = read_tests_config_state(&paths.state_path);
    let pending_hash = state
        .pending_default_hash
        .clone()
        .ok_or_else(|| AppError::msg("当前没有可应用的新默认测试配置"))?;

    if !paths.pending_path.exists() {
        return Err(AppError::msg("未找到待应用的新默认测试配置文件"));
    }
    if !paths.active_path.exists() {
        return Err(AppError::msg("当前测试配置文件不存在，无法创建备份"));
    }

    let pending_content = fs::read_to_string(&paths.pending_path)
        .map_err(|err| AppError::io("无法读取待应用测试配置文件", err))?;
    validate_tests_config_yaml(&pending_content)?;

    let backup_path = build_backup_path(&paths.active_path)?;
    fs::copy(&paths.active_path, &backup_path)
        .map_err(|err| AppError::io("无法备份当前测试配置文件", err))?;
    write_text_file(&paths.active_path, &pending_content)?;
    remove_file_if_exists(&paths.pending_path)?;

    let pending_version = state
        .pending_default_version
        .clone()
        .unwrap_or_else(|| app_version.to_string());
    update_last_applied_default(&mut state, &pending_hash, &pending_version);
    clear_pending_default(&mut state);
    write_tests_config_state(&paths.state_path, &state)?;

    Ok(backup_path)
}

fn validate_tests_config_yaml(content: &str) -> CommandResult<()> {
    serde_yaml::from_str::<TestsConfig>(content)
        .map(|_| ())
        .map_err(|err| AppError::yaml_de("待应用测试项配置解析失败", err))
}

fn update_last_applied_default(state: &mut TestsConfigState, hash: &str, version: &str) {
    state.last_applied_default_hash = Some(hash.to_string());
    state.last_applied_default_version = Some(version.to_string());
}

fn clear_pending_default(state: &mut TestsConfigState) {
    state.pending_default_hash = None;
    state.pending_default_version = None;
    state.ignored_default_hash = None;
}

fn build_tests_config_update_status(
    paths: &TestsConfigPaths,
    local_is_modified: bool,
    state: &TestsConfigState,
) -> TestsConfigUpdateStatus {
    let has_pending = state.pending_default_hash.is_some() && paths.pending_path.exists();
    let ignored_pending_default = has_pending
        && state.pending_default_hash.is_some()
        && state.pending_default_hash == state.ignored_default_hash;

    TestsConfigUpdateStatus {
        active_path: paths.active_path.display().to_string(),
        new_default_available: has_pending,
        local_is_modified,
        pending_default_path: has_pending.then(|| paths.pending_path.display().to_string()),
        pending_default_version: has_pending
            .then(|| state.pending_default_version.clone())
            .flatten(),
        ignored_pending_default,
    }
}

fn read_tests_config_file(app: &tauri::AppHandle) -> CommandResult<TestsConfig> {
    let tests_path =
        resolve_readable_config_path(app, TESTS_CONFIG_RELATIVE_PATH, "测试项配置文件")?;
    info!("Using tests at {}", tests_path.display());
    let tests_data = fs::read_to_string(&tests_path)
        .map_err(|err| AppError::io("无法读取测试项配置文件", err))?;
    serde_yaml::from_str(&tests_data).map_err(|err| AppError::yaml_de("测试项配置解析失败", err))
}

fn resolve_stage_order(tests_config: &TestsConfig) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut resolved = Vec::<String>::new();

    for stage in &tests_config.stages {
        if let Some(normalized) = normalize_stage(stage) {
            if seen.insert(normalized.clone()) {
                resolved.push(normalized);
            }
        }
    }

    for test in &tests_config.tests {
        if let Some(normalized) = normalize_stage(&test.stage) {
            if seen.insert(normalized.clone()) {
                resolved.push(normalized);
            }
        }
    }

    resolved
}

fn normalize_stage(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn copy_default_resource_if_missing(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<()> {
    let dest = resolve_writable_config_path(app, relative, label)?;
    if dest.exists() {
        return Ok(());
    }

    ensure_parent_dir(&dest)?;
    let src = resolve_default_resource_path(app, relative)?;
    fs::copy(&src, &dest).map_err(|err| AppError::io("无法复制默认配置文件", err))?;
    info!("Initialized default config: {}", dest.display());
    Ok(())
}

fn read_default_resource_text(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<String> {
    let src = resolve_default_resource_path(app, relative)?;
    fs::read_to_string(&src).map_err(|err| AppError::msg(format!("{label}: {err}")))
}

fn resolve_default_resource_path(app: &tauri::AppHandle, relative: &str) -> CommandResult<PathBuf> {
    match app.path().resolve(relative, BaseDirectory::Resource) {
        Ok(src) if src.exists() => Ok(src),
        _ => {
            let current_dir =
                std::env::current_dir().map_err(|err| AppError::io("无法获取当前目录", err))?;
            let dev_src = current_dir.join("src-tauri").join(relative);
            if dev_src.exists() {
                Ok(dev_src)
            } else {
                Err(AppError::msg(format!("默认配置资源不存在: {relative}")))
            }
        }
    }
}

fn resolve_tests_config_paths(app: &tauri::AppHandle) -> CommandResult<TestsConfigPaths> {
    let app_config_dir = resolve_app_config_dir(app, "测试配置目录")?;
    let active_path = app_config_dir.join(TESTS_CONFIG_RELATIVE_PATH);
    Ok(TestsConfigPaths {
        pending_path: active_path.with_extension("yaml.new"),
        active_path,
        state_path: app_config_dir.join(TESTS_CONFIG_STATE_RELATIVE_PATH),
    })
}

fn resolve_readable_config_path(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<PathBuf> {
    let config_path = resolve_writable_config_path(app, relative, label)?;
    if config_path.exists() {
        return Ok(config_path);
    }

    warn!(
        "{} not found at expected path: {}",
        label,
        config_path.display()
    );
    Err(AppError::msg(format!("无法找到{label}路径")))
}

fn resolve_writable_config_path(
    app: &tauri::AppHandle,
    relative: &str,
    label: &str,
) -> CommandResult<PathBuf> {
    Ok(resolve_app_config_dir(app, label)?.join(relative))
}

fn resolve_app_config_dir(app: &tauri::AppHandle, label: &str) -> CommandResult<PathBuf> {
    app.path()
        .app_config_dir()
        .map_err(|_| AppError::msg(format!("无法找到{label}路径")))
}

fn ensure_parent_dir(path: &Path) -> CommandResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io("无法创建配置目录", err))?;
    }
    Ok(())
}

fn write_text_file(path: &Path, content: &str) -> CommandResult<()> {
    ensure_parent_dir(path)?;
    fs::write(path, content).map_err(|err| AppError::io("无法写入配置文件", err))
}

fn remove_file_if_exists(path: &Path) -> CommandResult<()> {
    if path.exists() {
        fs::remove_file(path).map_err(|err| AppError::io("无法删除配置文件", err))?;
    }
    Ok(())
}

fn read_tests_config_state(path: &Path) -> TestsConfigState {
    if !path.exists() {
        return TestsConfigState::default();
    }

    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(err) => {
            warn!(
                "Failed to read tests config state {}: {}",
                path.display(),
                err
            );
            return TestsConfigState::default();
        }
    };

    match serde_json::from_str::<TestsConfigState>(&data) {
        Ok(state) => state,
        Err(err) => {
            warn!(
                "Failed to parse tests config state {}: {}",
                path.display(),
                err
            );
            TestsConfigState::default()
        }
    }
}

fn write_tests_config_state(path: &Path, state: &TestsConfigState) -> CommandResult<()> {
    ensure_parent_dir(path)?;
    let data = serde_json::to_string_pretty(state)
        .map_err(|err| AppError::msg(format!("测试配置状态序列化失败: {err}")))?;
    fs::write(path, data).map_err(|err| AppError::io("无法写入测试配置状态文件", err))
}

fn build_backup_path(active_path: &Path) -> CommandResult<PathBuf> {
    let parent = active_path
        .parent()
        .ok_or_else(|| AppError::msg("无法定位测试配置目录"))?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");

    for suffix in 0..100 {
        let file_name = if suffix == 0 {
            format!("tests.backup.{timestamp}.yaml")
        } else {
            format!("tests.backup.{timestamp}-{suffix}.yaml")
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(parent.join(format!("tests.backup.{timestamp}-{nanos}.yaml")))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tests_config(name: &str) -> String {
        format!(
            "stages:\n  - info\ntests:\n  - name: \"{name} 应用板硬件信息\"\n    names:\n      zh: \"{name} 应用板硬件信息\"\n      en: \"{name} Application Board Hardware Info\"\n    command: param_id526\n    stage: info\n    checks:\n      - name: pcb_rev\n        output: pcb_rev\n        min: 1\n        max: 9\n"
        )
    }

    fn temp_paths() -> TestsConfigPaths {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("irs-tools-config-test-{stamp}"));
        let active_path = base.join("config/tests.yaml");
        TestsConfigPaths {
            pending_path: active_path.with_extension("yaml.new"),
            active_path,
            state_path: base.join("config/tests.state.json"),
        }
    }

    #[test]
    fn first_install_copies_default_config() {
        let paths = temp_paths();
        let default_content = sample_tests_config("v1");

        let status =
            reconcile_tests_config_with_content(&paths, &default_content, "0.5.1").unwrap();

        assert!(!status.new_default_available);
        assert!(!status.local_is_modified);
        assert_eq!(
            fs::read_to_string(&paths.active_path).unwrap(),
            default_content
        );
        assert!(!paths.pending_path.exists());

        fs::remove_dir_all(paths.active_path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn upgrade_replaces_unmodified_active_config() {
        let paths = temp_paths();
        let v1 = sample_tests_config("v1");
        let v2 = sample_tests_config("v2");

        reconcile_tests_config_with_content(&paths, &v1, "0.5.0").unwrap();
        let status = reconcile_tests_config_with_content(&paths, &v2, "0.5.1").unwrap();

        assert!(!status.new_default_available);
        assert!(!status.local_is_modified);
        assert_eq!(fs::read_to_string(&paths.active_path).unwrap(), v2);
        assert!(!paths.pending_path.exists());

        fs::remove_dir_all(paths.active_path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn upgrade_preserves_modified_active_config_and_stages_candidate() {
        let paths = temp_paths();
        let v1 = sample_tests_config("v1");
        let v2 = sample_tests_config("v2");
        let local = sample_tests_config("local");

        reconcile_tests_config_with_content(&paths, &v1, "0.5.0").unwrap();
        write_text_file(&paths.active_path, &local).unwrap();

        let status = reconcile_tests_config_with_content(&paths, &v2, "0.5.1").unwrap();

        assert!(status.new_default_available);
        assert!(status.local_is_modified);
        assert_eq!(fs::read_to_string(&paths.active_path).unwrap(), local);
        assert_eq!(fs::read_to_string(&paths.pending_path).unwrap(), v2);

        fs::remove_dir_all(paths.active_path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn existing_install_without_state_is_treated_as_local_config() {
        let paths = temp_paths();
        let v1 = sample_tests_config("v1");
        let v2 = sample_tests_config("v2");

        write_text_file(&paths.active_path, &v1).unwrap();
        let status = reconcile_tests_config_with_content(&paths, &v2, "0.5.1").unwrap();

        assert!(status.new_default_available);
        assert!(status.local_is_modified);
        assert_eq!(fs::read_to_string(&paths.active_path).unwrap(), v1);
        assert_eq!(fs::read_to_string(&paths.pending_path).unwrap(), v2);

        fs::remove_dir_all(paths.active_path.parent().unwrap().parent().unwrap()).unwrap();
    }

    #[test]
    fn apply_pending_update_backs_up_and_promotes_candidate() {
        let paths = temp_paths();
        let v1 = sample_tests_config("v1");
        let v2 = sample_tests_config("v2");
        let local = sample_tests_config("local");

        reconcile_tests_config_with_content(&paths, &v1, "0.5.0").unwrap();
        write_text_file(&paths.active_path, &local).unwrap();
        reconcile_tests_config_with_content(&paths, &v2, "0.5.1").unwrap();

        let backup_path = apply_pending_tests_config_update_with_paths(&paths, "0.5.1").unwrap();
        let status = reconcile_tests_config_with_content(&paths, &v2, "0.5.1").unwrap();

        assert!(!status.new_default_available);
        assert!(!status.local_is_modified);
        assert_eq!(fs::read_to_string(&paths.active_path).unwrap(), v2);
        assert_eq!(fs::read_to_string(&backup_path).unwrap(), local);
        assert!(!paths.pending_path.exists());

        fs::remove_dir_all(paths.active_path.parent().unwrap().parent().unwrap()).unwrap();
    }
}
