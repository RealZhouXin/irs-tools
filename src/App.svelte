<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  type CheckResult = {
    name: string;
    min: number | null;
    max: number | null;
    value: number | null;
    passed: boolean;
  };

  type TestResult = {
    name: string;
    command: string;
    raw_response: string;
    passed: boolean;
    checks: CheckResult[];
  };

  type TestSummary = {
    results: TestResult[];
    overall_passed: boolean;
  };

  type Language = "zh" | "en";
  type StatusKey = "idle" | "running" | "done" | "failed";

  const textMap = {
    zh: {
      title: "检测软件",
      subtitle: "点击开始检测后，将通过 DLL 调用测试指令并返回检测结果。",
      start: "开始检测",
      statusTitle: "检测状态",
      status: {
        idle: "等待开始",
        running: "检测中，请稍候...",
        done: "检测完成",
        failed: "检测失败",
      },
      summary: {
        pass: "全部通过",
        fail: "未通过",
        pending: "进行中",
        idle: "未开始",
      },
      table: {
        group: "检测项",
        command: "命令",
        range: "阈值范围",
        value: "检测值",
        result: "结果",
        empty: "尚未开始检测",
      },
      pass: "通过",
      fail: "未通过",
      retest: "重测",
      retesting: "重测中...",
      configTitle: "配置说明",
      configPrefix: "检测阈值与连接方式可在",
      configSuffix: "中配置。",
      langLabel: "EN",
    },
    en: {
      title: "Test Console",
      subtitle: "Click Start to run DLL test commands and show the results.",
      start: "Start Test",
      statusTitle: "Status",
      status: {
        idle: "Ready",
        running: "Running...",
        done: "Completed",
        failed: "Failed",
      },
      summary: {
        pass: "All Pass",
        fail: "Failed",
        pending: "In Progress",
        idle: "Not Started",
      },
      table: {
        group: "Test Item",
        command: "Command",
        range: "Range",
        value: "Value",
        result: "Result",
        empty: "No tests started.",
      },
      pass: "Pass",
      fail: "Fail",
      retest: "Retest",
      retesting: "Retesting...",
      configTitle: "Configuration",
      configPrefix: "Thresholds and connection settings are in",
      configSuffix: ".",
      langLabel: "中文",
    },
  } as const;

  let results: TestResult[] = [];
  let language: Language = "zh";
  let statusKey: StatusKey = "idle";
  let summaryState: "pass" | "fail" | "pending" | "idle" = "idle";
  let running = false;
  let retesting: string | null = null;
  let error: string | null = null;

  $: text = textMap[language];
  $: summaryLabel = text.summary[summaryState];

  onMount(() => {
    let unlisten: (() => void) | null = null;

    invoke("show_main_window").catch((err) => {
      console.error("Failed to show main window", err);
    });

    listen<TestResult>("test-group-complete", (event) => {
      const incoming = event.payload;
      const existingIndex = results.findIndex(
        (item) => item.name === incoming.name,
      );
      if (existingIndex === -1) {
        results = [...results, incoming];
        return;
      }
      const next = [...results];
      next[existingIndex] = incoming;
      results = next;
    })
      .then((stop) => {
        unlisten = stop;
      })
      .catch((err) => {
        console.error("Failed to listen test-group-complete", err);
      });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  const handleStart = async () => {
    running = true;
    retesting = null;
    error = null;
    results = [];
    statusKey = "running";
    summaryState = "pending";

    try {
      const summary = await invoke<TestSummary>("start_test");
      results = summary.results;
      statusKey = "done";
      summaryState = summary.overall_passed ? "pass" : "fail";
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
      statusKey = "failed";
      summaryState = "fail";
    } finally {
      running = false;
    }
  };

  const handleRetest = async (groupName: string) => {
    if (running || summaryState === "pending" || summaryState === "idle") {
      return;
    }
    retesting = groupName;
    error = null;

    try {
      const updated = await invoke<TestResult>("retest_group", { groupName });
      const next = results.map((item) =>
        item.name === updated.name ? updated : item,
      );
      const allPassed = next.every((item) => item.passed);
      summaryState = allPassed ? "pass" : "fail";
      results = next;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
    } finally {
      retesting = null;
    }
  };
</script>

<main class="container">
  <header>
    <div>
      <h1>{text.title}</h1>
      <p class="subtitle">{text.subtitle}</p>
    </div>
    <div class="header-actions">
      <button class="primary" on:click={handleStart} disabled={running}>
        {text.start}
      </button>
      <button
        class="lang-toggle"
        on:click={() => {
          language = language === "zh" ? "en" : "zh";
        }}
      >
        {text.langLabel}
      </button>
    </div>
  </header>

  <section class="status">
    <div>
      <h2>{text.statusTitle}</h2>
      <p>{text.status[statusKey]}</p>
    </div>
    <div
      class="summary"
      data-state={summaryState === "idle" ? undefined : summaryState}
    >
      {summaryLabel}
    </div>
  </section>

  <section>
    <table>
      <thead>
        <tr>
          <th>{text.table.group}</th>
          <th>{text.table.command}</th>
          <th>{text.table.range}</th>
          <th>{text.table.value}</th>
          <th>{text.table.result}</th>
        </tr>
      </thead>
      <tbody>
        {#if error}
          <tr>
            <td colspan="5" class="error">
              {error}
            </td>
          </tr>
        {:else if results.length === 0}
          <tr>
            <td colspan="5" class="empty">
              {text.table.empty}
            </td>
          </tr>
        {:else}
          {#each results as group (group.name)}
            <tr class="group-row">
              <td>{group.name}</td>
              <td>
                <code>{group.command}</code>
              </td>
              <td>-</td>
              <td>-</td>
              <td class={group.passed ? "pass" : "fail"}>
                <div class="group-actions">
                  <span>{group.passed ? text.pass : text.fail}</span>
                  <button
                    class="retest"
                    on:click={() => handleRetest(group.name)}
                    disabled={running ||
                      retesting !== null ||
                      summaryState === "pending" ||
                      summaryState === "idle"}
                  >
                    {retesting === group.name ? text.retesting : text.retest}
                  </button>
                </div>
              </td>
            </tr>
            {#each group.checks as check (group.name + "-" + check.name)}
              <tr class="child-row">
                <td class="indent">{check.name}</td>
                <td>-</td>
                <td>
                  {check.min === null || check.max === null
                    ? "-"
                    : `${check.min} ~ ${check.max}`}
                </td>
                <td>{check.value === null ? "-" : check.value}</td>
                <td class={check.passed ? "pass" : "fail"}>
                  {check.passed ? text.pass : text.fail}
                </td>
              </tr>
            {/each}
          {/each}
        {/if}
      </tbody>
    </table>
  </section>

  <section class="note">
    <h3>{text.configTitle}</h3>
    <p>
      {text.configPrefix} <code>src-tauri/config/thresholds.json</code>
      {text.configSuffix}
    </p>
  </section>
</main>
