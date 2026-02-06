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
    langLabel: string;
};
