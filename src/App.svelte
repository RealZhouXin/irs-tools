<script lang="ts">
  import { onMount } from "svelte";
  import type {
    BaseConfig,
    LogLevel,
    Language,
    StatusKey,
    SummaryState,
    TestResult,
  } from "./types";
  import { getTranslation } from "./i18n/locales";
  import {
    TAURI_EVENTS,
    loadAppInfo,
    loadBaseConfig,
    retestGroup,
    saveBaseConfig,
    showMainWindow,
    startTest,
    subscribeTestGroupComplete,
  } from "./services/tauri";
  import Sidebar from "./components/Sidebar.svelte";
  import MainView from "./views/MainView.svelte";
  import SettingsView from "./views/SettingsView.svelte";

  type SettingsDraft = {
    mode: "network" | "serial";
    ip_address: string;
    port: string;
    port_number: number;
    read_timeout_ms: number;
    log_level: LogLevel;
  };

  // Svelte 5 Runes state management
  let results = $state<TestResult[]>([]);
  let language = $state<Language>("zh");
  let statusKey = $state<StatusKey>("idle");
  let summaryState = $state<SummaryState>("idle");
  let running = $state(false);
  let retesting = $state<string | null>(null);
  let error = $state<string | null>(null);
  let view = $state<"main" | "settings">("main");

  let settingsDraft = $state<SettingsDraft | null>(null);
  let settingsSaving = $state(false);
  let settingsSaved = $state(false);
  let settingsError = $state<string | null>(null);
  let appName = $state<string | null>(null);
  let appVersion = $state<string | null>(null);
  let tauriVersion = $state<string | null>(null);
  let aboutError = $state<string | null>(null);

  // Derived state
  const text = $derived(getTranslation(language));
  const summaryLabel = $derived(text.summary[summaryState]);

  function toggleLanguage() {
    language = language === "zh" ? "en" : "zh";
  }

  function applyConfigToDraft(config: BaseConfig) {
    if (config.connection.mode === "network") {
      settingsDraft = {
        mode: "network",
        ip_address: config.connection.ip_address,
        port: config.connection.port,
        port_number: 1,
        read_timeout_ms: config.read_timeout_ms,
        log_level: config.log_level ?? "info",
      };
    } else {
      settingsDraft = {
        mode: "serial",
        ip_address: "",
        port: "",
        port_number: config.connection.port_number,
        read_timeout_ms: config.read_timeout_ms,
        log_level: config.log_level ?? "info",
      };
    }
  }

  function handleIncomingResult(incoming: TestResult) {
    const existingIndex = results.findIndex((item) => item.name === incoming.name);
    if (existingIndex === -1) {
      results.push(incoming);
    } else {
      results[existingIndex] = incoming;
    }
  }

  onMount(() => {
    let unlisten: (() => void) | null = null;

    showMainWindow().catch((err) => {
      console.error("Failed to show main window", err);
    });

    loadBaseConfig()
      .then(applyConfigToDraft)
      .catch((err) => {
        console.error("Failed to load config", err);
      });

    subscribeTestGroupComplete(handleIncomingResult)
      .then((stop) => {
        unlisten = stop;
      })
      .catch((err) => {
        console.error(`Failed to listen ${TAURI_EVENTS.testGroupComplete}`, err);
      });

    loadAppInfo()
      .then(({ name, version, tauriVersion: tauri }) => {
        appName = name;
        appVersion = version;
        tauriVersion = tauri;
      })
      .catch((err) => {
        const message = err instanceof Error ? err.message : String(err);
        aboutError = message;
      });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  const handleOpenTests = () => {
    view = "main";
  };

  const handleOpenSettings = () => {
    view = "settings";
  };

  const handleStart = async () => {
    running = true;
    retesting = null;
    error = null;
    results = [];
    statusKey = "running";
    summaryState = "pending";

    try {
      const summary = await startTest();
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
      const updated = await retestGroup(groupName);
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

  const handleSettingsSave = async () => {
    if (!settingsDraft || settingsSaving) {
      return;
    }
    settingsSaving = true;
    settingsSaved = false;
    settingsError = null;

    const config: BaseConfig =
      settingsDraft.mode === "network"
        ? {
            connection: {
              mode: "network",
              ip_address: settingsDraft.ip_address,
              port: settingsDraft.port,
            },
            read_timeout_ms: settingsDraft.read_timeout_ms,
            log_level: settingsDraft.log_level,
          }
        : {
            connection: {
              mode: "serial",
              port_number: settingsDraft.port_number,
            },
            read_timeout_ms: settingsDraft.read_timeout_ms,
            log_level: settingsDraft.log_level,
          };

    try {
      const saved = await saveBaseConfig(config);
      if (saved.connection.mode === "network") {
        settingsDraft = {
          mode: "network",
          ip_address: saved.connection.ip_address,
          port: saved.connection.port,
          port_number: settingsDraft.port_number,
          read_timeout_ms: saved.read_timeout_ms,
          log_level: saved.log_level,
        };
      } else {
        settingsDraft = {
          mode: "serial",
          ip_address: settingsDraft.ip_address,
          port: settingsDraft.port,
          port_number: saved.connection.port_number,
          read_timeout_ms: saved.read_timeout_ms,
          log_level: saved.log_level,
        };
      }
      settingsSaved = true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      settingsError = message;
    } finally {
      settingsSaving = false;
    }
  };
</script>

<div class="app-shell">
  <Sidebar
    {view}
    {text}
    onOpenTests={handleOpenTests}
    onOpenSettings={handleOpenSettings}
  />

  <div class="content">
    {#if view === "main"}
      <MainView
        {text}
        {results}
        {error}
        {running}
        {statusKey}
        {summaryState}
        {summaryLabel}
        {retesting}
        onStart={handleStart}
        onRetest={handleRetest}
        onToggleLanguage={toggleLanguage}
      />
    {:else}
      <SettingsView
        {text}
        {settingsDraft}
        {settingsSaving}
        {settingsSaved}
        {settingsError}
        {aboutError}
        {appName}
        {appVersion}
        {tauriVersion}
        onToggleLanguage={toggleLanguage}
        onSave={handleSettingsSave}
      />
    {/if}
  </div>
</div>
