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
