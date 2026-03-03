# IRS Tools 项目结构（当前版本）

## 1. 技术栈

- 前端：Svelte 5 + TypeScript + Vite
- 桌面容器：Tauri v2
- 后端：Rust
- 设备通信：`CommDllv2.dll`（头文件：`RS232libProduction.h`）
- 数据存储：SQLite（`rusqlite`）
- 导出：CSV

---

## 2. 根目录结构（关键文件）

```text
.
├─ src/                        # 前端源码
├─ src-tauri/                  # Rust + Tauri 后端
├─ dist/                       # 前端构建产物（Vite）
├─ src-tauri/config/
│  ├─ threshold.toml           # 基础连接/超时配置
│  └─ tests.toml               # 测试组定义
├─ PROJECT_STRUCTURE.md        # 本文档
├─ DATABASE_DESIGN.md
├─ TASK_ORCHESTRATION_OPTIMIZATION.md
├─ README.md
├─ package.json
└─ bun.lock
```

---

## 3. 前端结构（`src/`）

### 3.1 入口与顶层状态

- `src/main.ts`
  - 挂载 `App.svelte`。
- `src/App.svelte`
  - 全局状态中心与交互编排。
  - 负责启动测试、停止测试、单项重测、结果更新、导出、语言切换。
  - 事件监听：
    - `test-group-complete`
    - `key-state-update`
    - `front-light-confirm-request`
  - 前灯确认流程：收到 `front-light-confirm-request` 弹窗 -> 调用 `confirm_front_light` 回传。

### 3.2 服务层

- `src/services/tauri.ts`
  - 封装 `invoke` 与 `listen`，统一前后端桥接。
  - 命令调用：
    - `start_test`
    - `stop_test`
    - `retest_group`
    - `confirm_front_light`
    - `get_base_config` / `save_base_config`
    - `get_test_stages`
    - `export_test_results_csv`
    - `get_available_export_dates`

### 3.3 视图与业务组件

- `src/views/MainView.svelte`
  - 主测试页面。
- `src/views/SettingsView.svelte`
  - 配置页与 About 区域。
- `src/components/`
  - 业务组件：`ResultsTable`、`SettingsForm`、`StatusCard` 等。
  - 弹窗组件：
    - `KeyTestDialog.svelte`
    - `ConfirmDialog.svelte`（前灯人工确认）
    - `ExportDialog.svelte`

### 3.4 类型

- `src/types/`：按领域拆分类型（`config.ts`、`test.ts`、`status.ts`、`i18n.ts`、`index.ts`）。
- `src/types.ts`：兼容性导出入口（重新导出 `src/types/*`）。

### 3.5 UI 基础组件

- `src/lib/components/ui/`
  - 通用 UI 组件集合（button/input/dialog/sidebar/table/select/calendar 等）。

---

## 4. 后端结构（`src-tauri/src/`）

### 4.1 入口与命令注册

- `main.rs`
  - Tauri 启动入口。
  - 注册命令：
    - `start_test`
    - `stop_test`
    - `retest_group`
    - `confirm_front_light`
    - `show_main_window`
    - `get_base_config`
    - `save_base_config`
    - `get_test_stages`
    - `export_test_results_csv`
    - `get_available_export_dates`

### 4.2 命令层

- `commands.rs`
  - `#[tauri::command]` 实现层。
  - 主要负责参数接收、线程调度（`spawn_blocking`）、调用服务层/数据库层。

### 4.3 编排层

- `test_service.rs`
  - 整体测试流程编排：
    - 读取配置
    - 创建设备网关
    - 进入/退出测试模式（`ParamId374`）
    - 遍历测试组并统一调用 `test_runner::run_group`
    - 逐组发 `test-group-complete`
    - 处理 stop、错误、结果持久化

### 4.4 执行层

- `test_runner.rs`
  - 单个测试组执行与结果构建。
  - 包含命令执行分发：
    - 读取类（068/588/654/272/080/120/122/470/794）
    - 动作类（468/606）
    - 组合类（`CuttingHeightSetAndVerify`）
    - 按键测试（776）
  - 特殊流程：
    - `ParamId776`：轮询并推送 `key-state-update`。
    - `ParamId606`：执行后推送 `front-light-confirm-request`，阻塞等待前端 `confirm_front_light`，再返回最终结果。
  - 内含单元测试（fake gateway）。

### 4.5 配置与模型

- `config.rs`
  - 读取/写入基础配置与测试配置（TOML）。
- `models.rs`
  - 配置、命令、结果、事件载荷等数据结构。
  - 包含 `FrontLightConfirmRequestPayload`、`KeyStatePayload` 等。
- `events.rs`
  - 事件常量：
    - `test-group-complete`
    - `key-state-update`
    - `front-light-confirm-request`
- `types.rs`
  - `AppError` 与 `CommandResult<T>`。

### 4.6 设备接入层

- `device_gateway.rs`
  - 定义 `DeviceGateway` 与 `DeviceGatewayFactory` 抽象。
- `comm_dll.rs`
  - `CommDllv2.dll` 动态加载与参数指令调用封装。

### 4.7 数据层

- `db.rs`
  - SQLite 初始化与写入。
  - 存储测试会话与检查项。
  - 支持日期范围导出 CSV、查询可导出日期。

---

## 5. 配置与资源

- `src-tauri/config/threshold.toml`
  - 连接模式（serial/network）、连接参数、`read_timeout_ms`、日志级别等。
- `src-tauri/config/tests.toml`
  - 测试组、stage、命令参数、阈值规则。
- `src-tauri/tauri.conf.json`
  - Tauri 打包与资源配置。
- 运行资源：
  - `src-tauri/CommDllv2.dll`
  - `src-tauri/libcrypto-3-x64.dll`

---

## 6. 关键调用链

1. 前端 `App.svelte` 触发动作（开始测试/重测/停止/确认）。
2. `src/services/tauri.ts` 通过 `invoke` 调用 Rust 命令。
3. `commands.rs` 将测试任务交给 `test_service.rs`。
4. `test_service.rs` 统一调用 `test_runner.rs::run_group`。
5. `test_runner.rs` 通过 `DeviceGateway` 调用 `comm_dll.rs` 完成设备通信。
6. 运行时事件回推前端：
   - `test-group-complete`
   - `key-state-update`
   - `front-light-confirm-request`
7. 前端收到前灯确认请求后调用 `confirm_front_light`，后端继续并返回 `ParamId606` 最终结果。

---

## 7. 开发命令（项目根目录）

```bash
# 安装依赖
bun install

# 前端开发
bun run dev

# 前端构建
bun run build

# Tauri 联调
bun run tauri dev

# Tauri 打包
bun run tauri build
```

> 也可用 `npm run build` 执行前端构建（当前项目脚本兼容）。
