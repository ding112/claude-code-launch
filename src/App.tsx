import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import TabSwitcher, { type Tab } from "./components/TabSwitcher";
import SessionsPage from "./pages/SessionsPage";
import SettingsPage from "./pages/SettingsPage";
import SetupPage from "./pages/SetupPage";
import type { LogEvent, PrereqResult } from "./types";
import "./App.css";

type AppMode = "loading" | "setup" | "dashboard";

function App() {
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [mode, setMode] = useState<AppMode>("loading");
  const [tab, setTab] = useState<Tab>("sessions");
  const [initialPrereqs, setInitialPrereqs] = useState<PrereqResult | null>(null);

  useEffect(() => {
    const unlistenPromise = listen<LogEvent>("launch-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    invoke<PrereqResult>("check_prereqs")
      .then((result) => {
        setInitialPrereqs(result);
        if (result.claudeInstalled) {
          setMode("dashboard");
        } else {
          setMode("setup");
        }
      })
      .catch(() => {
        setMode("setup");
      });
  }, []);

  if (mode === "loading") {
    return (
      <main className="flex min-h-screen items-center justify-center bg-slate-50">
        <div className="text-center">
          <div className="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-slate-300 border-t-black" />
          <p className="text-sm text-slate-500">正在检测环境...</p>
        </div>
      </main>
    );
  }

  if (mode === "setup") {
    return (
      <main className="min-h-screen bg-slate-50">
        <SetupPage
          logs={logs}
          initialPrereqs={initialPrereqs}
          onInstallComplete={() => setMode("dashboard")}
        />
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-slate-50">
      <header className="flex items-center justify-between border-b border-black/6 bg-white px-6 py-3">
        <h1 className="text-lg font-bold tracking-tight text-slate-900">
          Claude Code Launch
        </h1>
        <TabSwitcher tab={tab} onTabChange={setTab} />
      </header>

      {tab === "sessions" && <SessionsPage />}
      {tab === "settings" && <SettingsPage />}
      {tab === "setup" && (
        <SetupPage logs={logs} initialPrereqs={initialPrereqs} />
      )}
    </main>
  );
}

export default App;
