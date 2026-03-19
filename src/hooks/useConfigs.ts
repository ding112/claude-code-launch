import { useState, useCallback } from "react";
import type { ConfigItem } from "../types";
import { fetchConfigs } from "../api";

export type SourceFilter = "all" | "claude" | "cursor";

export type ConfigGroup = {
  label: string;
  items: ConfigItem[];
};

export function useConfigs() {
  const [configs, setConfigs] = useState<ConfigItem[]>([]);
  const [projectCount, setProjectCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [sourceFilter, setSourceFilter] = useState<SourceFilter>("all");
  const [selectedConfigId, setSelectedConfigId] = useState<string | null>(null);

  const loadConfigs = useCallback(async (filter?: SourceFilter) => {
    const f = filter ?? sourceFilter;
    setLoading(true);
    try {
      const source = f === "all" ? undefined : f;
      const resp = await fetchConfigs(source);
      setConfigs(resp.items);
      setProjectCount(resp.project_count);
    } catch (e) {
      console.error("failed to load configs:", e);
    } finally {
      setLoading(false);
    }
  }, [sourceFilter]);

  const changeFilter = useCallback((f: SourceFilter) => {
    setSourceFilter(f);
    void loadConfigs(f);
  }, [loadConfigs]);

  const groupedConfigs = groupConfigs(configs);

  return {
    configs,
    projectCount,
    loading,
    sourceFilter,
    changeFilter,
    selectedConfigId,
    setSelectedConfigId,
    loadConfigs,
    groupedConfigs,
  };
}

function groupConfigs(items: ConfigItem[]): ConfigGroup[] {
  const groups: ConfigGroup[] = [];

  const globalClaude = items.filter(i => i.scope === "global" && i.source === "claude");
  const globalCursor = items.filter(i => i.scope === "global" && i.source === "cursor");
  const projectItems = items.filter(i => i.scope === "project");

  if (globalClaude.length > 0) {
    groups.push({ label: "Claude Code — 全局", items: globalClaude });
  }
  if (globalCursor.length > 0) {
    groups.push({ label: "Cursor — 全局", items: globalCursor });
  }

  const projectMap = new Map<string, ConfigItem[]>();
  for (const item of projectItems) {
    const key = item.project_path ?? "unknown";
    const list = projectMap.get(key) ?? [];
    list.push(item);
    projectMap.set(key, list);
  }

  for (const [projectPath, projectConfigs] of projectMap) {
    const shortName = projectPath.split("/").pop() ?? projectPath;
    groups.push({ label: `项目: ${shortName}`, items: projectConfigs });
  }

  return groups;
}
