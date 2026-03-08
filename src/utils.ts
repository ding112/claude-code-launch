import { RISK_STYLES } from "./constants";

export function riskClass(level: string): string {
  return RISK_STYLES[level.toLowerCase()] ?? RISK_STYLES.none;
}

export function formatTimestamp(value: number): string {
  return new Date(value).toLocaleString();
}
