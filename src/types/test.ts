export type CheckResult = {
  name: string;
  min: number | null;
  max: number | null;
  value: number | null;
  passed: boolean;
};

export type TestResult = {
  name: string;
  stage: string;
  command: string;
  raw_response: string;
  passed: boolean;
  checks: CheckResult[];
};

export type TestSummary = {
  results: TestResult[];
  overall_passed: boolean;
};

export type KeyStatePayload = {
  up_pressed: boolean;
  down_pressed: boolean;
  back_pressed: boolean;
  confirm_pressed: boolean;
};

export type FrontLightConfirmRequestPayload = {
  name: string;
  stage: string;
  front_light_mode: number;
  power: number;
};

export type RearLightColor = "red" | "green" | "blue";

export type RearLightConfirmRequestPayload = {
  name: string;
  stage: string;
  rear_light_mode: number;
  expected_color: RearLightColor;
  step_index: number;
  total_steps: number;
};

export type SpeakerConfirmRequestPayload = {
  name: string;
  stage: string;
  on: number;
};

export type EmergencyStopPhase =
  | "press_emergency_stop"
  | "unlock_by_back_and_confirm";

export type EmergencyStopTestPayload = {
  name: string;
  stage: string;
  phase: EmergencyStopPhase;
  mower_main_p: number;
  elapsed_ms: number;
  timeout_ms: number;
};
