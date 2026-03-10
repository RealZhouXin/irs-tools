export type AppUpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "up_to_date"
  | "downloading"
  | "installing"
  | "error";

export type AppUpdateInfo = {
  currentVersion: string;
  version: string;
  date: string | null;
  body: string | null;
};

export type AppUpdateProgress = {
  phase: "downloading" | "installing";
  downloaded: number;
  contentLength: number | null;
};
