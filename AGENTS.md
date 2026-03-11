# AGENTS.md

IRS Tools 项目的 AI Agent 工作指南与架构说明。

本文件用于帮助 AI coding agents（Codex / Claude Code / Cursor / OpenCode 等）理解项目结构、开发规则与运行方式。

Agents 在修改代码或理解系统时  **必须遵循本文件的约束** 。

---

# 1 项目概览

IRS Tools 是一个  **设备测试桌面工具** 。

核心功能：

* 控制设备进入测试模式
* 按测试组执行设备命令
* 实时接收设备状态
* 人工确认测试步骤
* 保存测试结果
* 导出 CSV 报告

系统结构：

```text
Svelte UI
   ↓
Tauri IPC
   ↓
Rust Backend
   ↓
Device Gateway
   ↓
CommDllv2.dll
   ↓
Hardware Device
```

---

# 2 技术栈

### 前端

* Svelte 5
* TypeScript
* Vite
* Tailwind / shadcn-style UI

### 桌面容器

* Tauri v2
* Tauri updater plugin（GitHub Releases 静态 `latest.json`）

### 后端

* Rust

主要 crates：

* tauri
* tokio
* rusqlite
* serde
* tracing

### 设备通信

```text
CommDllv2.dll
```

头文件：

```text
RS232libProduction.h
```

### 数据存储

SQLite

```text
rusqlite
```

### 导出

CSV

---

# 3 项目结构

```text
.
├─ src/                        # 前端源码
├─ src-tauri/                  # Rust + Tauri backend
├─ .github/workflows/          # CI / Release workflows
├─ dist/                       # Vite build output
├─ src-tauri/capabilities/     # Tauri capability permissions
├─ src-tauri/config/
│  ├─ threshold.toml
│  └─ tests.yaml
├─ AGENTS.md
├─ DATABASE_DESIGN.md
├─ TASK_ORCHESTRATION_OPTIMIZATION.md
├─ README.md
├─ package.json
└─ bun.lock
```

---

# 4 前端架构

目录：

```text
src/
```

## 4.1 入口

```text
src/main.ts
```

负责：

* 挂载 `App.svelte`

---

## 4.2 App.svelte

核心职责：

* 全局状态中心
* 测试流程控制
* IPC 事件监听
* 应用更新状态管理

主要事件：

```text
test-group-complete
device-sn-update
key-state-update
front-light-confirm-request
wheel-motor-test-update
```

前灯确认流程：

```text
backend
  ↓
front-light-confirm-request
  ↓
ConfirmDialog
  ↓
confirm_front_light
  ↓
backend resume
```

---

## 4.3 服务层

```text
src/services/tauri.ts
```

统一封装：

```text
invoke
listen
plugin-updater
```

命令：

```text
start_test
stop_test
retest_group
confirm_front_light
confirm_wheel_motor_lifted
get_base_config
save_base_config
get_test_stages
get_tests_config_update_status
applyTestsConfigUpdate
ignoreTestsConfigUpdate
export_test_results_csv
get_available_export_dates
confirm_wheel_motor_lifted
checkForAppUpdate
downloadAndInstallAppUpdate
```

Agent 修改前端逻辑时  **必须优先复用此文件** 。

---

## 4.4 页面

```text
src/views/
```

### MainView.svelte

主测试界面。

### SettingsView.svelte

配置页。

---

## 4.5 组件

```text
src/components/
```

主要组件：

```text
ResultsTable
SettingsForm
StatusCard
KeyTestDialog
ConfirmDialog
ExportDialog
```

---

## 4.6 UI组件库

```text
src/lib/components/ui/
```

基础组件：

```text
button
input
dialog
sidebar
table
select
calendar
```

---

## 4.7 类型系统

```text
src/types/
```

拆分：

```text
config.ts
test.ts
status.ts
i18n.ts
```

统一导出：

```text
src/types.ts
```

---

# 5 后端架构

目录：

```text
src-tauri/src/
```

---

# 5.1 main.rs

Tauri 启动入口。

注册命令：

```text
start_test
stop_test
retest_group
confirm_front_light
show_main_window
get_base_config
save_base_config
get_test_stages
get_tests_config_update_status
apply_tests_config_update
ignore_tests_config_update
export_test_results_csv
get_available_export_dates
```

---

# 5.2 commands.rs

负责：

* `#[tauri::command]`
* 参数解析
* 线程调度

典型模式：

```text
spawn_blocking
```

---

# 5.3 test_service.rs

测试编排层。

职责：

```text
读取配置
创建设备连接
连接成功后读取SN（ParamId526）
推送SN到前端（device-sn-update）
进入测试模式
遍历测试组
保存结果
退出测试模式
```

关键调用：

```text
test_runner::run_group
```

---

# 5.4 test_runner.rs

测试执行层。

负责：

```text
执行单组测试
命令分发
结果构建
```

支持命令：

读取类

```text
068
588
654
272
080
120
122
470
794
796
114
```

动作类

```text
468
606
254
256
```

组合类

```text
CuttingHeightSetAndVerify
WheelMotorTest
```

按键测试

```text
776
```

---

## 特殊流程

### 按键测试

```text
ParamId776
```

流程：

```text
轮询设备
→ 推送 key-state-update
→ UI实时显示
```

---

### 前灯确认

```text
ParamId606
```

流程：

```text
backend execute
  ↓
front-light-confirm-request
  ↓
frontend confirm
  ↓
confirm_front_light
confirm_wheel_motor_lifted
  ↓
backend continue
```

---

### 驱动轮电机测试

```text
WheelMotorTest
```

流程：

```text
backend emit wheel-motor-test-update(lift_confirm)
  ↓
frontend confirm
  ↓
confirm_wheel_motor_lifted
  ↓
backend execute 254(right) + 114采样
  ↓
backend execute 256(left) + 114采样
  ↓
按两条 checks（right_wheel_motor / left_wheel_motor）出结果
```

---

# 6 设备接入层

```text
device_gateway.rs
```

定义抽象：

```text
DeviceGateway
DeviceGatewayFactory
```

---

```text
comm_dll.rs
```

封装：

```text
CommDllv2.dll
```

功能：

```text
动态加载 DLL
调用设备命令
返回状态
```

---

# 7 数据层

```text
db.rs
```

职责：

```text
SQLite 初始化
测试会话保存
测试项结果保存
CSV 导出
可导出日期查询
```

---

# 8 配置

目录：

```text
src-tauri/config/
```

### threshold.toml

基础配置：

```text
serial/network
timeout
log level
```

---

### tests.yaml

测试定义：

```text
test group
stage
commands
threshold
rules
```

运行时相关文件：

```text
AppData\Roaming\com.greenworks.irs-tools\config\tests.yaml
AppData\Roaming\com.greenworks.irs-tools\config\tests.state.json
AppData\Roaming\com.greenworks.irs-tools\config\tests.yaml.new
```

升级规则：

* `tests.yaml` 视为用户可编辑的活动配置。
* 包内 `src-tauri/config/tests.yaml` 仅作为默认配置来源。
* 若本地 `tests.yaml` 仍等于上次应用的默认版本，升级后自动切换到新默认。
* 若本地 `tests.yaml` 已被修改，升级后保留当前文件，并把新默认写入 `tests.yaml.new` 供设置页提示与手动应用。

---

# 9 关键调用链

```text
App.svelte
    ↓
tauri invoke
    ↓
commands.rs
    ↓
test_service.rs
    ↓
test_runner.rs
    ↓
DeviceGateway
    ↓
comm_dll.rs
    ↓
CommDllv2.dll
    ↓
Device
```

---

# 10 开发命令

安装依赖

```text
bun install
```

前端开发

```text
bun run dev
```

前端构建

```text
bun run build
```

Tauri 联调

```text
bun run tauri dev
```

打包

```text
bun run tauri build
```

生成 updater 签名密钥

```text
bunx tauri signer generate --ci --write-keys %USERPROFILE%\.tauri\irs-tools-updater.key
```

---

# 11 Agent 工作规则

AI agents  **必须遵守以下规则** 。

### 1 不要破坏 IPC 协议

事件名称必须保持稳定：

```text
test-group-complete
device-sn-update
key-state-update
front-light-confirm-request
wheel-motor-test-update
```

---

### 2 不要改变设备通信协议

```text
CommDllv2.dll
```

调用方式必须保持兼容。

---

### 3 不要在 UI 组件直接调用 invoke

统一通过：

```text
src/services/tauri.ts
```

包括 updater 检查与安装逻辑。

---

### 4 测试逻辑必须在 backend

不要在前端写测试逻辑。

---

### 5 Rust 必须保持分层

```text
commands
service
runner
gateway
db
```

---

# 12 Agent Self-Documentation（自动完善文档）

Agents  **被允许更新 AGENTS.md** 。

当出现以下情况时：

* 新增模块
* 新增测试流程
* 新增 IPC 事件
* 新增数据库结构
* 新增设备命令

Agent 应当：

1. 更新对应章节
2. 添加新的结构说明
3. 保持文档结构一致

推荐追加规则：

```text
## Agent Notes




Change:
Description of architecture change.
```

---

# 13 Agent Learning Rule

Agents 在理解项目时应：

1. 优先阅读：

```text
AGENTS.md
README.md
DATABASE_DESIGN.md
```

2. 然后阅读：

```text
main.rs
test_service.rs
test_runner.rs
```

3. 最后再阅读 UI。

---

# 14 Agent Safety Rules

Agent  **禁止** ：

* 修改设备 DLL 接口
* 修改 IPC 事件名
* 修改数据库 schema（除非 migration 同步）

---

## Agent Notes

Change:
Added Tauri updater integration backed by GitHub Releases `latest.json`.

Description of architecture change.
- Frontend updater actions are wrapped in `src/services/tauri.ts`.
- Tauri capabilities now include `updater:default`.
- Release workflow now publishes `.msi`, `.msi.sig`, and `latest.json`.

Change:
Read mower SN with `ParamId526` immediately after each successful device connection.

Description of architecture change.
- `test_service.rs` now triggers `ParamId526` right after gateway creation, before the normal test loop.
- The fetched `PcbSerNo` is reused as the session SN for database persistence, even when the selected stages do not include the explicit `526` test group.

Change:
Push connected mower SN to the frontend immediately after it is read.

Description of architecture change.
- Backend emits `device-sn-update` once `ParamId526` returns after connect.
- Frontend subscribes through `src/services/tauri.ts` and updates the main view `machineSn` state without waiting for a `ParamId526` test result row.

Change:
Added managed upgrade flow for user-editable `tests.yaml`.

Description of architecture change.
- Backend now tracks `tests.yaml` lifecycle with `tests.state.json`.
- Unmodified local `tests.yaml` files are auto-updated to the new packaged default during startup.
- Locally edited `tests.yaml` files are preserved and new defaults are staged to `tests.yaml.new`.
- Settings page exposes read-only status plus actions to ignore the reminder or back up and apply the new default.
