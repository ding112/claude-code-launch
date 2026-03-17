import { useEffect, useMemo, useState } from "react";
import type { SessionItem, DiscoverResult } from "../types";
import { fetchSessions, archiveSession, discoverSessions } from "../api";

export function useSessions() {
  const [sessions, setSessions] = useState<SessionItem[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [archiving, setArchiving] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [sessionMessage, setSessionMessage] = useState<string>("");
  const [collapsedProjects, setCollapsedProjects] = useState<Record<string, boolean>>({});

  const selectedSession = useMemo(
    () => sessions.find((s) => s.session_id === selectedSessionId),
    [sessions, selectedSessionId],
  );

  const groupedSessions = useMemo(() => {
    const groups: Record<string, SessionItem[]> = {};
    for (const session of sessions) {
      if (!groups[session.project_name]) {
        groups[session.project_name] = [];
      }
      groups[session.project_name].push(session);
    }
    return groups;
  }, [sessions]);

  useEffect(() => {
    setCollapsedProjects((prev) => {
      const next = { ...prev };
      let changed = false;
      for (const projectName of Object.keys(groupedSessions)) {
        if (!(projectName in next)) {
          next[projectName] = true;
          changed = true;
        }
      }
      return changed ? next : prev;
    });
  }, [groupedSessions]);

  const toggleProject = (projectName: string) => {
    setCollapsedProjects((prev) => ({
      ...prev,
      [projectName]: !(prev[projectName] ?? true),
    }));
  };

  const loadSessions = async (): Promise<SessionItem[]> => {
    setLoading(true);
    try {
      const data = await fetchSessions();
      setSessions(data);
      if (!selectedSessionId && data.length > 0) {
        setSelectedSessionId(data[0].session_id);
      }
      return data;
    } finally {
      setLoading(false);
    }
  };

  const archiveSelectedSession = async (
    clearEvents: () => void,
  ) => {
    if (!selectedSessionId) return;

    setArchiving(true);
    setSessionMessage("");
    const currentSessionId = selectedSessionId;
    try {
      const ok = await archiveSession(currentSessionId);
      if (!ok) {
        setSessionMessage("归档失败，请稍后重试。");
        return;
      }

      const updatedSessions = await loadSessions();
      const nextSessionId = updatedSessions[0]?.session_id ?? "";
      if (updatedSessions.some((item) => item.session_id === currentSessionId)) {
        setSelectedSessionId(currentSessionId);
      } else {
        setSelectedSessionId(nextSessionId);
        if (!nextSessionId) {
          clearEvents();
        }
      }
      setSessionMessage("已归档当前 session。");
    } finally {
      setArchiving(false);
    }
  };

  const runDiscover = async (): Promise<DiscoverResult | null> => {
    setDiscovering(true);
    setSessionMessage("");
    try {
      const result = await discoverSessions();
      await loadSessions();
      const parts: string[] = [];
      if (result.scanned > 0 || result.imported > 0 || result.updated > 0) {
        parts.push(
          `Claude Code: 扫描 ${result.scanned}，导入 ${result.imported}，更新 ${result.updated}${result.errors > 0 ? `，错误 ${result.errors}` : ""}`,
        );
      }
      if (result.cursor_scanned > 0 || result.cursor_imported > 0 || result.cursor_updated > 0) {
        parts.push(
          `Cursor: 扫描 ${result.cursor_scanned}，导入 ${result.cursor_imported}，更新 ${result.cursor_updated}${result.cursor_errors > 0 ? `，错误 ${result.cursor_errors}` : ""}`,
        );
      }
      setSessionMessage(
        parts.length > 0 ? parts.join("；") + "。" : "未发现新的历史 session。",
      );
      return result;
    } catch {
      setSessionMessage("发现历史 session 失败，请稍后重试。");
      return null;
    } finally {
      setDiscovering(false);
    }
  };

  return {
    sessions,
    selectedSessionId,
    setSelectedSessionId,
    selectedSession,
    groupedSessions,
    collapsedProjects,
    loading,
    archiving,
    discovering,
    sessionMessage,
    toggleProject,
    loadSessions,
    archiveSelectedSession,
    runDiscover,
  };
}
