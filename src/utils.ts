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
