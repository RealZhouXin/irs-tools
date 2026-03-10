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

## CI / Release

- 普通提交和 Pull Request 会触发 GitHub Actions 的 Windows CI，执行：
  - `bun install --frozen-lockfile`
  - `bun run build`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
- 推送 `v*` 格式的 tag 会触发发布工作流，自动构建 Tauri 的 MSI 并上传到 GitHub Releases。

示例：

```bash
git tag v0.3.0
git push origin v0.3.0
```

生成的安装包来自：

```text
src-tauri/target/release/bundle/msi/*.msi
```

## 配置说明

配置文件位于 `src-tauri/config/threshold.json` 与 `src-tauri/config/tests.json`，分别用于设置连接方式/超时与测试项。

字段说明：
- `connection`: 连接方式配置。
  - `mode`: `"serial"` 或 `"network"`.
  - `port_number`: 串口号（仅在 `mode` 为 `"serial"` 时使用）。
  - `ip_address`: 设备 IP（仅在 `mode` 为 `"network"` 时使用）。
  - `port`: 设备端口（仅在 `mode` 为 `"network"` 时使用）。
- `read_timeout_ms`: 读取超时时间（毫秒）。
- `tests`: 测试大项列表，每个大项对应一条命令（在 `tests.json` 中）。
  - `name`: 大项名称（用于 UI 展示）。
  - `command`: 命令类型，例如 `"param_id588"`、`"param_id654"`、`"param_id272"`、`"param_id080"`。
  - `checks`: 子项列表（仅对读取类命令如 `param_id588` 使用）。
    - `name`: 子项名称。
    - `output`: 返回字段标识，例如 `"maj_par_sw_ver"`。
    - `min` / `max`: 阈值范围。
  - 其他命令参数：如 `param_id606` 需要 `front_light_mode`、`power`。

## 配置示例

```json
// src-tauri/config/threshold.json
{
  "connection": {
    "mode": "network",
    "ip_address": "10.101.1.100",
    "port": "12345"
  },
  "read_timeout_ms": 1000
}

// src-tauri/config/tests.json
{
  "tests": [
    {
      "name": "080 割草机状态",
      "command": "param_id080",
      "checks": [
        { "name": "mower_main_p", "output": "mower_main_p", "min": 0, "max": 255 },
        { "name": "mower_sub_state", "output": "mower_sub_state", "min": 0, "max": 255 },
        { "name": "time_stp_nxt_start", "output": "time_stp_nxt_start", "min": 0, "max": 4294967295 },
        { "name": "batt_stat", "output": "batt_stat", "min": 0, "max": 255 },
        { "name": "stat_flags", "output": "stat_flags", "min": 0, "max": 65535 },
        { "name": "wrless_con_stat", "output": "wrless_con_stat", "min": 0, "max": 255 },
        { "name": "sign_quality", "output": "sign_quality", "min": 0, "max": 255 },
        { "name": "source_for_next_start_stop", "output": "source_for_next_start_stop", "min": 0, "max": 255 },
        { "name": "notify", "output": "notify", "min": 0, "max": 65535 },
        { "name": "configuration_hash", "output": "configuration_hash", "min": 0, "max": 255 }
      ]
    },
    {
      "name": "272 电池信息",
      "command": "param_id272",
      "checks": [
        { "name": "batt_pack_pn", "output": "batt_pack_pn", "min": 0, "max": 999999999 },
        { "name": "batt_pack_rev", "output": "batt_pack_rev", "min": 0, "max": 65535 },
        { "name": "batt_pack_prod_date", "output": "batt_pack_prod_date", "min": 0, "max": 999999999 },
        { "name": "batt_sw_ver", "output": "batt_sw_ver", "min": 0, "max": 999999999 },
        { "name": "batt_ser_no", "output": "batt_ser_no", "min": 0, "max": 999999999 },
        { "name": "batt_dev_gr_no", "output": "batt_dev_gr_no", "min": 0, "max": 999999999 },
        { "name": "batt_sub_dev_no", "output": "batt_sub_dev_no", "min": 0, "max": 999999999 },
        { "name": "batt_var_no", "output": "batt_var_no", "min": 0, "max": 65535 },
        { "name": "bms_dev_gr_no", "output": "bms_dev_gr_no", "min": 0, "max": 65535 },
        { "name": "bms_sub_dev_no", "output": "bms_sub_dev_no", "min": 0, "max": 65535 },
        { "name": "bms_var_no", "output": "bms_var_no", "min": 0, "max": 65535 },
        { "name": "bms_pcba_pn", "output": "bms_pcba_pn", "min": 0, "max": 999999999 },
        { "name": "bms_pcba_rev", "output": "bms_pcba_rev", "min": 0, "max": 65535 },
        { "name": "bms_temp_sensor_type", "output": "bms_temp_sensor_type", "min": 0, "max": 999999999 }
      ]
    },
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
      "name": "654 系统软件版本",
      "command": "param_id654",
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
- 配置：`src-tauri/config/threshold.json`（连接/超时）与 `src-tauri/config/tests.json`（测试项）。

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
- 通过 CommDllv2.dll 调用测试指令（如 `ParamId080`、`ParamId272`、`ParamId588`、`ParamId654`、`ParamId606`）。
- 一条命令可包含多个子项阈值对比，减少重复指令发送。
- 支持按大项重测并更新结果。

## 常见问题

1. 提示无法加载 DLL？
   - 确保 `CommDllv2.dll` 位于资源目录或程序当前目录。
2. `ConnectMowerViaNetwork` 连接失败？
   - 检查 `ip_address` 与 `port` 配置是否正确。
3. 读取超时或无返回？
   - 调整 `read_timeout_ms`，并确认设备处于可响应状态。
