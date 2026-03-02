# 测试结果数据库与导出设计方案（基于 SQLite 与方案 B）

## 1. 架构目标

为支持产线级别的测试追溯和数据分析（如计算良品率、查看单项指标的散点图和 CPK），将原有纯运行时的测试结果持久化。
本项目采用 **Tauri + Svelte + SQLite** 架构，并选择**方案 B（微观存储：一条记录对应一个具体的 Check）**来存储测试数据，并提供按时间筛选导出为平展 CSV 的功能。

## 2. 技术选型

- **数据库**：`rusqlite` (启用 `bundled` 特性，免除客户端环境依赖)。在设备数据本地持久化场景性能强劲、即插即用。
- **导出格式**：使用 `csv` 库生成带中文字段的标准逗号分隔文件。
- **存储路径**：默认放置于 `%LOCALAPPDATA%\irs-tools\database\test_history.db`。

## 3. 数据库结构设计

采用主从表设计。一个测试批次触发多条执行指令，每个指令展开为多个具体的检查项 (Check)。

### 3.1 主表：`test_sessions` (测试批次)
记录一次用户点击“开始测试”产生的全局结果。

| 字段名 | 数据类型 | 说明 |
| --- | --- | --- |
| `id` | `INTEGER` | 主键 (自增) |
| `stage` | `TEXT` | 触发的大场景 (例如: 总装测试, 老化测试) |
| `status` | `TEXT` | 整体执行结果 (`Pass`, `Fail`, `Error`) |
| `duration_ms` | `INTEGER` | 整体耗时 |
| `start_time` | `DATETIME` | 测试开始时间，使用本地时间 |

### 3.2 从表：`test_checks` (微观检查项记录)
**方案 B 核心**：表记录将细化到 `Check` 级别。如果一次 `param_id120` 返回了 11 个角度和加速度的检查项，此表中将写入 11 行。

| 字段名 | 数据类型 | 说明 |
| --- | --- | --- |
| `id` | `INTEGER` | 主键 (自增) |
| `session_id` | `INTEGER` | 外键，关联 `test_sessions(id)` |
| `group_name` | `TEXT` | 测试组的逻辑名称 (例如 "Pitch & Roll checks") |
| `command` | `TEXT` | 执行的通信命令 (例如 "param_id120") |
| `check_name` | `TEXT` | **重点**：具体考核的单项指标 (例如 "AccelX") |
| `val_real` | `REAL` | **重点**：解析出的实际检测值，可为 `NULL` |
| `min_limit` | `REAL` | 配置中的最小阈值，可为 `NULL` |
| `max_limit` | `REAL` | 配置中的最大阈值，可为 `NULL` |
| `passed` | `INTEGER` | 是否通过 (0/1，即 `false`/`true`) |
| `raw_response`| `TEXT` | 底层返回的完整响应字符串数据，供极少数异常排查使用 |

## 4. 后端写入逻辑设计 (Rust)

在 `test_service.rs` 或事件处理的最顶层：
1. **测试开始**前，向 `test_sessions` 插入一条具有初始状态的父记录，并获取到 `session_id` (`last_insert_rowid()`)。
2. 内部每执行完成一个 `TestResult` (其对应的结构里存有 `Vec<CheckResult>`) 后，**遍历 `checks` 数组**：
3. 对每个具体的 `CheckResult` 构造一条 `INSERT` 语句写入 `test_checks` 表，并将指令本身的信息（`group_name`, `command`, `raw_response`）作为冗余补充进此行。
4. 测试完全结束时，计算最终的 `duration` 和全局 `status`，更新 `test_sessions` 表。

## 5. CSV 导出与呈现

借助联表查询，将所有子项绑定母级的元数据展平，并直接流式写入 CSV 文件。这使得表格每一行都有精确无遗的情况维度（时间、组、条件、结果）。

**预期的 SQL 查询语句：**
```sql
SELECT 
    s.start_time, s.stage, c.group_name, c.command, c.check_name, 
    c.val_real, c.min_limit, c.max_limit, c.passed, c.raw_response
FROM test_sessions s
JOIN test_checks c ON s.id = c.session_id
WHERE s.start_time BETWEEN ? AND ?
ORDER BY s.id ASC, c.id ASC
```

**对应导出的 CSV 表头与行示例：**
| 测试时间 | 测试场景 | 测试项组 | 执行指令 | **检查项 (Check)** | **实测值** | 下限 | 上限 | 是否通过 | 原始报文(Raw) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 26-03-02 10:15 | 出厂快测 | 陀螺仪检测 | param_id120 | PitchAngle | **1.2** | -10 | 10 | TRUE | PitchAngle=... |
| 26-03-02 10:15 | 出厂快测 | 陀螺仪检测 | param_id120 | AccelX | **2.5** | -5 | 5 | TRUE | PitchAngle=... |
| 26-03-02 10:16 | 功率测试 | 电池检测 | param_id272 | BattTemp | **85.0** | -10 | 80 | FALSE | BattPackPN=... |

### 设计优点
通过将 `Check` 展平成列，生产经理可以使用 Excel 强大的数据透视功能，单独拉取比如 `check_name = 'BattTemp'` 的数据创建线型趋势图去评估产线质量，彻底摆脱了复杂的 JSON 解析。

## 6. 前端 UI 设计

1. **界面集成**：在 `MainView.svelte` 增加一个“数据导出”按钮。
2. **零依赖弹窗**：通过 Svelte 实现一个小型弹窗记录起止日期，调用 `HTML5 <input type="date">`，避免引入庞大的日期选择库。
3. **系统级存储**：利用 Tauri `plugin-dialog` 唤出系统的文件保存框，由 Rust 后端接管大量数据的 I/O 写入以确保界面丝滑不卡顿。
