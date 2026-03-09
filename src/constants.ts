export const RISK_STYLES: Record<string, string> = {
  high: "bg-risk-high-bg text-risk-high-text border-risk-high-border",
  medium: "bg-risk-medium-bg text-risk-medium-text border-risk-medium-border",
  low: "bg-risk-low-bg text-risk-low-text border-risk-low-border",
  none: "bg-risk-none-bg text-risk-none-text border-risk-none-border",
};

export const KNOWN_EVENTS = [
  "PermissionRequest",
  "Notification",
  "Stop",
  "SubagentStop",
  "SessionStart",
  "SessionEnd",
  "PreToolUse",
  "PostToolUse",
  "UserPromptSubmit",
] as const;

export const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:8787";
export const TRANSCRIPT_PAGE_SIZE = 200;
