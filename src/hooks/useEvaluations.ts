import { useState, useCallback } from "react";
import type { EvaluationResponse } from "../types";
import { fetchEvaluations } from "../api";

export function useEvaluations() {
  const [evaluations, setEvaluations] = useState<EvaluationResponse>({
    items: [],
    total: 0,
    page: 1,
    page_size: 20,
  });

  const loadEvaluations = useCallback(async (sessionId: string, page: number, pageSize: number) => {
    const data = await fetchEvaluations(sessionId, page, pageSize);
    setEvaluations(data);
  }, []);

  const gotoEvaluationPage = (sessionId: string, nextPage: number) => {
    if (!sessionId) return;
    void loadEvaluations(sessionId, nextPage, evaluations.page_size);
  };

  const clearEvaluations = () => {
    setEvaluations((prev) => ({ ...prev, items: [], total: 0, page: 1 }));
  };

  return {
    evaluations,
    loadEvaluations,
    gotoEvaluationPage,
    clearEvaluations,
  };
}
