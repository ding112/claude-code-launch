import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PrereqResult } from "../types";

export function usePrereqs() {
  const [data, setData] = useState<PrereqResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<PrereqResult>("check_prereqs");
      setData(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : "环境检测失败";
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { data, loading, error, run };
}
