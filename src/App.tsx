import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Stepper } from "./components/Stepper";
import { LogPanel } from "./components/LogPanel";
import { usePrereqs } from "./hooks/usePrereqs";
import { useInstall } from "./hooks/useInstall";
import { useVerify } from "./hooks/useVerify";
import type { LogEvent } from "./types";

function App() {
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [step, setStep] = useState(0);
  const prereqs = usePrereqs();
  const install = useInstall();
  const verify = useVerify();

  useEffect(() => {
    const unlistenPromise = listen<LogEvent>("launch-log", (event) => {
      setLogs((prev) => [...prev, event.payload]);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    handleCheck();
  }, []);

  const blockerCount = useMemo(
    () =>
      prereqs.data?.items.filter((item) => item.severity === "blocker" && !item.available)
        .length ?? 0,
    [prereqs.data],
  );

  async function handleCheck() {
    setStep(0);
    const result = await prereqs.run();
    if (result.claudeInstalled) {
      setStep(2);
      return;
    }
    setStep(1);
  }

  async function handleInstall() {
    if (prereqs.data?.claudeInstalled) {
      setStep(2);
      return;
    }
    setStep(1);
    const result = await install.run();
    if (result.status === "success") {
      await handleVerify();
    } else {
      setStep(2);
    }
  }

  async function handleVerify() {
    setStep(2);
    try {
      const result = await verify.run();
      if (result.success) {
        setStep(3);
      }
    } catch (e) {
      // 保持在当前步骤，等待重试
    }
  }

  const installStatusText = install.data
    ? install.data.status === "success"
      ? "安装成功"
      : "安装失败"
    : prereqs.data?.claudeInstalled
      ? "已安装，已跳过"
    : "未执行";

  return (
    <main className="mx-auto min-h-screen max-w-5xl space-y-4 bg-slate-50 px-6 py-6 text-slate-900">
      <header className="space-y-2">
        <h1 className="text-2xl font-bold">Claude Code 安装向导</h1>
        <p className="text-sm text-slate-500">
          支持 Windows / macOS / Linux，通过 npm 全局安装 Claude Code。
        </p>
      </header>

      <Stepper currentStep={step} />

      <section className="grid min-w-0 gap-4 rounded-xl border border-slate-200 bg-white p-4 shadow-sm md:grid-cols-3">
        <div className="min-w-0 rounded-lg border border-slate-200 p-3">
          <h2 className="mb-2 text-sm font-semibold">1. 环境检测</h2>
          <button
            type="button"
            onClick={handleCheck}
            disabled={prereqs.loading}
            className="rounded-md bg-blue-600 px-3 py-2 text-sm text-white hover:bg-blue-500 disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-400"
          >
            {prereqs.loading ? "检测中..." : "执行检测"}
          </button>
          <p className="mt-2 text-xs text-slate-500">
            {prereqs.data
              ? `平台: ${prereqs.data.platform}，阻塞项: ${blockerCount}${
                  prereqs.data.claudeInstalled
                    ? `，Claude: ${prereqs.data.claudeVersion ?? "已安装"}`
                    : ""
                }`
              : "尚未检测"}
          </p>
        </div>

        <div className="min-w-0 rounded-lg border border-slate-200 p-3">
          <h2 className="mb-2 text-sm font-semibold">2. 安装执行</h2>
          <button
            type="button"
            onClick={handleInstall}
            disabled={install.loading || !prereqs.data || prereqs.data.claudeInstalled}
            className="rounded-md bg-indigo-600 px-3 py-2 text-sm text-white hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-400"
          >
            {prereqs.data?.claudeInstalled
              ? "已安装，跳过安装"
              : install.loading
                ? "安装中..."
                : "开始安装"}
          </button>
          <div className="mt-3 flex items-center gap-2">
            <span className="text-xs text-slate-500">状态:</span>
            {install.data ? (
              <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${
                install.data.status === 'success' ? 'bg-emerald-50 text-emerald-700 border border-emerald-200' :
                install.data.status === 'failed' ? 'bg-rose-50 text-rose-700 border border-rose-200' :
                'bg-amber-50 text-amber-700 border border-amber-200'
              }`}>
                {install.data.status === 'success' && (
                  <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
                )}
                {install.data.status === 'failed' && (
                  <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
                )}
                {installStatusText}
              </span>
            ) : (
              <span className="text-xs text-slate-500">未执行</span>
            )}
          </div>
          {install.data?.summary ? (
            <p className="mt-2 text-xs leading-relaxed text-slate-500">{install.data.summary}</p>
          ) : null}
        </div>

        <div className="min-w-0 rounded-lg border border-slate-200 p-3">
          <h2 className="mb-2 text-sm font-semibold">3. 安装验证</h2>
          <button
            type="button"
            onClick={handleVerify}
            disabled={verify.loading || (!install.data && !prereqs.data?.claudeInstalled)}
            className="rounded-md bg-emerald-600 px-3 py-2 text-sm text-white hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-400"
          >
            {verify.loading ? "验证中..." : "执行验证"}
          </button>
          <div className="mt-3 flex items-center gap-2">
            <span className="text-xs text-slate-500">状态:</span>
            {verify.data ? (
              <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${
                verify.data.success ? 'bg-emerald-50 text-emerald-700 border border-emerald-200' : 'bg-rose-50 text-rose-700 border border-rose-200'
              }`}>
                {verify.data.success ? (
                  <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
                ) : (
                  <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
                )}
                {verify.data.success ? "验证通过" : "验证失败"}
              </span>
            ) : (
              <span className="text-xs text-slate-500">尚未验证</span>
            )}
          </div>
        </div>
      </section>

      <section className="grid min-w-0 gap-4 md:grid-cols-2">
        <article className="min-w-0 rounded-xl border border-slate-200 bg-white p-4 text-sm text-slate-800 shadow-sm">
          <h3 className="mb-2 font-semibold">检测结果</h3>
          {!prereqs.data ? (
            <p className="text-slate-500">暂无检测结果。</p>
          ) : (
            <ul className="space-y-3">
              {prereqs.data.items.map((item) => {
                const isSuccess = item.available;
                const isBlocker = !isSuccess && item.severity === "blocker";
                const isWarning = !isSuccess && item.severity === "warning";
                
                let icon;
                let bgClass;
                let borderClass;
                let textClass;
                let iconColor;

                if (isSuccess) {
                  icon = (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                  );
                  bgClass = "bg-emerald-50/50";
                  borderClass = "border-emerald-200";
                  textClass = "text-emerald-800";
                  iconColor = "text-emerald-500";
                } else if (isBlocker) {
                  icon = (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  );
                  bgClass = "bg-rose-50/50";
                  borderClass = "border-rose-200";
                  textClass = "text-rose-800";
                  iconColor = "text-rose-500";
                } else if (isWarning) {
                  icon = (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                  );
                  bgClass = "bg-amber-50/50";
                  borderClass = "border-amber-200";
                  textClass = "text-amber-800";
                  iconColor = "text-amber-500";
                } else {
                  icon = (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  );
                  bgClass = "bg-blue-50/50";
                  borderClass = "border-blue-200";
                  textClass = "text-blue-800";
                  iconColor = "text-blue-500";
                }

                return (
                  <li key={item.name} className={`flex items-start gap-3 rounded-lg border p-3 ${bgClass} ${borderClass}`}>
                    <div className={`mt-0.5 shrink-0 ${iconColor}`}>
                      {icon}
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center justify-between gap-2">
                        <p className={`font-medium ${textClass}`}>{item.name}</p>
                        <span className={`shrink-0 rounded px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider ${
                          isSuccess ? 'bg-emerald-100 text-emerald-700' : 
                          isBlocker ? 'bg-rose-100 text-rose-700' : 
                          isWarning ? 'bg-amber-100 text-amber-700' : 
                          'bg-blue-100 text-blue-700'
                        }`}>
                          {isSuccess ? '可用' : item.severity}
                        </span>
                      </div>
                      <p className="mt-1 text-xs leading-relaxed text-slate-500">{item.message}</p>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </article>

        <article className={`min-w-0 rounded-xl border p-4 text-sm shadow-sm ${
          verify.data 
            ? verify.data.success 
              ? 'border-emerald-200 bg-emerald-50/50' 
              : 'border-rose-200 bg-rose-50/50'
            : 'border-slate-200 bg-white'
        }`}>
          <div className="mb-3 flex items-center justify-between">
            <h3 className={`font-semibold ${
              verify.data 
                ? verify.data.success ? 'text-emerald-700' : 'text-rose-700'
                : 'text-slate-800'
            }`}>
              验证输出
            </h3>
            {verify.data && (
              <span className={`text-xs font-medium ${verify.data.success ? 'text-emerald-600' : 'text-rose-600'}`}>
                {verify.data.success ? '🎉 Claude Code 已就绪' : '⚠️ 验证未通过'}
              </span>
            )}
          </div>
          {!verify.data ? (
            <p className="text-slate-500">暂无验证结果。</p>
          ) : (
            <div className="space-y-2">
              <pre className="min-w-0 max-h-28 overflow-auto whitespace-pre-wrap break-all rounded border border-slate-200 bg-slate-50 p-2 text-xs text-slate-700">
                {verify.data.versionOutput || "无版本输出"}
              </pre>
              <pre className="min-w-0 max-h-28 overflow-auto whitespace-pre-wrap break-all rounded border border-slate-200 bg-slate-50 p-2 text-xs text-slate-700">
                {verify.data.doctorOutput || "无 doctor 输出"}
              </pre>
              {verify.data.errorSummary ? (
                <p className="text-xs text-rose-600">{verify.data.errorSummary}</p>
              ) : null}
            </div>
          )}
        </article>
      </section>

      <LogPanel logs={logs} />

      <section className="rounded-xl border border-slate-200 bg-white p-4 text-xs text-slate-600 shadow-sm">
        <h3 className="mb-2 font-semibold text-slate-800">失败排障建议</h3>
        <ul className="list-disc space-y-1 pl-4">
          <li>检查网络是否可访问 `nodejs.org` 与 `registry.npmmirror.com`。</li>
          <li>若 npm 不可用，向导会自动下载安装 Node.js LTS；如自动安装失败，请手动安装 Node.js。</li>
          <li>若 `claude` 不可用，请重开终端并检查 PATH。</li>
        </ul>
      </section>
    </main>
  );
}

export default App;
