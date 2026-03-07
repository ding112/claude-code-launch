export type Severity = "blocker" | "warning" | "info";

export interface PrereqItem {
  name: string;
  available: boolean;
  severity: Severity;
  message: string;
}

export interface PrereqResult {
  platform: string;
  claudeInstalled: boolean;
  claudeVersion?: string | null;
  items: PrereqItem[];
}

export interface InstallAttempt {
  method: string;
  success: boolean;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  errorSummary?: string | null;
}

export interface InstallResult {
  status: "success" | "failed";
  selectedMethod?: string | null;
  summary: string;
  attempts: InstallAttempt[];
}

export interface VerifyResult {
  success: boolean;
  versionOutput: string;
  doctorOutput: string;
  errorSummary?: string | null;
}

export interface LogEvent {
  step: "prereq" | "install" | "verify" | string;
  level: "info" | "warn" | "error" | string;
  message: string;
  raw?: string | null;
  timestamp: number;
}
