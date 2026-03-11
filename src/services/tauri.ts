import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getName, getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { check } from "@tauri-apps/plugin-updater";
import type {
  AppUpdateInfo,
  AppUpdateProgress,
  BaseConfig,
  CollisionBarPromptPayload,
  DeviceSnPayload,
  EmergencyStopTestPayload,
  FrontLightConfirmRequestPayload,
  KeyStatePayload,
  RearLightConfirmRequestPayload,
  SpeakerConfirmRequestPayload,
  TestResult,
  TestSummary,
  WheelMotorTestUpdatePayload,
} from "../types";

let pendingAppUpdate: Awaited<ReturnType<typeof check>> = null;

export const TAURI_EVENTS = {
  testGroupComplete: "test-group-complete",
  deviceSnUpdate: "device-sn-update",
  keyStateUpdate: "key-state-update",
  frontLightConfirmRequest: "front-light-confirm-request",
  rearLightConfirmRequest: "rear-light-confirm-request",
  speakerConfirmRequest: "speaker-confirm-request",
  emergencyStopTestUpdate: "emergency-stop-test-update",
  collisionBarPromptRequest: "collision-bar-prompt-request",
  wheelMotorTestUpdate: "wheel-motor-test-update",
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

export function confirmFrontLight(isLit: boolean) {
  return invoke("confirm_front_light", { isLit });
}

export function confirmRearLight(isLit: boolean) {
  return invoke("confirm_rear_light", { isLit });
}

export function confirmSpeaker(heardSound: boolean) {
  return invoke("confirm_speaker", { heardSound });
}

export function cancelEmergencyStopTest() {
  return invoke("cancel_emergency_stop_test");
}

export function cancelKeyTest() {
  return invoke("cancel_key_test");
}

export function cancelSensorPromptTest() {
  return invoke("cancel_sensor_prompt_test");
}

export function confirmWheelMotorLifted(isLifted: boolean) {
  return invoke("confirm_wheel_motor_lifted", { isLifted });
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

export async function subscribeDeviceSnUpdate(
  handler: (payload: DeviceSnPayload) => void,
) {
  const stop = await listen<DeviceSnPayload>(TAURI_EVENTS.deviceSnUpdate, (event) => {
    handler(event.payload);
  });
  return stop;
}

export async function subscribeFrontLightConfirmRequest(
  handler: (payload: FrontLightConfirmRequestPayload) => void,
) {
  const stop = await listen<FrontLightConfirmRequestPayload>(
    TAURI_EVENTS.frontLightConfirmRequest,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function subscribeRearLightConfirmRequest(
  handler: (payload: RearLightConfirmRequestPayload) => void,
) {
  const stop = await listen<RearLightConfirmRequestPayload>(
    TAURI_EVENTS.rearLightConfirmRequest,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function subscribeSpeakerConfirmRequest(
  handler: (payload: SpeakerConfirmRequestPayload) => void,
) {
  const stop = await listen<SpeakerConfirmRequestPayload>(
    TAURI_EVENTS.speakerConfirmRequest,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function subscribeEmergencyStopTestUpdate(
  handler: (payload: EmergencyStopTestPayload) => void,
) {
  const stop = await listen<EmergencyStopTestPayload>(
    TAURI_EVENTS.emergencyStopTestUpdate,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function subscribeCollisionBarPromptRequest(
  handler: (payload: CollisionBarPromptPayload) => void,
) {
  const stop = await listen<CollisionBarPromptPayload>(
    TAURI_EVENTS.collisionBarPromptRequest,
    (event) => {
      handler(event.payload);
    },
  );
  return stop;
}

export async function subscribeWheelMotorTestUpdate(
  handler: (payload: WheelMotorTestUpdatePayload) => void,
) {
  const stop = await listen<WheelMotorTestUpdatePayload>(
    TAURI_EVENTS.wheelMotorTestUpdate,
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

export async function checkForAppUpdate() {
  const [update, currentVersion] = await Promise.all([check(), getVersion()]);
  pendingAppUpdate = update;
  if (!update) {
    return null;
  }

  return {
    currentVersion,
    version: update.version,
    date:
      typeof update.date === "string"
        ? update.date
        : update.date?.toISOString?.() ?? null,
    body: update.body ?? null,
  } satisfies AppUpdateInfo;
}

export async function downloadAndInstallAppUpdate(
  onProgress?: (progress: AppUpdateProgress) => void,
) {
  if (!pendingAppUpdate) {
    throw new Error("No pending update. Please check for updates first.");
  }

  let downloaded = 0;
  let contentLength: number | null = null;

  await pendingAppUpdate.download((event) => {
    switch (event.event) {
      case "Started":
        downloaded = 0;
        contentLength = event.data.contentLength ?? null;
        onProgress?.({
          phase: "downloading",
          downloaded,
          contentLength,
        });
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        onProgress?.({
          phase: "downloading",
          downloaded,
          contentLength,
        });
        break;
      case "Finished":
        onProgress?.({
          phase: "installing",
          downloaded,
          contentLength,
        });
        break;
    }
  });

  try {
    await pendingAppUpdate.install();
  } finally {
    pendingAppUpdate = null;
  }
}
