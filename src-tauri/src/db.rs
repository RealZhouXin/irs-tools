use std::path::PathBuf;

use chrono::{DateTime, Local, NaiveDate};
use csv::Writer;
use rusqlite::{Connection, params};
use tauri::Manager;

use crate::models::TestResult;
use crate::types::{AppError, CommandResult};

const DATABASE_RELATIVE_PATH: &str = "database/test_history.db";

pub fn init_database(app: &tauri::AppHandle) -> CommandResult<()> {
    let connection = open_connection(app)?;
    init_schema(&connection)?;
    Ok(())
}

pub fn persist_test_results(
    app: &tauri::AppHandle,
    stage: &str,
    status: &str,
    start_time: DateTime<Local>,
    duration_ms: i64,
    results: &[TestResult],
) -> CommandResult<()> {
    let mut connection = open_connection(app)?;
    init_schema(&connection)?;

    let tx = connection
        .transaction()
        .map_err(|err| AppError::msg(format!("开启数据库事务失败: {err}")))?;

    tx.execute(
        "INSERT INTO test_sessions (stage, status, duration_ms, start_time) VALUES (?1, ?2, ?3, ?4)",
        params![
            stage,
            status,
            duration_ms,
            start_time.format("%Y-%m-%d %H:%M:%S").to_string()
        ],
    )
    .map_err(|err| AppError::msg(format!("写入 test_sessions 失败: {err}")))?;

    let session_id = tx.last_insert_rowid();

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO test_checks \
                 (session_id, group_name, command, check_name, val_real, min_limit, max_limit, passed, raw_response) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .map_err(|err| AppError::msg(format!("准备 test_checks 写入语句失败: {err}")))?;

        for result in results {
            for check in &result.checks {
                stmt.execute(params![
                    session_id,
                    result.name,
                    result.command,
                    check.name,
                    check.value,
                    check.min,
                    check.max,
                    if check.passed { 1 } else { 0 },
                    result.raw_response,
                ])
                .map_err(|err| AppError::msg(format!("写入 test_checks 失败: {err}")))?;
            }
        }
    }

    tx.commit()
        .map_err(|err| AppError::msg(format!("提交数据库事务失败: {err}")))?;

    Ok(())
}

pub fn export_test_results_csv(
    app: &tauri::AppHandle,
    start_date: &str,
    end_date: &str,
    output_path: &str,
) -> CommandResult<usize> {
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|err| AppError::msg(format!("开始日期格式无效: {err}")))?;
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|err| AppError::msg(format!("结束日期格式无效: {err}")))?;

    if start > end {
        return Err(AppError::msg("开始日期不能晚于结束日期"));
    }

    let connection = open_connection(app)?;
    init_schema(&connection)?;

    let start_ts = format!("{} 00:00:00", start.format("%Y-%m-%d"));
    let end_ts = format!("{} 23:59:59", end.format("%Y-%m-%d"));

    let mut stmt = connection
        .prepare(
            "SELECT \
             s.start_time, s.stage, c.group_name, c.command, c.check_name, \
             c.val_real, c.min_limit, c.max_limit, c.passed, c.raw_response \
             FROM test_sessions s \
             JOIN test_checks c ON s.id = c.session_id \
             WHERE s.start_time BETWEEN ?1 AND ?2 \
             ORDER BY s.id ASC, c.id ASC",
        )
        .map_err(|err| AppError::msg(format!("准备导出查询失败: {err}")))?;

    let output = PathBuf::from(output_path);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|err| AppError::io("创建导出目录失败", err))?;
    }

    let mut writer = Writer::from_path(&output)
        .map_err(|err| AppError::msg(format!("创建 CSV 文件失败: {err}")))?;
    writer
        .write_record([
            "测试时间",
            "测试场景",
            "测试项组",
            "执行指令",
            "检查项(Check)",
            "实测值",
            "下限",
            "上限",
            "是否通过",
            "原始报文(Raw)",
        ])
        .map_err(|err| AppError::msg(format!("写入 CSV 表头失败: {err}")))?;

    let mut rows = stmt
        .query(params![start_ts, end_ts])
        .map_err(|err| AppError::msg(format!("执行导出查询失败: {err}")))?;

    let mut count = 0usize;
    while let Some(row) = rows
        .next()
        .map_err(|err| AppError::msg(format!("读取导出结果失败: {err}")))?
    {
        let start_time: String = row
            .get(0)
            .map_err(|err| AppError::msg(format!("读取测试时间失败: {err}")))?;
        let stage: String = row
            .get(1)
            .map_err(|err| AppError::msg(format!("读取测试场景失败: {err}")))?;
        let group_name: String = row
            .get(2)
            .map_err(|err| AppError::msg(format!("读取测试组失败: {err}")))?;
        let command: String = row
            .get(3)
            .map_err(|err| AppError::msg(format!("读取命令失败: {err}")))?;
        let check_name: String = row
            .get(4)
            .map_err(|err| AppError::msg(format!("读取检查项失败: {err}")))?;
        let val_real: Option<f64> = row
            .get(5)
            .map_err(|err| AppError::msg(format!("读取实测值失败: {err}")))?;
        let min_limit: Option<f64> = row
            .get(6)
            .map_err(|err| AppError::msg(format!("读取下限失败: {err}")))?;
        let max_limit: Option<f64> = row
            .get(7)
            .map_err(|err| AppError::msg(format!("读取上限失败: {err}")))?;
        let passed: i64 = row
            .get(8)
            .map_err(|err| AppError::msg(format!("读取通过状态失败: {err}")))?;
        let raw_response: String = row
            .get(9)
            .map_err(|err| AppError::msg(format!("读取原始报文失败: {err}")))?;

        writer
            .write_record([
                start_time,
                stage,
                group_name,
                command,
                check_name,
                format_optional_float(val_real),
                format_optional_float(min_limit),
                format_optional_float(max_limit),
                if passed == 1 {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                },
                raw_response,
            ])
            .map_err(|err| AppError::msg(format!("写入 CSV 数据失败: {err}")))?;

        count += 1;
    }

    writer
        .flush()
        .map_err(|err| AppError::msg(format!("刷新 CSV 文件失败: {err}")))?;

    Ok(count)
}

pub fn get_available_export_dates(app: &tauri::AppHandle) -> CommandResult<Vec<String>> {
    let connection = open_connection(app)?;
    init_schema(&connection)?;

    let mut stmt = connection
        .prepare(
            "SELECT DISTINCT date(start_time) AS d
             FROM test_sessions
             WHERE date(start_time) IS NOT NULL
             ORDER BY d ASC",
        )
        .map_err(|err| AppError::msg(format!("准备可导出日期查询失败: {err}")))?;

    let mut rows = stmt
        .query([])
        .map_err(|err| AppError::msg(format!("执行可导出日期查询失败: {err}")))?;

    let mut dates = Vec::<String>::new();
    while let Some(row) = rows
        .next()
        .map_err(|err| AppError::msg(format!("读取可导出日期失败: {err}")))?
    {
        let value: Option<String> = row
            .get(0)
            .map_err(|err| AppError::msg(format!("读取日期列失败: {err}")))?;
        let Some(date_str) = value else {
            continue;
        };
        if NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").is_ok() {
            dates.push(date_str);
        }
    }

    Ok(dates)
}

fn resolve_database_path(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(app_local_data_dir) = app.path().app_local_data_dir() {
        return app_local_data_dir.join(DATABASE_RELATIVE_PATH);
    }

    if let Ok(current_dir) = std::env::current_dir() {
        return current_dir.join(DATABASE_RELATIVE_PATH);
    }

    PathBuf::from(DATABASE_RELATIVE_PATH)
}

fn open_connection(app: &tauri::AppHandle) -> CommandResult<Connection> {
    let db_path = resolve_database_path(app);

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| AppError::io("创建数据库目录失败", err))?;
    }

    Connection::open(db_path).map_err(|err| AppError::msg(format!("打开数据库失败: {err}")))
}

fn init_schema(connection: &Connection) -> CommandResult<()> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS test_sessions (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 stage TEXT NOT NULL,
                 status TEXT NOT NULL,
                 duration_ms INTEGER NOT NULL,
                 start_time TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS test_checks (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 session_id INTEGER NOT NULL,
                 group_name TEXT NOT NULL,
                 command TEXT NOT NULL,
                 check_name TEXT NOT NULL,
                 val_real REAL NULL,
                 min_limit REAL NULL,
                 max_limit REAL NULL,
                 passed INTEGER NOT NULL,
                 raw_response TEXT NOT NULL,
                 FOREIGN KEY(session_id) REFERENCES test_sessions(id) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS idx_test_sessions_start_time ON test_sessions(start_time);
             CREATE INDEX IF NOT EXISTS idx_test_checks_session_id ON test_checks(session_id);",
        )
        .map_err(|err| AppError::msg(format!("初始化数据库结构失败: {err}")))?;

    Ok(())
}

fn format_optional_float(value: Option<f64>) -> String {
    match value {
        Some(v) => {
            if v.fract() == 0.0 {
                format!("{v:.0}")
            } else {
                v.to_string()
            }
        }
        None => String::new(),
    }
}
