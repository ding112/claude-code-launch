import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { InstallResult } from "../types";

export function useInstall() {
  const [data, setData] = useState<InstallResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<InstallResult>("run_install");
      setData(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : "安装执行失败";
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { data, loading, error, run };
}
