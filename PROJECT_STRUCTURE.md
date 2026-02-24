# IRS Tools 项目结构说明（Tauri + Svelte + Rust）

## 1. 技术栈总览
- 前端：Svelte + TypeScript + Vite（包管理使用 Bun）
- 后端：Rust + Tauri v2
- 设备通信：通过 `CommDllv2.dll`（Windows DLL）调用底层接口

## 2. 前端结构（`src/`）

### 2.1 入口与启动
- `src/main.ts`
  - 前端入口，挂载 `App.svelte`。
- `src/App.svelte`
  - 全局页面状态与核心交互控制器。
  - 管理测试状态、结果列表、语言切换、设置页切换、错误信息等。

### 2.2 页面与组件分层
- `src/views/MainView.svelte`
  - 主测试页面，包含“开始测试”、状态展示、结果表格。
- `src/views/SettingsView.svelte`
  - 设置页面，包含连接参数配置和 About 信息。
- `src/components/`
  - 细分 UI 组件（如 `Sidebar`、`ResultsTable`、`SettingsForm`、`StatusCard`、`AboutCard`）。

### 2.3 类型与服务
- `src/types/`
  - 前端类型定义（配置、测试结果、状态、i18n）。
- `src/services/tauri.ts`
  - 前后端桥接层，封装 Tauri API：
    - `invoke(...)` 调后端命令
    - `listen(...)` 监听后端事件（如 `test-group-complete`）

## 3. 后端结构（`src-tauri/src/`）

### 3.1 入口与命令注册
- `src-tauri/src/main.rs`
  - Tauri 应用入口。
  - 注册命令：
    - `start_test`
    - `retest_group`
    - `show_main_window`
    - `get_base_config`
    - `save_base_config`

### 3.2 命令层（业务编排）
- `src-tauri/src/commands.rs`
  - 实现所有 `#[tauri::command]`。
  - 当前是“薄命令层”：主要负责命令入口转发到 `TestService`，不再承载测试编排细节。

### 3.3 应用服务层（测试编排）
- `src-tauri/src/test_service.rs`
  - 负责测试流程编排（读取配置、创建网关、循环执行、汇总结果）。
  - 在每组测试完成后发送 `test-group-complete` 事件。
  - 通过 `DeviceGatewayFactory` 注入具体设备实现，降低对底层的直接依赖。

### 3.4 测试执行层
- `src-tauri/src/test_runner.rs`
  - 负责单个测试组执行与结果构造（含阈值检查）。
  - 包含不依赖硬件的单元测试（使用 fake gateway）。

### 3.5 数据模型层
- `src-tauri/src/models.rs`
  - 定义配置模型、测试组模型、检查项模型、测试结果模型。
  - 包含 `ConnectionConfig`、`BaseConfig`、`TestGroup`、`TestResult`、`TestSummary` 等。
  - 提供不同命令返回结果的结构体与格式化显示。

### 3.6 配置层
- `src-tauri/src/config.rs`
  - 配置读取与写入逻辑：
    - `threshold.json`：基础连接参数和超时设置
    - `tests.json`：测试项及阈值规则
  - 启动时将“基础配置 + 测试配置”组合成完整测试配置。

### 3.7 设备网关抽象层
- `src-tauri/src/device_gateway.rs`
  - 定义 `DeviceGateway` 与 `DeviceGatewayFactory` 抽象。
  - 提供 `DllDeviceGatewayFactory` 默认实现，将 `CommSession` 适配为网关接口。

### 3.8 设备通信层（DLL 封装）
- `src-tauri/src/comm_dll.rs`
  - 使用 `libloading` 动态加载 `CommDllv2.dll`。
  - 封装串口/网络连接与参数命令调用：
    - `param_id068`
    - `param_id588`
    - `param_id654`
    - `param_id272`
    - `param_id080`
    - `param_id606`
  - 统一处理 return code，并转换成可读错误信息。

### 3.9 事件与错误模型
- `src-tauri/src/events.rs`
  - 事件名常量定义（如 `TEST_GROUP_COMPLETE`），避免魔法字符串散落。
- `src-tauri/src/types.rs`
  - `CommandResult<T>` 使用结构化 `AppError`，替代纯字符串错误。

## 4. 配置与资源
- `src-tauri/config/threshold.json`
  - 连接方式（serial/network）和读取超时。
- `src-tauri/config/tests.json`
  - 测试组与阈值规则，属于“配置驱动测试”核心。
- `src-tauri/tauri.conf.json`
  - Tauri 构建配置：
    - 开发前置命令：`bun run dev`
    - 打包前置命令：`bun run build`
    - 资源打包包括：配置文件、`CommDllv2.dll`、`libcrypto-3-x64.dll`

## 5. 前后端调用链
1. 前端在 `App.svelte` 中触发动作（开始测试/重测/保存设置）。
2. `src/services/tauri.ts` 使用 `invoke` 调用 Rust 命令。
3. `commands.rs` 将请求转发给 `test_service.rs`。
4. `test_service.rs` 通过 `device_gateway.rs` 创建设备网关，并调用 `test_runner.rs` 执行测试组。
5. `comm_dll.rs` 作为底层 DLL 适配实现，完成实际通信。
6. Rust 将测试结果返回前端，并通过事件推送分组进度。
7. 前端更新状态与表格，完成 UI 展示。

## 6. 项目特征总结
- 结构清晰：前端展示与后端设备逻辑边界明确。
- 强配置驱动：测试项和阈值放在 JSON，便于非代码调整。
- 实时反馈：后端事件推送让前端可逐项刷新结果。
- 工程组合稳定：Bun + Vite（前端开发效率）与 Tauri + Rust（本地能力与性能）结合。
- 可测试性提升：核心测试执行逻辑可通过 fake gateway 做单测，无需真实硬件。
- 依赖方向优化：应用层依赖网关抽象而非 DLL 细节，实现更符合 DIP。

## 7. 新同事 5 分钟上手

### 7.1 环境准备（Windows）
- 安装 Rust 工具链（`cargo` / `rustc`）
- 安装 Bun
- 准备 Tauri 运行环境（WebView2 等）
- 确认仓库中 `src-tauri/CommDllv2.dll` 存在

### 7.2 常用命令（项目根目录执行）
```powershell
# 安装前端依赖（如已安装可跳过）
bun install

# 仅前端开发（Vite）
bun run dev

# 构建前端静态资源
bun run build

# Tauri 开发模式（前后端一起）
bun run tauri dev

# Tauri 打包（MSI）
bun run tauri build
```

### 7.3 首次联调建议顺序
1. 先执行 `bun run dev`，确认前端页面可正常打开。
2. 再执行 `bun run tauri dev`，确认桌面窗口可启动。
3. 打开设置页，检查连接模式（serial/network）和超时参数。
4. 检查 `src-tauri/config/tests.json` 测试项是否符合当前设备。
5. 执行“开始测试”，观察是否持续收到分组完成结果。

### 7.4 常见问题排查
- 问题：启动后看不到窗口
  - 排查：项目中默认窗口 `visible: false`，前端会调用 `show_main_window` 显示；若调用失败看 Rust 日志。
- 问题：提示找不到 `CommDllv2.dll`
  - 排查：确认 DLL 在资源目录或运行目录；`commands.rs` 会按多个候选路径查找。
- 问题：连接失败（串口或网络）
  - 排查：检查 `threshold.json` 的连接参数、设备在线状态、端口占用、网络连通性。
- 问题：测试项解析失败
  - 排查：检查 `tests.json` 的字段名、`command` 值、`checks` 结构与枚举值是否匹配。
- 问题：前端能启动但无法调用后端命令
  - 排查：确认命令已在 `main.rs` 的 `invoke_handler` 注册，且前端 `invoke` 名称一致。
