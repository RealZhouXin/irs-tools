export type CheckResult = {
    name: string;
    min: number | null;
    max: number | null;
    value: number | null;
    passed: boolean;
};

export type TestResult = {
    name: string;
    command: string;
    raw_response: string;
    passed: boolean;
    checks: CheckResult[];
};

export type TestSummary = {
    results: TestResult[];
    overall_passed: boolean;
};

export type Language = "zh" | "en";

export type StatusKey = "idle" | "running" | "done" | "failed";

export type SummaryState = "pass" | "fail" | "pending" | "idle";

export type ConnectionConfig =
    | {
          mode: "serial";
          port_number: number;
      }
    | {
          mode: "network";
          ip_address: string;
          port: string;
      };

export type BaseConfig = {
    connection: ConnectionConfig;
    read_timeout_ms: number;
};

export type Translation = {
    title: string;
    subtitle: string;
    start: string;
    statusTitle: string;
    status: {
        idle: string;
        running: string;
        done: string;
        failed: string;
    };
    summary: {
        pass: string;
        fail: string;
        pending: string;
        idle: string;
    };
    table: {
        group: string;
        command: string;
        range: string;
        value: string;
        result: string;
        empty: string;
    };
    pass: string;
    fail: string;
    retest: string;
    retesting: string;
    configTitle: string;
    configPrefix: string;
    configMiddle: string;
    configSuffix: string;
    navTests: string;
    navSettings: string;
    settingsTitle: string;
    settingsSubtitle: string;
    settingsSave: string;
    settingsSaving: string;
    settingsSaved: string;
    settingsError: string;
    settingsLoading: string;
    fieldMode: string;
    fieldNetwork: string;
    fieldSerial: string;
    fieldIp: string;
    fieldPort: string;
    fieldSerialPort: string;
    fieldTimeout: string;
    langLabel: string;
};
