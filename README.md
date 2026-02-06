# irs-tools

## 安装与运行

### 前置环境 (Windows)

1. **安装 Rust**
   - 访问 [Rust 官网](https://www.rust-lang.org/zh-CN/tools/install) 下载 `rustup-init.exe` 并安装。
   - 确保安装过程中包含了 Visual Studio C++ 生成工具。

2. **安装 Bun**
   - 在 PowerShell 中执行：
     ```powershell
     winget install oven-sh.bun
     ```

### 开发流程

1. 安装依赖（前端使用 bun）。

```bash
bun install
```

2. 运行开发环境（会启动前端 + Tauri 后端）。

```bash
bun run tauri dev
```

3. 构建发布包（可选）。

```bash
bun run tauri build
```

## 配置说明

配置文件位于 `src-tauri/config/thresholds.json`，用于设置连接方式与测试项。

字段说明：
- `connection`: 连接方式配置。
  - `mode`: `"serial"` 或 `"network"`.
  - `port_number`: 串口号（仅在 `mode` 为 `"serial"` 时使用）。
  - `ip_address`: 设备 IP（仅在 `mode` 为 `"network"` 时使用）。
  - `port`: 设备端口（仅在 `mode` 为 `"network"` 时使用）。
- `read_timeout_ms`: 读取超时时间（毫秒）。
- `tests`: 测试大项列表，每个大项对应一条命令。
  - `name`: 大项名称（用于 UI 展示）。
  - `command`: 命令类型，例如 `"param_id588"`。
  - `checks`: 子项列表（仅对读取类命令如 `param_id588` 使用）。
    - `name`: 子项名称。
    - `output`: 返回字段标识，例如 `"maj_par_sw_ver"`。
    - `min` / `max`: 阈值范围。
  - 其他命令参数：如 `param_id606` 需要 `front_light_mode`、`power`。

## 配置示例

```json
{
  "connection": {
    "mode": "network",
    "ip_address": "10.101.1.100",
    "port": "12345"
  },
  "read_timeout_ms": 1000,
  "tests": [
    {
      "name": "588 应用软件",
      "command": "param_id588",
      "checks": [
        { "name": "maj_par_sw_ver", "output": "maj_par_sw_ver", "min": 0, "max": 255 },
        { "name": "min_par_sw_ver", "output": "min_par_sw_ver", "min": 0, "max": 255 },
        { "name": "build_no", "output": "build_no", "min": 0, "max": 999999 }
      ]
    },
    {
      "name": "606 前灯设置",
      "command": "param_id606",
      "front_light_mode": 1,
      "power": 100
    }
  ]
}
```

## 开发结构

- 前端：`src/`，负责 UI 展示与调用 Tauri 命令。
- 后端：`src-tauri/`，负责加载 DLL、连接设备、执行测试与返回结果。
- 配置：`src-tauri/config/thresholds.json`，定义连接方式与测试项。

## 模块图

```text
src-tauri/src/
  main.rs
    -> commands.rs
    -> config.rs
    -> models.rs
    -> comm_dll.rs
    -> types.rs

commands.rs
  -> config.rs (读取配置)
  -> comm_dll.rs (加载 DLL + 连接设备)
  -> models.rs (配置/结果数据结构)
  -> types.rs (CommandResult)

comm_dll.rs
  -> models.rs (ParamId588Result, ConnectionConfig)
  -> types.rs (CommandResult)

config.rs
  -> models.rs (TestConfig)
  -> types.rs (CommandResult)
```

## 功能概述

- 支持串口或网络连接设备。
- 通过 CommDllv2.dll 调用测试指令（如 `ParamId588`、`ParamId606`）。
- 一条命令可包含多个子项阈值对比，减少重复指令发送。
- 支持按大项重测并更新结果。

## 常见问题

1. 提示无法加载 DLL？
   - 确保 `CommDllv2.dll` 位于资源目录或程序当前目录。
2. `ConnectMowerViaNetwork` 连接失败？
   - 检查 `ip_address` 与 `port` 配置是否正确。
3. 读取超时或无返回？
   - 调整 `read_timeout_ms`，并确认设备处于可响应状态。
