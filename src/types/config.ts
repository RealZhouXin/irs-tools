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

export type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

export type BaseConfig = {
  connection: ConnectionConfig;
  read_timeout_ms: number;
  log_level: LogLevel;
};

export type TestsConfigUpdateStatus = {
  activePath: string;
  newDefaultAvailable: boolean;
  localIsModified: boolean;
  pendingDefaultPath: string | null;
  pendingDefaultVersion: string | null;
  ignoredPendingDefault: boolean;
};

export type ApplyTestsConfigUpdateResult = {
  backupPath: string;
  status: TestsConfigUpdateStatus;
};
