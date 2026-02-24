import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import type { BaseConfig, TestResult, TestSummary } from "../types";

export const TAURI_EVENTS = {
  testGroupComplete: "test-group-complete",
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

export function startTest() {
  return invoke<TestSummary>("start_test");
}

export function retestGroup(groupName: string) {
  return invoke<TestResult>("retest_group", { groupName });
}

export async function subscribeTestGroupComplete(
  handler: (result: TestResult) => void,
) {
  const stop = await listen<TestResult>(TAURI_EVENTS.testGroupComplete, (event) => {
    handler(event.payload);
  });
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
