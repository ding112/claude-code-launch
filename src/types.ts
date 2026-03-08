// ── Setup (wizard) types ──

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

// ── Monitoring (overseer) types ──

export type SessionItem = {
  session_id: string;
  project_name: string;
  agent_type: string;
  last_active_at_ms: number;
  latest_risk_level: string;
  evaluation_count: number;
};

export type EventItem = {
  event_id: string;
  session_id: string;
  event_type: string;
  payload: Record<string, unknown>;
  created_at_ms: number;
};

export type EventResponse = {
  items: EventItem[];
  total: number;
  page: number;
  page_size: number;
};

export type EvaluationItem = {
  evaluation_id: string;
  event_id?: string;
  risk_level: string;
  risk_category: string;
  efficiency_level: string;
  suggestion: string;
  status: string;
  error_message?: string;
  retry_count: number;
  created_at_ms: number;
};

export type EvaluationResponse = {
  items: EvaluationItem[];
  total: number;
  page: number;
  page_size: number;
};

export type TranscriptLineItem = {
  line_no: number;
  line_content: string;
};

export type TranscriptResponse = {
  session_id: string;
  items: TranscriptLineItem[];
  has_more: boolean;
  next_before_line_no?: number;
  updated_at_ms: number;
  imported_offset_bytes: number;
  last_error_message?: string;
  last_error_stack?: string;
};

export type EvalSettings = {
  enabled: boolean;
  sampling_rate: number;
  provider: string;
  model: string;
  base_url: string;
  api_key?: string;
  timeout_ms: number;
};

export type HookItem = {
  type: string;
  command: string;
  timeout?: number;
};

export type HookBlock = {
  matcher: string;
  hooks: HookItem[];
};

export type HooksData = {
  events: Record<string, HookBlock[]>;
};

export type HooksInitResult = {
  events: Record<string, HookBlock[]>;
  added_count: number;
};
