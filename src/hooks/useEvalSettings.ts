import { useState } from "react";
import type { EvalSettings } from "../types";
import { fetchSettings, saveSettingsApi } from "../api";

export function useEvalSettings() {
  const [settings, setSettings] = useState<EvalSettings>({
    enabled: true,
    sampling_rate: 1,
    provider: "openai",
    model: "gpt-4o-mini",
    base_url: "https://api.openai.com/v1",
    api_key: "",
    timeout_ms: 8000,
  });
  const [saveMessage, setSaveMessage] = useState<string>("");

  const loadSettings = async () => {
    const data = await fetchSettings();
    setSettings(data);
  };

  const saveSettings = async () => {
    setSaveMessage("");
    const ok = await saveSettingsApi(settings);
    setSaveMessage(ok ? "已保存并生效。" : "保存失败，请检查配置。");
  };

  return {
    settings,
    setSettings,
    saveMessage,
    loadSettings,
    saveSettings,
  };
}
