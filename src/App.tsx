import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

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

const App = () => {
  const [results, setResults] = useState<TestResult[]>([]);
  const [language, setLanguage] = useState<Language>("zh");
  const [statusKey, setStatusKey] = useState<StatusKey>("idle");
  const [summaryState, setSummaryState] = useState<"pass" | "fail" | "pending" | "idle">("idle");
  const [running, setRunning] = useState(false);
  const [retesting, setRetesting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const text = textMap[language];

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    invoke("show_main_window").catch((err) => {
      console.error("Failed to show main window", err);
    });

    listen<TestResult>("test-group-complete", (event) => {
      const incoming = event.payload;
      setResults((prev) => {
        const existingIndex = prev.findIndex((item) => item.name === incoming.name);
        if (existingIndex === -1) {
          return [...prev, incoming];
        }
        const next = [...prev];
        next[existingIndex] = incoming;
        return next;
      });
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
  }, []);

  const handleStart = async () => {
    setRunning(true);
    setRetesting(null);
    setError(null);
    setResults([]);
    setStatusKey("running");
    setSummaryState("pending");

    try {
      const summary = await invoke<TestSummary>("start_test");
      setResults(summary.results);
      setStatusKey("done");
      setSummaryState(summary.overall_passed ? "pass" : "fail");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      setStatusKey("failed");
      setSummaryState("fail");
    } finally {
      setRunning(false);
    }
  };

  const handleRetest = async (groupName: string) => {
    if (running || summaryState === "pending" || summaryState === "idle") {
      return;
    }
    setRetesting(groupName);
    setError(null);

    try {
      const updated = await invoke<TestResult>("retest_group", { groupName });
      setResults((prev) => {
        const next = prev.map((item) => (item.name === updated.name ? updated : item));
        const allPassed = next.every((item) => item.passed);
        setSummaryState((prevState) => (prevState === "idle" ? prevState : allPassed ? "pass" : "fail"));
        return next;
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      setRetesting(null);
    }
  };

  const summaryLabel = text.summary[summaryState];

  return (
    <main className="container">
      <header>
        <div>
          <h1>{text.title}</h1>
          <p className="subtitle">{text.subtitle}</p>
        </div>
        <div className="header-actions">
          <button className="primary" onClick={handleStart} disabled={running}>
            {text.start}
          </button>
          <button className="lang-toggle" onClick={() => setLanguage((prev) => (prev === "zh" ? "en" : "zh"))}>
            {text.langLabel}
          </button>
        </div>
      </header>

      <section className="status">
        <div>
          <h2>{text.statusTitle}</h2>
          <p>{text.status[statusKey]}</p>
        </div>
        <div className="summary" data-state={summaryState === "idle" ? undefined : summaryState}>
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
            {error ? (
              <tr>
                <td colSpan={5} className="error">
                  {error}
                </td>
              </tr>
            ) : results.length === 0 ? (
              <tr>
                <td colSpan={5} className="empty">
                  {text.table.empty}
                </td>
              </tr>
            ) : (
              results.flatMap((group) => {
                const groupRow = (
                  <tr key={group.name} className="group-row">
                    <td>{group.name}</td>
                    <td>
                      <code>{group.command}</code>
                    </td>
                    <td>-</td>
                    <td>-</td>
                    <td className={group.passed ? "pass" : "fail"}>
                      <div className="group-actions">
                        <span>{group.passed ? text.pass : text.fail}</span>
                        <button
                          className="retest"
                          onClick={() => handleRetest(group.name)}
                          disabled={
                            running || retesting !== null || summaryState === "pending" || summaryState === "idle"
                          }
                        >
                          {retesting === group.name ? text.retesting : text.retest}
                        </button>
                      </div>
                    </td>
                  </tr>
                );

                const childRows = group.checks.map((check) => (
                  <tr key={`${group.name}-${check.name}`} className="child-row">
                    <td className="indent">{check.name}</td>
                    <td>-</td>
                    <td>
                      {check.min === null || check.max === null ? "-" : `${check.min} ~ ${check.max}`}
                    </td>
                    <td>{check.value === null ? "-" : check.value}</td>
                    <td className={check.passed ? "pass" : "fail"}>{check.passed ? text.pass : text.fail}</td>
                  </tr>
                ));

                return [groupRow, ...childRows];
              })
            )}
          </tbody>
        </table>
      </section>

      <section className="note">
        <h3>{text.configTitle}</h3>
        <p>
          {text.configPrefix} <code>src-tauri/config/thresholds.json</code> {text.configSuffix}
        </p>
      </section>
    </main>
  );
};

export default App;
