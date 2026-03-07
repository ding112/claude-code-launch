import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { VerifyResult } from "../types";

export function useVerify() {
  const [data, setData] = useState<VerifyResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<VerifyResult>("run_verify");
      setData(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : "验证执行失败";
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return { data, loading, error, run };
}
