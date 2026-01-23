# irs-tools

## 配置文件说明

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
