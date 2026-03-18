import { RISK_STYLES } from "./constants";

export function riskClass(level: string): string {
  return RISK_STYLES[level.toLowerCase()] ?? RISK_STYLES.none;
}

export function formatTimestamp(value: number): string {
  return new Date(value).toLocaleString();
}

export function truncate(str: string, max: number): string {
  return str.length > max ? `${str.slice(0, max)}…` : str;
}

export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

export function formatDateShort(dateStr: string): string {
  const d = new Date(dateStr + "T00:00:00");
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

export function tokenDisplay(n: number): string {
  return n > 0 ? formatTokens(n) : "—";
}
