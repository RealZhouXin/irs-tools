# IRS Tools 任务编排模型优化方案

当前的 IRS Tools 任务编排模型在处理线性、扁平化的测试流程时非常高效，但随着业务复杂度的增加，为了支持多步骤联动、条件分支、复杂的错误恢复以及人工介入等更复杂的任务组，我们需要对现有的架构进行深度优化。

以下是五个核心优化方向及其详细分析与实施步骤：

## 1. 消除前后端耦合：抽象“人工介入”机制 (Human-in-the-loop)

### 痛点分析

在当前的 `App.svelte` 中，前端硬编码了 `if (incoming.command === "ParamId606")` 来拦截车灯测试并弹窗。这是一种“抽象泄漏”，如果未来新增喇叭测试、屏幕测试等需要人工确认的环节，前端代码需要不断修改，违背了配置驱动的初衷。

### 优化方案

将“是否需要人工确认”下沉到配置文件和数据模型中，前端只负责通用的交互展示。

### 实施步骤

1. **修改配置文件 (`tests.toml`)**：为需要人工确认的测试项增加 `interaction = "manual_confirm"` 字段。
2. **扩展数据模型 (`models.rs`)**：
   - 在 `TestGroup` 结构体中增加 `pub interaction: Option<String>` 字段。
   - 在 `TestResult` 结构体中增加 `pub interaction: Option<String>` 字段，以便将该属性传递给前端。
3. **修改执行逻辑 (`test_runner.rs`)**：在构建 `TestResult` 时，将 `interaction` 字段透传。
4. **重构前端逻辑 (`App.svelte` & `types/test.ts`)**：
   - 在 `TestResult` 类型中增加 `interaction?: string | null`。
   - 将硬编码的 `incoming.command === "ParamId606"` 替换为 `incoming.interaction === "manual_confirm"`。

---

## 2. 引入任务依赖与条件分支 (DAG / 状态机)

### 痛点分析

当前的 `test_service.rs` 是一个简单的线性循环。如果“前置电源检查”失败，后续的“电机测试”其实没有必要进行，但当前逻辑只能全部跑完（除非发生底层通信 Err）。这不仅浪费时间，还可能在硬件状态异常时强行发送危险指令。

假设我们有以下三个测试项：

1. **测试 A**：检查电池电压是否正常（> 20V）。
2. **测试 B**：开启割草电机，测试高负荷运转。
3. **测试 C**：检查主板温度传感器是否正常。

**痛点 1：危险的无效执行**
如果“测试 A（电池电压）”失败了（比如只有 10V，处于严重亏电状态），此时如果继续执行“测试 B（开启电机）”，可能会导致电池过放损坏，甚至引发主板保护锁死。

**痛点 2：浪费时间的无效等待**
如果某个前置模块根本不存在或通信失败，后续针对该模块的 10 个详细测试项都会依次执行、依次超时、依次报错，白白浪费大量测试时间。

### 优化方案

在配置模型中引入依赖图（DAG）或条件跳过机制，让测试编排器具备“思考”能力：**在执行当前任务前，先看看它的前置条件是否满足。**

### 实施步骤

1. **修改配置文件 (`tests.toml`)**：为测试项引入 `id`（唯一标识符）和 `depends_on`（依赖项数组）字段。

   ```toml
   [[tests]]
   id = "check_voltage"
   name = "01. 电池电压检查"
   command = "param_id068"
   # ... checks ...

   [[tests]]
   id = "motor_test"
   name = "02. 电机高负荷测试"
   command = "param_id588"
   # 核心：声明依赖。只有 check_voltage 成功了，才执行本测试
   depends_on = ["check_voltage"] 

   [[tests]]
   id = "temp_sensor"
   name = "03. 温度传感器检查"
   command = "param_id120"
   # 这个测试是独立的，不依赖电压，所以没有 depends_on
   ```
2. **扩展数据模型 (`models.rs`)**：

   - 在 `TestGroup` 中增加 `id` 和 `depends_on` 字段。
   - 扩展测试结果状态，引入 `Skipped(String)` 状态，替代简单的 `passed: bool`。

   ```rust
   #[derive(Debug, Deserialize, Serialize, Clone)]
   pub struct TestGroup {
       pub id: String, // 新增：唯一标识符
       pub name: String,
       #[serde(default)]
       pub depends_on: Vec<String>, // 新增：依赖的测试项 ID 列表
       #[serde(flatten)]
       pub command: CommandGroupSpec,
   }

   // 扩展测试结果的状态
   #[derive(Debug, Serialize, Clone, PartialEq)]
   #[serde(rename_all = "snake_case")]
   pub enum TestStatus {
       Passed,
       Failed,
       Skipped(String), // 记录被跳过的原因，例如："依赖项 check_voltage 失败"
   }

   #[derive(Debug, Serialize, Clone)]
   pub struct TestResult {
       pub id: String,
       pub name: String,
       pub status: TestStatus, // 替代原来的 pub passed: bool
       // ... 其他字段
   }
   ```
3. **重构编排器 (`test_service.rs`)**：

   - 在执行循环中维护一个“已完成任务的状态字典”。
   - 在执行每个任务前，检查其 `depends_on` 列表中的前置任务是否全部 `Passed`。
   - 如果前置任务失败或被跳过，则将当前任务标记为 `Skipped` 并记录原因，不再调用底层硬件接口。

   ```rust
   use std::collections::HashMap;

   pub fn start_test(&self) -> CommandResult<TestSummary> {
       // ... 前置准备 ...

       // 用于记录已经执行过的测试项的结果状态
       let mut execution_history: HashMap<String, TestStatus> = HashMap::new();
       let mut results = Vec::new();

       for group in tests {
           let group_id = group.id.clone();
           let group_name = group.name.clone();

           // 1. 依赖检查逻辑 (Dependency Check)
           let mut should_skip = false;
           let mut skip_reason = String::new();

           for dep_id in &group.depends_on {
               match execution_history.get(dep_id) {
                   Some(TestStatus::Passed) => {
                       // 依赖项通过了，继续检查下一个依赖
                       continue; 
                   }
                   Some(TestStatus::Failed) => {
                       should_skip = true;
                       skip_reason = format!("依赖项 [{}] 测试失败", dep_id);
                       break;
                   }
                   Some(TestStatus::Skipped(_)) => {
                       should_skip = true;
                       skip_reason = format!("依赖项 [{}] 被跳过", dep_id);
                       break;
                   }
                   None => {
                       // 依赖项还没执行？这说明 TOML 配置的顺序不对，或者依赖了不存在的 ID
                       should_skip = true;
                       skip_reason = format!("依赖项 [{}] 未执行或不存在", dep_id);
                       break;
                   }
               }
           }

           // 2. 执行或跳过
           let result = if should_skip {
               info!("Skipping group {}: {}", group_name, skip_reason);
               TestResult {
                   id: group_id.clone(),
                   name: group_name.clone(),
                   status: TestStatus::Skipped(skip_reason),
                   // ... 填充空数据
               }
           } else {
               // 正常执行硬件测试
               match run_group(gateway.as_ref(), group) {
                   Ok(res) => res,
                   Err(err) => { /* 处理底层错误并 break */ }
               }
           };

           // 3. 记录结果并推送到前端
           execution_history.insert(group_id, result.status.clone());
           self.app.emit(TEST_GROUP_COMPLETE, &result).ok();
           results.push(result);
       }

       // ... 退出测试模式 ...
   }
   ```
4. **前端 UI 适配**：在表格中增加对 `Skipped` 状态的展示（如灰色图标和跳过原因）。

   - **状态图标**：除了绿色的“通过”和红色的“失败”，增加一个灰色的“跳过”图标（如 ⏭️ 或 ➖）。
   - **原因展示**：在表格的“结果/备注”列，显示后端传来的 `skip_reason`（例如：“因前置电源测试失败而跳过”）。
   - **进度条计算**：跳过的任务不应该算作“失败”，但整个测试批次的最终结果（Summary）如果包含 Failed，则整体为 Fail。

### 进阶：条件分支 (Conditionals)

`depends_on` 解决的是“前置失败则不执行”的问题。更高级的编排还需要“条件分支”（If-Else）。
例如：**如果设备是型号 A，执行测试 X；如果是型号 B，执行测试 Y。**

可以在 TOML 中引入 `run_if` 表达式：

```toml
[[tests]]
id = "read_device_info"
command = "param_id068"

[[tests]]
id = "test_feature_for_model_a"
# 只有当 read_device_info 读取到的 build_no 大于 1000 时才执行
run_if = "read_device_info.build_no > 1000" 
command = "param_id122"
```

**实现思路**：
这需要在 Rust 中引入一个轻量级的表达式求值引擎（如 `evalexpr` 库）。

1. `test_service.rs` 需要将之前所有测试的 `CheckResult`（具体读取到的数值）保存到一个上下文中。
2. 遇到 `run_if` 时，将上下文注入表达式引擎，计算出 `true` 或 `false`。
3. 如果为 `false`，则将该任务标记为 `Skipped("条件不满足")`。

### 总结

引入 DAG（依赖图）是测试软件从“玩具”走向“工业级”的关键一步。

* **安全性**：防止在硬件状态异常时强行发送危险指令。
* **效率**：遇到前置阻断错误时迅速“剪枝”，大幅缩短无效测试时间。
* **实现成本**：在现有的 Rust 架构上实现 `depends_on` 非常简单（只需维护一个 HashMap 记录历史状态），性价比极高。建议作为第一步，强烈建议先实现 `depends_on`，后续再考虑复杂的 `run_if` 表达式。

---

## 3. 动态指令组合与轻量级脚本化 (Composite Commands)

### 痛点分析

在 `models.rs` 中，为了实现复杂的逻辑，硬编码了 `CuttingHeightSetAndVerify` 这种包含 `wait_ms` 和多次 `checks` 的复合指令。如果未来有更多类似的组合，Rust 的 Enum 会无限膨胀，且每次修改都需要重新编译。

例如，如果硬件工程师说：“我需要一个新测试：先开灯（606），等 1 秒，读电压（068），再关灯（606）”。你就必须去修改 `models.rs`，加一个 `LightOnAndCheckVoltage` 枚举，再去 `test_runner.rs` 写一堆逻辑，最后重新编译整个 Rust 后端。这违背了“配置驱动”的初衷。

### 优化方案

引入“声明式组合”，不再在 Rust 代码中硬编码复杂的业务逻辑，而是允许用户在 `tests.toml` 配置文件中，像搭积木一样，把基础的单条指令组合成一个复杂的测试流程。

“声明式组合”的意思是，我们在 Rust 中只保留最基础的原子指令（如 068, 468, 470, 606，以及一个基础的 `sleep` 指令）。然后，我们在 `models.rs` 中引入一个通用的 `Composite`（组合）类型。

### 实施步骤

1. **扩展指令枚举 (`models.rs`)**：

   - 增加基础的 `Sleep { wait_ms: u64 }` 指令。
   - 增加 `Composite { steps: Vec<CommandGroupSpec> }` 类型。

   ```rust
   pub enum CommandGroupSpec {
       ParamId068 { checks: Vec<ParamId068Check> },
       ParamId468 { cutting_height_mm: u8 },
       ParamId470 { checks: Vec<ParamId470Check> },
       Sleep { wait_ms: u64 }, // 新增一个基础的等待指令

       // 核心：支持将多个指令串联成一个数组
       Composite {
           steps: Vec<CommandGroupSpec> 
       }
   }
   ```
2. **修改执行逻辑 (`test_runner.rs`)**：

   - 为 `Composite` 类型编写递归或循环执行逻辑，按顺序执行 `steps` 中的原子指令。
   - 收集所有步骤的校验结果，汇总为最终的 `TestResult`。
3. **重构配置文件 (`tests.toml`)**：将原有的 `cutting_height_set_and_verify` 替换为 `composite` 类型的配置，将设置、等待、读取拆分为独立的步骤。

   **示例：割草高度设置与校验**

   ```toml
   [[tests]]
   name = "割草高度设置与校验"
   command = "composite"

     # 步骤 1：设置高度为 50mm
     [[tests.steps]]
     command = "param_id468"
     cutting_height_mm = 50

     # 步骤 2：等待电机转动 2000 毫秒
     [[tests.steps]]
     command = "sleep"
     wait_ms = 2000

     # 步骤 3：读取高度并校验
     [[tests.steps]]
     command = "param_id470"
     [[tests.steps.checks]]
     name = "current_height"
     output = "cutting_height"
     min = 48
     max = 52
   ```

   **示例：车灯开启时的电压压降测试**

   ```toml
   [[tests]]
   name = "车灯开启时的电压压降测试"
   command = "composite"

     [[tests.steps]]
     command = "param_id606"
     front_light_mode = 1 # 开灯

     [[tests.steps]]
     command = "sleep"
     wait_ms = 1000

     [[tests.steps]]
     command = "param_id068" # 测电压
     [[tests.steps.checks]]
     # ... 电压校验规则

     [[tests.steps]]
     command = "param_id606"
     front_light_mode = 0 # 关灯
   ```

### 总结

“声明式组合”就是把**流程控制权**从 Rust 源码交还给 TOML 配置文件。Rust 后端退化成一个纯粹的“指令执行引擎”，它只负责按顺序执行 `steps` 数组里的原子指令。这样，无论测试逻辑怎么变，只要不涉及新的底层 DLL 接口，就永远不需要重新编译软件。

---

## 4. 细粒度的生命周期与重试策略配置化

### 痛点分析

`test_runner.rs` 中硬编码了 `PARAM_ID470_MAX_RETRIES = 5` 和延迟时间。全局只有进入和退出测试模式的 Hook，缺乏针对特定指令的灵活配置。

### 优化方案

将重试策略提取到配置文件中，使 `test_runner.rs` 更加通用。

### 实施步骤

1. **修改数据模型 (`models.rs`)**：
   - 在需要重试的指令（如 `ParamId470`）中增加 `max_retries` 和 `retry_delay_ms` 字段。
   - 使用 `#[serde(default = "...")]` 提供默认值，保持向后兼容。
2. **修改执行逻辑 (`test_runner.rs`)**：
   - 移除硬编码的常量 `PARAM_ID470_MAX_RETRIES` 和 `PARAM_ID470_RETRY_DELAY_MS`。
   - 在调用 `build_checked_result_with_retry` 时，传入配置中的重试参数。
3. **更新配置文件 (`tests.toml`)**：允许在特定的测试项中覆盖默认的重试参数。

---

## 5. 任务层级嵌套与标签过滤 (Hierarchical & Tagging)

### 痛点分析

测试项是一个扁平的数组。如果设备有大量测试项，用户只想测特定模块（如“通信模块”），目前只能通过前端过滤或提供不同的配置文件。同时，前端 UI 展示长列表也会显得混乱。

目前你的 `tests.toml` 结构是这样的：

```toml
[[tests]]
name = "068 主板应用软件"
# ...
[[tests]]
name = "080 割草机状态"
# ...
[[tests]]
name = "272 电池信息"
# ...
```

在 Rust 中，它被解析为一个扁平的 `Vec<TestGroup>`。

**痛点 1：无法按模块/场景进行局部测试**

* 生产线上的工人可能只需要做“出厂全量测试”。
* 研发工程师在调试时，可能只想跑“电池相关测试”（比如 272 和其他电池指令）。
* 售后维修人员可能只想跑“快速健康检查”（只测几个核心状态）。
* **现状**：目前只能在前端点“开始测试”跑完全部，或者手动一个个点“重测”。如果想实现上述需求，只能维护多份不同的 `.toml` 文件（如 `tests_full.toml`, `tests_battery.toml`），这会导致配置难以维护。

**痛点 2：前端 UI 展示混乱**

* 当测试项达到 50 个时，前端的 `ResultsTable.svelte` 会变成一个极长的列表，用户很难快速定位到某个特定模块（比如“电机模块”）的测试结果。

### 优化方案

引入标签系统（Label）控制执行范围，引入树形层级结构（Hierarchical）优化 UI 展示。

### 实施步骤

#### 1. 引入标签系统 (Tagging)

参考gitlab ci配置的stage

* **修改配置文件 (`tests.toml`)**：为每个测试项增加一个 stage字段, 在配置的开头增加stage运行顺序
  ```toml
  stages = [software, battery, ...]
  [[tests]]
  name = "068 主板应用软件"
  command = "param_id068"
  stage = "software" 
  # ... checks ...

  [[tests]]
  name = "272 电池信息"
  command = "param_id272"
  stage = "battery" 
  # ... checks ...


  ```
* **修改 Rust 数据模型 (`models.rs`)**：在 `TestGroup` 结构体中增加 `stage` 字段。
  ```rust
  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct TestGroup {
      pub name: String,
      #[serde(default)] // 如果 toml 里没写 stage，默认为空数组
      pub stage: String, 
      #[serde(flatten)]
      pub command: CommandGroupSpec,
  }
  ```
* **修改后端执行逻辑 (`test_service.rs` & `commands.rs`)**：修改 `start_test` 命令，允许前端传入参数，参数为stage列表 `Vec <String>`, 后端按照列表依列表顺序执行每个stage的所有test
* **前端 UI 适配 (`App.svelte` & `MainView.svelte`)**：前端可以增加一个下拉框，让用户选择测试场景, 下拉框有全部测试、已经stages里的每个stage如果全部测试， 点开始测试的时候给后端传配置文件里stages的所有stage，如果选择特定的stage则给后端传选定的stage。同时在显示测试结果的时候将同一个stage的结果显示在一张卡片里



#### 2. 引入树形层级结构 (Hierarchical)

如果标签系统还不够，或者你希望前端的表格能**折叠/展开**（比如把所有电池测试放在一个“电池组”目录下），就需要引入树形结构。

* **修改配置文件 (`tests.toml`)**：将扁平的 `[[tests]]` 改为按 `category`（分类）组织。
  ```toml
  [[category]]
  name = "软件与状态检查"
    [[category.tests]]
    name = "068 主板应用软件"
    command = "param_id068"

    [[category.tests]]
    name = "080 割草机状态"
    command = "param_id080"

  [[category]]
  name = "电池系统"
    [[category.tests]]
    name = "272 电池信息"
    command = "param_id272"
  ```
* **修改 Rust 数据模型**：
  ```rust
  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct TestConfig {
      pub connection: ConnectionConfig,
      pub read_timeout_ms: u32,
      // 以前是 pub tests: Vec<TestGroup>,
      pub categories: Vec<TestCategory>, 
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct TestCategory {
      pub name: String,
      pub tests: Vec<TestGroup>,
  }
  ```
* **前端 UI 升级**：前端收到数据后，可以渲染成带有折叠面板（Accordion）的 UI。
  ```text
  ▼ 软件与状态检查 (2/2 通过)
     [通过] 068 主板应用软件
     [通过] 080 割草机状态
  ▼ 电池系统 (0/1 通过)
     [失败] 272 电池信息  <-- 展开查看详细 checks
  ```

### 总结：Tagging vs Hierarchical

* **标签系统 (Tagging)**：侧重于**执行控制**。它解决的是“在不同场景下，如何灵活地挑选一部分测试项来跑”的问题。实现成本极低，强烈推荐优先实现。
* **层级结构 (Hierarchical)**：侧重于**UI 展示与归类**。它解决的是“测试项太多，前端列表太长看不清”的问题。实现成本中等（需要改前端 UI 和后端解析逻辑）。

在实际的复杂工业测试软件中，通常是**两者结合使用**的：用层级结构来组织 UI 树，用标签系统来控制一键执行的范围执行“一键跑某类测试特定场景一键测试”。
