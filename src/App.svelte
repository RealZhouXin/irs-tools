<script lang="ts">
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
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
    loadTestStages,
    exportTestResultsCsv,
    retestGroup,
    saveBaseConfig,
    showMainWindow,
    startTest,
    stopTest,
    subscribeTestGroupComplete,
  } from "./services/tauri";
  import Sidebar from "./components/Sidebar.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import ExportDialog from "./components/ExportDialog.svelte";
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

  const ALL_STAGES_VALUE = "__all__";

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
  let availableStages = $state<string[]>([]);
  let selectedStage = $state<string>(ALL_STAGES_VALUE);
  let pendingLightConfirm = $state<TestResult | null>(null);
  let showLightConfirmDialog = $state(false);
  let showExportDialog = $state(false);
  let exportStartDate = $state<string | null>(null);
  let exportEndDate = $state<string | null>(null);
  let exporting = $state(false);
  let stopping = $state(false);
  let exportError = $state<string | null>(null);
  let exportSuccess = $state<string | null>(null);

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
    if (incoming.command === "ParamId606" && incoming.passed) {
      pendingLightConfirm = incoming;
      showLightConfirmDialog = true;
      return;
    }
    upsertResult(incoming);
  }

  function upsertResult(result: TestResult) {
    const existingIndex = results.findIndex((item) => item.name === result.name);
    if (existingIndex === -1) {
      results.push(result);
    } else {
      results[existingIndex] = result;
    }
  }

  function recalcSummaryState() {
    if (results.length === 0) {
      summaryState = "idle";
      return;
    }
    summaryState = results.every((item) => item.passed) ? "pass" : "fail";
  }

  function confirmLightResult(isLit: boolean) {
    if (!pendingLightConfirm) {
      showLightConfirmDialog = false;
      return;
    }

    const confirmed: TestResult = {
      ...pendingLightConfirm,
      passed: isLit,
      raw_response: `${pendingLightConfirm.raw_response}, LightOn=${isLit ? 1 : 0}`,
      checks: [
        {
          name: "light_confirmed",
          min: null,
          max: null,
          value: isLit ? 1 : 0,
          passed: isLit,
        },
      ],
    };

    upsertResult(confirmed);
    pendingLightConfirm = null;
    showLightConfirmDialog = false;

    if (!running) {
      recalcSummaryState();
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

    loadTestStages()
      .then((stages) => {
        availableStages = stages;
        if (
          selectedStage !== ALL_STAGES_VALUE &&
          !stages.includes(selectedStage)
        ) {
          selectedStage = ALL_STAGES_VALUE;
        }
      })
      .catch((err) => {
        console.error("Failed to load test stages", err);
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
    stopping = false;
    retesting = null;
    error = null;
    exportError = null;
    exportSuccess = null;
    results = [];
    pendingLightConfirm = null;
    showLightConfirmDialog = false;
    statusKey = "running";
    summaryState = "pending";

    try {
      const stagesToRun =
        selectedStage === ALL_STAGES_VALUE ? [...availableStages] : [selectedStage];
      const summary = await startTest(stagesToRun);
      statusKey = "done";

      for (const result of summary.results) {
        handleIncomingResult(result);
      }

      if (showLightConfirmDialog) {
        summaryState = "pending";
      } else {
        recalcSummaryState();
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      if (message.includes("测试已手动停止")) {
        error = message;
        statusKey = "idle";
        summaryState =
          results.length === 0
            ? "idle"
            : results.every((item) => item.passed)
              ? "pass"
              : "fail";
      } else {
        error = message;
        statusKey = "failed";
        summaryState = "fail";
      }
    } finally {
      running = false;
      stopping = false;
    }
  };

  const handleStop = async () => {
    if (!running || stopping) {
      return;
    }
    stopping = true;
    try {
      await stopTest();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
      stopping = false;
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
      if (updated.command === "ParamId606" && updated.passed) {
        pendingLightConfirm = updated;
        showLightConfirmDialog = true;
        summaryState = "pending";
      } else {
        upsertResult(updated);
        recalcSummaryState();
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
    } finally {
      retesting = null;
    }
  };

  const handleSelectStage = (stage: string) => {
    selectedStage = stage;
  };

  const handleOpenExport = () => {
    showExportDialog = true;
    exportError = null;
  };

  const handleConfirmExport = async (startDate: string, endDate: string) => {
    if (exporting) {
      return;
    }
    if (startDate > endDate) {
      throw new Error(text.exportInvalidRange);
    }

    exportError = null;
    exportSuccess = null;
    exporting = true;
    exportStartDate = startDate;
    exportEndDate = endDate;

    try {
      const defaultName = `test-results-${startDate}_to_${endDate}.csv`;
      const filePath = await save({
        title: text.exportDialogTitle,
        defaultPath: defaultName,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (!filePath) {
        return;
      }

      const count = await exportTestResultsCsv(
        startDate,
        endDate,
        String(filePath),
      );
      exportSuccess = `${text.exportSuccess} (${count} rows)`;
      showExportDialog = false;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      exportError = `${text.exportFailed}: ${message}`;
      throw new Error(exportError);
    } finally {
      exporting = false;
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

<div class="flex h-screen overflow-hidden">
  <Sidebar
    {view}
    {text}
    onOpenTests={handleOpenTests}
    onOpenSettings={handleOpenSettings}
  />

  <div class="flex-1 overflow-auto">
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
        {exportError}
        {exportSuccess}
        {exporting}
        {stopping}
        stageOptions={availableStages}
        {selectedStage}
        onStart={handleStart}
        onStop={handleStop}
        onOpenExport={handleOpenExport}
        onRetest={handleRetest}
        onSelectStage={handleSelectStage}
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

  <ConfirmDialog
    open={showLightConfirmDialog}
    title={text.confirmTitle}
    message={text.confirmLightQuestion}
    yesLabel={text.confirmYes}
    noLabel={text.confirmNo}
    onYes={() => confirmLightResult(true)}
    onNo={() => confirmLightResult(false)}
  />

  <ExportDialog
    open={showExportDialog}
    {exporting}
    {text}
    initialStartDate={exportStartDate}
    initialEndDate={exportEndDate}
    onClose={() => (showExportDialog = false)}
    onConfirm={handleConfirmExport}
  />
</div>
