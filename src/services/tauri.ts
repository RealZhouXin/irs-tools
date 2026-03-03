import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import type { BaseConfig, KeyStatePayload, TestResult, TestSummary } from "../types";

export const TAURI_EVENTS = {
  testGroupComplete: "test-group-complete",
  keyStateUpdate: "key-state-update",
} as const;

export function showMainWindow() {
  return invoke("show_main_window");
}

export function loadBaseConfig() {
  return invoke<BaseConfig>("get_base_config");
}

export function saveBaseConfig(config: BaseConfig) {
  return invoke<BaseConfig>("save_base_config", { config });
}

export function loadTestStages() {
  return invoke<string[]>("get_test_stages");
}

export function startTest(stages: string[]) {
  return invoke<TestSummary>("start_test", { stages });
}

export function stopTest() {
  return invoke("stop_test");
}

export function retestGroup(groupName: string) {
  return invoke<TestResult>("retest_group", { groupName });
}

export function exportTestResultsCsv(
  startDate: string,
  endDate: string,
  outputPath: string,
) {
  return invoke<number>("export_test_results_csv", {
    startDate,
    endDate,
    outputPath,
  });
}

export function loadAvailableExportDates() {
  return invoke<string[]>("get_available_export_dates");
}

export async function subscribeTestGroupComplete(
  handler: (result: TestResult) => void,
) {
  const stop = await listen<TestResult>(TAURI_EVENTS.testGroupComplete, (event) => {
    handler(event.payload);
  });
  return stop;
}

export async function subscribeKeyStateUpdate(
  handler: (payload: KeyStatePayload) => void,
) {
  const stop = await listen<KeyStatePayload>(
    TAURI_EVENTS.keyStateUpdate,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function loadAppInfo() {
  const [name, version, tauriVersion] = await Promise.all([
    getName(),
    getVersion(),
    getTauriVersion(),
  ]);
  return { name, version, tauriVersion };
}
