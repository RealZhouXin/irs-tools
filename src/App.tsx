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

const App = () => {
  const [results, setResults] = useState<TestResult[]>([]);
  const [statusText, setStatusText] = useState("等待开始");
  const [summaryState, setSummaryState] = useState<"pass" | "fail" | "pending" | "idle">("idle");
  const [running, setRunning] = useState(false);
  const [retesting, setRetesting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

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
    setStatusText("检测中，请稍候...");
    setSummaryState("pending");

    try {
      const summary = await invoke<TestSummary>("start_test");
      setResults(summary.results);
      setStatusText("检测完成");
      setSummaryState(summary.overall_passed ? "pass" : "fail");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      setStatusText("检测失败");
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

  const summaryLabel =
    summaryState === "pass"
      ? "全部通过"
      : summaryState === "fail"
        ? "未通过"
        : summaryState === "pending"
          ? "进行中"
          : "未开始";

  return (
    <main className="container">
      <header>
        <div>
          <h1>检测软件</h1>
          <p className="subtitle">点击开始检测后，将通过 DLL 调用测试指令并返回检测结果。</p>
        </div>
        <button className="primary" onClick={handleStart} disabled={running}>
          开始检测
        </button>
      </header>

      <section className="status">
        <div>
          <h2>检测状态</h2>
          <p>{statusText}</p>
        </div>
        <div className="summary" data-state={summaryState === "idle" ? undefined : summaryState}>
          {summaryLabel}
        </div>
      </section>

      <section>
        <table>
          <thead>
            <tr>
              <th>检测项</th>
              <th>命令</th>
              <th>阈值范围</th>
              <th>检测值</th>
              <th>结果</th>
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
                  尚未开始检测
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
                        <span>{group.passed ? "通过" : "未通过"}</span>
                        <button
                          className="retest"
                          onClick={() => handleRetest(group.name)}
                          disabled={
                            running || retesting !== null || summaryState === "pending" || summaryState === "idle"
                          }
                        >
                          {retesting === group.name ? "重测中..." : "重测"}
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
                    <td className={check.passed ? "pass" : "fail"}>{check.passed ? "通过" : "未通过"}</td>
                  </tr>
                ));

                return [groupRow, ...childRows];
              })
            )}
          </tbody>
        </table>
      </section>

      <section className="note">
        <h3>配置说明</h3>
        <p>
          检测阈值与连接方式可在 <code>src-tauri/config/thresholds.json</code> 中配置。
        </p>
      </section>
    </main>
  );
};

export default App;
