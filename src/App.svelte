<script lang="ts">
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import type {
    BaseConfig,
    CollisionBarPromptPayload,
    EmergencyStopTestPayload,
    KeyStatePayload,
    LogLevel,
    Language,
    RearLightColor,
    RearLightConfirmRequestPayload,
    StatusKey,
    SummaryState,
    TestResult,
  } from "./types";
  import { getTranslation } from "./i18n/locales";
  import {
    cancelSensorPromptTest,
    cancelEmergencyStopTest,
    cancelKeyTest,
    confirmFrontLight,
    confirmRearLight,
    confirmSpeaker,
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
    subscribeFrontLightConfirmRequest,
    subscribeEmergencyStopTestUpdate,
    subscribeCollisionBarPromptRequest,
    subscribeRearLightConfirmRequest,
    subscribeSpeakerConfirmRequest,
    subscribeKeyStateUpdate,
    subscribeTestGroupComplete,
  } from "./services/tauri";
  import AppSidebar from "./components/Sidebar.svelte";
  import * as SidebarUI from "$lib/components/ui/sidebar/index.js";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import ExportDialog from "./components/ExportDialog.svelte";
  import EmergencyStopDialog from "./components/EmergencyStopDialog.svelte";
  import KeyTestDialog from "./components/KeyTestDialog.svelte";
  import InstructionDialog from "./components/InstructionDialog.svelte";
  import SpeakerTestDialog from "./components/SpeakerTestDialog.svelte";
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

  type LightConfirmTarget = "front" | "rear";

  const ALL_STAGES_VALUE = "__all__";

  // Svelte 5 Runes state management
  let results = $state<TestResult[]>([]);
  let language = $state<Language>("zh");
  let statusKey = $state<StatusKey>("idle");
  let summaryState = $state<SummaryState>("idle");
  let machineSn = $state<string | null>(null);
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
  let showLightConfirmDialog = $state(false);
  let lightConfirmTarget = $state<LightConfirmTarget>("front");
  let rearLightConfirmRequest = $state<RearLightConfirmRequestPayload | null>(null);
  let showKeyTestDialog = $state(false);
  let keyTestDialogDismissed = $state(false);
  let showSpeakerTestDialog = $state(false);
  let showCollisionBarDialog = $state(false);
  let collisionBarPromptPayload = $state<CollisionBarPromptPayload | null>(null);
  let showEmergencyStopDialog = $state(false);
  let emergencyStopPayload = $state<EmergencyStopTestPayload | null>(null);
  let keyState = $state<KeyStatePayload>({
    up_pressed: false,
    down_pressed: false,
    back_pressed: false,
    confirm_pressed: false,
  });
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
    if (incoming.command === "ParamId776") {
      showKeyTestDialog = false;
      keyTestDialogDismissed = false;
    }
    if (incoming.command === "ParamId080EmergencyStop") {
      showEmergencyStopDialog = false;
      emergencyStopPayload = null;
    }
    if (incoming.command === "ParamId568") {
      showSpeakerTestDialog = false;
    }
    if (incoming.command === "ParamId118") {
      showCollisionBarDialog = false;
      collisionBarPromptPayload = null;
    }
    if (incoming.command === "ParamId526") {
      machineSn = extractPcbSerNo(incoming.raw_response);
    }
    upsertResult(incoming);
  }

  function extractPcbSerNo(rawResponse: string): string | null {
    const match = rawResponse.match(/(?:^|,\s*)PcbSerNo=(\d+)/);
    return match?.[1] ?? null;
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

  async function confirmLightResult(isLit: boolean) {
    try {
      if (lightConfirmTarget === "rear") {
        await confirmRearLight(isLit);
      } else {
        await confirmFrontLight(isLit);
      }
      showLightConfirmDialog = false;
      if (lightConfirmTarget === "rear") {
        rearLightConfirmRequest = null;
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
    }
  }

  async function confirmSpeakerResult(heardSound: boolean) {
    try {
      await confirmSpeaker(heardSound);
      showSpeakerTestDialog = false;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      error = message;
    }
  }

  function getRearLightColorLabel(color: RearLightColor): string {
    if (language === "zh") {
      switch (color) {
        case "red":
          return "红色";
        case "green":
          return "绿色";
        case "blue":
          return "蓝色";
      }
    }
    return color;
  }

  function getLightConfirmMessage(): string {
    if (lightConfirmTarget !== "rear" || !rearLightConfirmRequest) {
      return text.confirmLightQuestion;
    }
    const colorLabel = getRearLightColorLabel(rearLightConfirmRequest.expected_color);
    const progress = `${rearLightConfirmRequest.step_index}/${rearLightConfirmRequest.total_steps}`;
    return language === "zh"
      ? `请确认当前尾灯为${colorLabel}（${progress}）`
      : `Please confirm the rear light is ${colorLabel} (${progress})`;
  }

  function getEmergencyStopInstruction(): string {
    if (!emergencyStopPayload) {
      return "";
    }
    if (emergencyStopPayload.phase === "unlock_by_back_and_confirm") {
      return language === "zh"
        ? "请按返回键 + 确认键解锁急停键"
        : "Please unlock using Back + Confirm keys";
    }
    return language === "zh" ? "请按下急停键" : "Please press the emergency stop key";
  }

  function getEmergencyStopStatus(): string {
    if (!emergencyStopPayload) {
      return "";
    }
    const elapsedSeconds = Math.floor(emergencyStopPayload.elapsed_ms / 1000);
    const timeoutSeconds = Math.floor(emergencyStopPayload.timeout_ms / 1000);
    return language === "zh"
      ? `MowerMainP=${emergencyStopPayload.mower_main_p}，已耗时 ${elapsedSeconds}s / 超时 ${timeoutSeconds}s`
      : `MowerMainP=${emergencyStopPayload.mower_main_p}, elapsed ${elapsedSeconds}s / timeout ${timeoutSeconds}s`;
  }

  async function handleEmergencyStopDialogClose() {
    showEmergencyStopDialog = false;
    emergencyStopPayload = null;
    try {
      await cancelEmergencyStopTest();
    } catch (err) {
      console.error("Failed to cancel emergency stop test", err);
    }
  }

  async function handleKeyTestDialogClose() {
    showKeyTestDialog = false;
    keyTestDialogDismissed = true;
    try {
      await cancelKeyTest();
    } catch (err) {
      console.error("Failed to cancel key test", err);
    }
  }

  async function handleSpeakerTestDialogClose() {
    await confirmSpeakerResult(false);
  }

  async function handleSensorPromptDialogClose() {
    showCollisionBarDialog = false;
    collisionBarPromptPayload = null;
    try {
      await cancelSensorPromptTest();
    } catch (err) {
      console.error("Failed to cancel sensor prompt test", err);
    }
  }

  function isLiftSensorPrompt(): boolean {
    return collisionBarPromptPayload?.prompt_kind === "lift_sensor";
  }

  function getSensorPromptTitle(): string {
    return isLiftSensorPrompt()
      ? text.liftSensorTestTitle
      : text.collisionBarTestTitle;
  }

  function getSensorPromptMessage(): string {
    const base = isLiftSensorPrompt()
      ? text.liftSensorTestInstruction
      : text.collisionBarTestInstruction;
    return collisionBarPromptPayload
      ? `${base} (${collisionBarPromptPayload.name})`
      : base;
  }

  onMount(() => {
    let unlisten: (() => void) | null = null;
    let unlistenKeyState: (() => void) | null = null;
    let unlistenEmergencyStopUpdate: (() => void) | null = null;
    let unlistenCollisionBarPrompt: (() => void) | null = null;
    let unlistenFrontLightConfirm: (() => void) | null = null;
    let unlistenRearLightConfirm: (() => void) | null = null;
    let unlistenSpeakerConfirm: (() => void) | null = null;

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

    subscribeKeyStateUpdate((payload) => {
      keyState = payload;
      if (!keyTestDialogDismissed) {
        showKeyTestDialog = true;
      }
    })
      .then((stop) => {
        unlistenKeyState = stop;
      })
      .catch((err) => {
        console.error(`Failed to listen ${TAURI_EVENTS.keyStateUpdate}`, err);
      });

    subscribeEmergencyStopTestUpdate((payload) => {
      emergencyStopPayload = payload;
      showEmergencyStopDialog = true;
      summaryState = "pending";
    })
      .then((stop) => {
        unlistenEmergencyStopUpdate = stop;
      })
      .catch((err) => {
        console.error(
          `Failed to listen ${TAURI_EVENTS.emergencyStopTestUpdate}`,
          err,
        );
      });

    subscribeCollisionBarPromptRequest((payload) => {
      collisionBarPromptPayload = payload;
      showCollisionBarDialog = true;
      summaryState = "pending";
    })
      .then((stop) => {
        unlistenCollisionBarPrompt = stop;
      })
      .catch((err) => {
        console.error(
          `Failed to listen ${TAURI_EVENTS.collisionBarPromptRequest}`,
          err,
        );
      });

    subscribeFrontLightConfirmRequest(() => {
      lightConfirmTarget = "front";
      rearLightConfirmRequest = null;
      showLightConfirmDialog = true;
      summaryState = "pending";
    })
      .then((stop) => {
        unlistenFrontLightConfirm = stop;
      })
      .catch((err) => {
        console.error(
          `Failed to listen ${TAURI_EVENTS.frontLightConfirmRequest}`,
          err,
        );
      });

    subscribeRearLightConfirmRequest((payload) => {
      lightConfirmTarget = "rear";
      rearLightConfirmRequest = payload;
      showLightConfirmDialog = true;
      summaryState = "pending";
    })
      .then((stop) => {
        unlistenRearLightConfirm = stop;
      })
      .catch((err) => {
        console.error(
          `Failed to listen ${TAURI_EVENTS.rearLightConfirmRequest}`,
          err,
        );
      });

    subscribeSpeakerConfirmRequest(() => {
      showSpeakerTestDialog = true;
      summaryState = "pending";
    })
      .then((stop) => {
        unlistenSpeakerConfirm = stop;
      })
      .catch((err) => {
        console.error(
          `Failed to listen ${TAURI_EVENTS.speakerConfirmRequest}`,
          err,
        );
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
      if (unlistenKeyState) {
        unlistenKeyState();
      }
      if (unlistenEmergencyStopUpdate) {
        unlistenEmergencyStopUpdate();
      }
      if (unlistenCollisionBarPrompt) {
        unlistenCollisionBarPrompt();
      }
      if (unlistenFrontLightConfirm) {
        unlistenFrontLightConfirm();
      }
      if (unlistenRearLightConfirm) {
        unlistenRearLightConfirm();
      }
      if (unlistenSpeakerConfirm) {
        unlistenSpeakerConfirm();
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
    showLightConfirmDialog = false;
    lightConfirmTarget = "front";
    rearLightConfirmRequest = null;
    showEmergencyStopDialog = false;
    emergencyStopPayload = null;
    showSpeakerTestDialog = false;
    showCollisionBarDialog = false;
    collisionBarPromptPayload = null;
    showKeyTestDialog = false;
    keyTestDialogDismissed = false;
    keyState = { up_pressed: false, down_pressed: false, back_pressed: false, confirm_pressed: false };
    statusKey = "running";
    summaryState = "pending";
    machineSn = null;

    try {
      const stagesToRun =
        selectedStage === ALL_STAGES_VALUE ? [...availableStages] : [selectedStage];
      const summary = await startTest(stagesToRun);
      statusKey = "done";

      for (const result of summary.results) {
        handleIncomingResult(result);
      }
      recalcSummaryState();
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
      handleIncomingResult(updated);
      recalcSummaryState();
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

<SidebarUI.Provider open={false}>
  <AppSidebar
    {view}
    {text}
    onOpenTests={handleOpenTests}
    onOpenSettings={handleOpenSettings}
  />

  <SidebarUI.Inset class="overflow-auto">
    {#if view === "main"}
      <MainView
        {text}
        {results}
        {error}
        {running}
        {statusKey}
        {summaryState}
        {summaryLabel}
        {machineSn}
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
  </SidebarUI.Inset>

  <ConfirmDialog
    open={showLightConfirmDialog}
    title={text.confirmTitle}
    message={getLightConfirmMessage()}
    yesLabel={text.confirmYes}
    noLabel={text.confirmNo}
    showLightAnimation={lightConfirmTarget === "rear"}
    lightColor={
      lightConfirmTarget === "rear" ? rearLightConfirmRequest?.expected_color ?? null : null
    }
    onYes={() => confirmLightResult(true)}
    onNo={() => confirmLightResult(false)}
    onRequestClose={() => confirmLightResult(false)}
  />

  <KeyTestDialog
    open={showKeyTestDialog}
    {text}
    {keyState}
    onRequestClose={handleKeyTestDialogClose}
  />

  <SpeakerTestDialog
    open={showSpeakerTestDialog}
    title={text.speakerTestTitle}
    message={text.speakerTestQuestion}
    yesLabel={text.confirmYes}
    noLabel={text.confirmNo}
    onYes={() => confirmSpeakerResult(true)}
    onNo={() => confirmSpeakerResult(false)}
    onRequestClose={handleSpeakerTestDialogClose}
  />

  <InstructionDialog
    open={showCollisionBarDialog}
    title={getSensorPromptTitle()}
    message={getSensorPromptMessage()}
    onRequestClose={handleSensorPromptDialogClose}
  />

  <EmergencyStopDialog
    open={showEmergencyStopDialog}
    title={language === "zh" ? "急停键测试" : "Emergency Stop Test"}
    instruction={getEmergencyStopInstruction()}
    status={getEmergencyStopStatus()}
    showUnlockKeys={emergencyStopPayload?.phase === "unlock_by_back_and_confirm"}
    backLabel={text.keyTestBack}
    confirmLabel={text.keyTestConfirm}
    onRequestClose={handleEmergencyStopDialogClose}
  />

  <ExportDialog
    open={showExportDialog}
    {exporting}
    {text}
    {language}
    initialStartDate={exportStartDate}
    initialEndDate={exportEndDate}
    onClose={() => (showExportDialog = false)}
    onConfirm={handleConfirmExport}
  />
</SidebarUI.Provider>
