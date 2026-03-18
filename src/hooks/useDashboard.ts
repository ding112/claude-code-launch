import { useState, useCallback } from "react";
import type {
  SessionItem,
  ConfigItem,
  DailyActivity,
} from "../types";
import { fetchSessions } from "../api";
import { fetchConfigs } from "../api";
import { fetchDashboardActivity } from "../api";

export type DashboardStats = {
  todaySessions: number;
  totalTokens: number;
  activeProjects: number;
  configHealth: { active: number; missing: number; total: number };
};

export type ConfigSummary = {
  active: number;
  missing: number;
  total: number;
  projectCount: number;
  bySource: { claude: { active: number; missing: number }; cursor: { active: number; missing: number } };
};

function startOfTodayMs(): number {
  const now = new Date();
  return new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
}

function computeStats(sessions: SessionItem[], configs: ConfigItem[]): DashboardStats {
  const todayStart = startOfTodayMs();
  const todaySessions = sessions.filter((s) => s.last_active_at_ms >= todayStart).length;
  const totalTokens = sessions.reduce((sum, s) => sum + s.input_tokens + s.output_tokens, 0);
  const activeProjects = new Set(sessions.map((s) => s.project_name).filter(Boolean)).size;

  const active = configs.filter((c) => c.status === "active").length;
  const missing = configs.filter((c) => c.status === "missing").length;

  return {
    todaySessions,
    totalTokens,
    activeProjects,
    configHealth: { active, missing, total: configs.length },
  };
}

function computeConfigSummary(configs: ConfigItem[], projectCount: number): ConfigSummary {
  const active = configs.filter((c) => c.status === "active").length;
  const missing = configs.filter((c) => c.status === "missing").length;

  const claudeConfigs = configs.filter((c) => c.source === "claude");
  const cursorConfigs = configs.filter((c) => c.source === "cursor");

  return {
    active,
    missing,
    total: configs.length,
    projectCount,
    bySource: {
      claude: {
        active: claudeConfigs.filter((c) => c.status === "active").length,
        missing: claudeConfigs.filter((c) => c.status === "missing").length,
      },
      cursor: {
        active: cursorConfigs.filter((c) => c.status === "active").length,
        missing: cursorConfigs.filter((c) => c.status === "missing").length,
      },
    },
  };
}

export function useDashboard() {
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [recentSessions, setRecentSessions] = useState<SessionItem[]>([]);
  const [dailyActivity, setDailyActivity] = useState<DailyActivity[]>([]);
  const [configSummary, setConfigSummary] = useState<ConfigSummary | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [sessions, configsResp, activityResp] = await Promise.all([
        fetchSessions(),
        fetchConfigs(),
        fetchDashboardActivity(30),
      ]);

      setStats(computeStats(sessions, configsResp.items));
      setRecentSessions(sessions.slice(0, 10));
      setDailyActivity(activityResp.daily);
      setConfigSummary(computeConfigSummary(configsResp.items, configsResp.project_count));
    } catch (err) {
      console.error("failed to load dashboard data:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    loading,
    stats,
    recentSessions,
    dailyActivity,
    configSummary,
    refresh: load,
  };
}
