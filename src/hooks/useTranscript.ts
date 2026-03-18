import { useEffect, useRef, useState, type UIEvent } from "react";
import type { TranscriptResponse } from "../types";
import { fetchTranscript } from "../api";

export function useTranscript(selectedSessionId: string) {
  const [transcript, setTranscript] = useState<TranscriptResponse | null>(null);
  const [transcriptLoading, setTranscriptLoading] = useState(false);
  const [transcriptLoadingMore, setTranscriptLoadingMore] = useState(false);
  const [transcriptError, setTranscriptError] = useState<string>("");
  const transcriptAbortRef = useRef<AbortController | null>(null);
  const transcriptRequestSeqRef = useRef(0);

  const loadTranscript = async (sessionId: string) => {
    transcriptAbortRef.current?.abort();
    const controller = new AbortController();
    transcriptAbortRef.current = controller;
    const requestSeq = ++transcriptRequestSeqRef.current;

    setTranscriptLoading(true);
    setTranscriptLoadingMore(false);
    setTranscriptError("");
    try {
      const response = await fetchTranscript(sessionId, undefined, controller.signal);
      if (requestSeq !== transcriptRequestSeqRef.current) return;
      if (!response.ok) {
        setTranscript(null);
        setTranscriptError("Transcript 加载失败，请稍后重试。");
        return;
      }
      const data = (await response.json()) as TranscriptResponse;
      if (requestSeq !== transcriptRequestSeqRef.current) return;
      setTranscript({
        ...data,
        items: [...data.items].reverse(),
      });
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") return;
      if (requestSeq !== transcriptRequestSeqRef.current) return;
      setTranscript(null);
      setTranscriptError("Transcript 加载失败，请检查网络或服务状态。");
    } finally {
      if (requestSeq === transcriptRequestSeqRef.current) {
        setTranscriptLoading(false);
      }
    }
  };

  const loadOlderTranscriptLines = async () => {
    if (!selectedSessionId || !transcript?.has_more || !transcript?.next_before_line_no) return;
    if (transcriptLoading || transcriptLoadingMore) return;

    setTranscriptLoadingMore(true);
    try {
      const response = await fetchTranscript(selectedSessionId, transcript.next_before_line_no);
      if (!response.ok) {
        setTranscriptError("Transcript 加载更多失败，请稍后重试。");
        return;
      }
      const data = (await response.json()) as TranscriptResponse;
      setTranscript((prev) => {
        if (!prev) {
          return { ...data, items: [...data.items].reverse() };
        }
        return {
          ...prev,
          items: [...prev.items, ...[...data.items].reverse()],
          has_more: data.has_more,
          next_before_line_no: data.next_before_line_no,
          updated_at_ms: data.updated_at_ms,
          imported_offset_bytes: data.imported_offset_bytes,
          last_error_message: data.last_error_message,
          last_error_stack: data.last_error_stack,
        };
      });
    } catch {
      setTranscriptError("Transcript 加载更多失败，请检查网络或服务状态。");
    } finally {
      setTranscriptLoadingMore(false);
    }
  };

  const handleTranscriptScroll = (event: UIEvent<HTMLDivElement>) => {
    const container = event.currentTarget;
    if (container.scrollTop + container.clientHeight >= container.scrollHeight - 80) {
      void loadOlderTranscriptLines();
    }
  };

  useEffect(() => {
    if (!selectedSessionId) {
      transcriptAbortRef.current?.abort();
      setTranscript(null);
      setTranscriptError("");
      setTranscriptLoading(false);
      setTranscriptLoadingMore(false);
      return;
    }
    void loadTranscript(selectedSessionId);
  }, [selectedSessionId]);

  useEffect(
    () => () => {
      transcriptAbortRef.current?.abort();
    },
    [],
  );

  const skippedLines = transcript?.skipped_lines ?? 0;

  return {
    transcript,
    transcriptLoading,
    transcriptLoadingMore,
    transcriptError,
    skippedLines,
    handleTranscriptScroll,
  };
}
