<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
  import type {
    BaseConfig,
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
  let view = $state<"main" | "settings">("main");

  type SettingsDraft = {
    mode: "network" | "serial";
    ip_address: string;
    port: string;
    port_number: number;
    read_timeout_ms: number;
  };

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

  onMount(() => {
    let unlisten: (() => void) | null = null;

    invoke("show_main_window").catch((err) => {
      console.error("Failed to show main window", err);
    });

    invoke<BaseConfig>("get_base_config")
      .then((config) => {
        if (config.connection.mode === "network") {
          settingsDraft = {
            mode: "network",
            ip_address: config.connection.ip_address,
            port: config.connection.port,
            port_number: 1,
            read_timeout_ms: config.read_timeout_ms,
          };
        } else {
          settingsDraft = {
            mode: "serial",
            ip_address: "",
            port: "",
            port_number: config.connection.port_number,
            read_timeout_ms: config.read_timeout_ms,
          };
        }
      })
      .catch((err) => {
        console.error("Failed to load config", err);
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

    Promise.all([getName(), getVersion(), getTauriVersion()])
      .then(([name, version, tauri]) => {
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
          }
        : {
            connection: {
              mode: "serial",
              port_number: settingsDraft.port_number,
            },
            read_timeout_ms: settingsDraft.read_timeout_ms,
          };

    try {
      const saved = await invoke<BaseConfig>("save_base_config", { config });
      if (saved.connection.mode === "network") {
        settingsDraft = {
          mode: "network",
          ip_address: saved.connection.ip_address,
          port: saved.connection.port,
          port_number: settingsDraft.port_number,
          read_timeout_ms: saved.read_timeout_ms,
        };
      } else {
        settingsDraft = {
          mode: "serial",
          ip_address: settingsDraft.ip_address,
          port: settingsDraft.port,
          port_number: saved.connection.port_number,
          read_timeout_ms: saved.read_timeout_ms,
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
  <aside class="sidebar">
    <div class="nav-top">
      <button
        class="nav-btn"
        data-active={view === "main"}
        onclick={handleOpenTests}
        title={text.navTests}
        aria-label={text.navTests}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M7 6h10M7 12h10M7 18h10"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
    <div class="nav-bottom">
      <button
        class="nav-btn"
        data-active={view === "settings"}
        onclick={handleOpenSettings}
        title={text.navSettings}
        aria-label={text.navSettings}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm7.5-3.5c0-.46-.04-.92-.12-1.35l2.02-1.57-1.9-3.3-2.4.97a7.8 7.8 0 0 0-2.34-1.35l-.37-2.55h-3.8l-.37 2.55a7.8 7.8 0 0 0-2.34 1.35l-2.4-.97-1.9 3.3 2.02 1.57c-.08.43-.12.89-.12 1.35s.04.92.12 1.35l-2.02 1.57 1.9 3.3 2.4-.97c.72.57 1.51 1.04 2.34 1.35l.37 2.55h3.8l.37-2.55c.83-.31 1.62-.78 2.34-1.35l2.4.97 1.9-3.3-2.02-1.57c.08-.43.12-.89.12-1.35Z"
            fill="currentColor"
          />
        </svg>
      </button>
    </div>
  </aside>

  <div class="content">
    {#if view === "main"}
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
    {:else}
      <section class="settings">
        <header class="settings-header">
          <div>
            <h1>{text.settingsTitle}</h1>
            <p class="subtitle">{text.settingsSubtitle}</p>
          </div>
          <div class="header-actions">
            <button class="lang-toggle" onclick={toggleLanguage}>
              {text.langLabel}
            </button>
          </div>
        </header>

        <div class="settings-card">
          {#if settingsDraft === null}
            <p class="settings-message">{text.settingsLoading}</p>
          {:else}
            <div class="field">
              <span class="field-label">{text.fieldMode}</span>
              <div class="segmented" role="group" aria-label={text.fieldMode}>
                <button
                  class="segment"
                  data-active={settingsDraft.mode === "network"}
                  onclick={() => {
                    if (settingsDraft) settingsDraft.mode = "network";
                  }}
                >
                  {text.fieldNetwork}
                </button>
                <button
                  class="segment"
                  data-active={settingsDraft.mode === "serial"}
                  onclick={() => {
                    if (settingsDraft) settingsDraft.mode = "serial";
                  }}
                >
                  {text.fieldSerial}
                </button>
              </div>
            </div>

            {#if settingsDraft.mode === "network"}
              <div class="field-grid">
                <div class="field">
                  <label for="field-ip">{text.fieldIp}</label>
                  <input
                    id="field-ip"
                    type="text"
                    value={settingsDraft.ip_address}
                    oninput={(event) => {
                      if (settingsDraft)
                        settingsDraft.ip_address = event.currentTarget.value;
                    }}
                  />
                </div>
                <div class="field">
                  <label for="field-port">{text.fieldPort}</label>
                  <input
                    id="field-port"
                    type="text"
                    value={settingsDraft.port}
                    oninput={(event) => {
                      if (settingsDraft)
                        settingsDraft.port = event.currentTarget.value;
                    }}
                  />
                </div>
              </div>
            {:else}
              <div class="field">
                <label for="field-serial-port">{text.fieldSerialPort}</label>
                <input
                  id="field-serial-port"
                  type="number"
                  min="1"
                  value={settingsDraft.port_number}
                  oninput={(event) => {
                    if (!settingsDraft) return;
                    const value = Number(event.currentTarget.value);
                    settingsDraft.port_number = Number.isNaN(value) ? 1 : value;
                  }}
                />
              </div>
            {/if}

            <div class="field">
              <label for="field-timeout">{text.fieldTimeout}</label>
              <input
                id="field-timeout"
                type="number"
                min="0"
                value={settingsDraft.read_timeout_ms}
                oninput={(event) => {
                  if (!settingsDraft) return;
                  const value = Number(event.currentTarget.value);
                  settingsDraft.read_timeout_ms = Number.isNaN(value)
                    ? 0
                    : value;
                }}
              />
            </div>

            <div class="settings-actions">
              <button
                class="primary"
                onclick={handleSettingsSave}
                disabled={settingsSaving}
              >
                {settingsSaving ? text.settingsSaving : text.settingsSave}
              </button>
              {#if settingsSaved}
                <span class="settings-success">{text.settingsSaved}</span>
              {/if}
              {#if settingsError}
                <span class="settings-error">
                  {text.settingsError}: {settingsError}
                </span>
              {/if}
            </div>
          {/if}
        </div>

        <div class="about-card">
          <div>
            <h2>{text.aboutTitle}</h2>
            <p class="subtitle">{text.aboutSubtitle}</p>
          </div>
          {#if aboutError}
            <p class="settings-error">
              {text.aboutError}: {aboutError}
            </p>
          {:else if !appVersion}
            <p class="settings-message">{text.aboutLoading}</p>
          {:else}
            <div class="about-grid">
              <div class="about-item">
                <span class="about-label">{text.aboutName}</span>
                <span class="about-value">{appName ?? "-"}</span>
              </div>
              <div class="about-item">
                <span class="about-label">{text.aboutVersion}</span>
                <span class="about-value">{appVersion}</span>
              </div>
              <div class="about-item">
                <span class="about-label">{text.aboutTauriVersion}</span>
                <span class="about-value">{tauriVersion ?? "-"}</span>
              </div>
            </div>
          {/if}
        </div>
      </section>
    {/if}
  </div>
</div>
