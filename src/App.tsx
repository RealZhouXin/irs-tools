import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

type TestResult = {
  name: string;
  command: string;
  min: number | null;
  max: number | null;
  value: number | null;
  passed: boolean;
  raw_response: string;
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
  const [error, setError] = useState<string | null>(null);

  const handleStart = async () => {
    setRunning(true);
    setError(null);
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
      setResults([]);
      setStatusText("检测失败");
      setSummaryState("fail");
    } finally {
      setRunning(false);
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
              results.map((item) => (
                <tr key={item.name}>
                  <td>{item.name}</td>
                  <td>
                    <code>{item.command}</code>
                  </td>
                  <td>{item.min === null || item.max === null ? "-" : `${item.min} ~ ${item.max}`}</td>
                  <td>{item.value === null ? "-" : item.value}</td>
                  <td className={item.passed ? "pass" : "fail"}>
                    {item.passed ? "通过" : "未通过"}
                  </td>
                </tr>
              ))
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
