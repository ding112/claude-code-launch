import { useState, useEffect } from "react";
import type { ConfigContentResponse } from "../types";
import { fetchConfigContent } from "../api";

export function useConfigContent(configId: string | null) {
  const [content, setContent] = useState<ConfigContentResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!configId) {
      setContent(null);
      setError(null);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    fetchConfigContent(configId)
      .then((resp) => {
        if (!cancelled) {
          setContent(resp);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
          setContent(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [configId]);

  return { content, loading, error };
}
