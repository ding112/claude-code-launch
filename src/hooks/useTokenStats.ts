import { useState, useCallback } from "react";
import { fetchTokenStats } from "../api";
import type { TokenStatsResponse } from "../types";

export type TimeRange = "7d" | "30d" | "90d" | "all";

function timeRangeToMs(range: TimeRange): { fromMs?: number } {
  if (range === "all") return {};
  const days = range === "7d" ? 7 : range === "30d" ? 30 : 90;
  const fromMs = Date.now() - days * 24 * 60 * 60 * 1000;
  return { fromMs };
}

export function useTokenStats() {
  const [data, setData] = useState<TokenStatsResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [timeRange, setTimeRange] = useState<TimeRange>("30d");
  const [sourceFilter, setSourceFilter] = useState<string | undefined>();

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const { fromMs } = timeRangeToMs(timeRange);
      const result = await fetchTokenStats({ fromMs, source: sourceFilter });
      setData(result);
    } catch (err) {
      console.error("Failed to fetch token stats:", err);
    } finally {
      setLoading(false);
    }
  }, [timeRange, sourceFilter]);

  return {
    data,
    loading,
    timeRange,
    setTimeRange,
    sourceFilter,
    setSourceFilter,
    refresh,
  };
}
