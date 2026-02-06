<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type {
    TestResult,
    TestSummary,
    Language,
    StatusKey,
    SummaryState,
  } from "./types";
  import { getTranslation } from "./lib/locales";
  import ResultsTable from "./components/ResultsTable.svelte";

  // Svelte 5 Runes state management
  let results = $state<TestResult[]>([]);
  let language = $state<Language>("zh");
  let statusKey = $state<StatusKey>("idle");
  let summaryState = $state<SummaryState>("idle");
  let running = $state(false);
  let retesting = $state<string | null>(null);
  let error = $state<string | null>(null);

  // Derived state
  const text = $derived(getTranslation(language));
  const summaryLabel = $derived(text.summary[summaryState]);

  function toggleLanguage() {
    language = language === "zh" ? "en" : "zh";
  }

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
        results.push(incoming);
      } else {
        results[existingIndex] = incoming;
      }
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

      const idx = results.findIndex((r) => r.name === updated.name);
      if (idx !== -1) {
        results[idx] = updated;
      }

      const allPassed = results.every((item) => item.passed);
      summaryState = allPassed ? "pass" : "fail";
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
      <button class="primary" onclick={handleStart} disabled={running}>
        {text.start}
      </button>
      <button class="lang-toggle" onclick={toggleLanguage}>
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

  <ResultsTable
    {results}
    {text}
    {error}
    {running}
    {summaryState}
    {retesting}
    onRetest={handleRetest}
  />

  <section class="note">
    <h3>{text.configTitle}</h3>
    <p>
      {text.configPrefix}
      <code>src-tauri/config/threshold.json</code>
      {text.configMiddle}
      <code>src-tauri/config/tests.json</code>
      {text.configSuffix}
    </p>
  </section>
</main>
