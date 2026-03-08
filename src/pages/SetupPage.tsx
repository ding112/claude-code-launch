import { useState, useCallback } from "react";
import { usePrereqs } from "../hooks/usePrereqs";
import { useInstall } from "../hooks/useInstall";
import { useVerify } from "../hooks/useVerify";
import { LogPanel } from "../components/LogPanel";
import type { LogEvent, PrereqResult } from "../types";

type SetupPhase = "idle" | "running" | "success" | "failed";

export default function SetupPage({
  logs,
  onInstallComplete,
  initialPrereqs,
}: {
  logs: LogEvent[];
  onInstallComplete?: () => void;
  initialPrereqs?: PrereqResult | null;
}) {
  const prereqs = usePrereqs();
  const install = useInstall();
  const verify = useVerify();
  const [phase, setPhase] = useState<SetupPhase>("idle");
  const [statusMessage, setStatusMessage] = useState("");

  const prereqData = prereqs.data ?? initialPrereqs;

  const runOneClick = useCallback(async () => {
    setPhase("running");
    setStatusMessage("正在检测环境...");

    try {
      const prereqResult = await prereqs.run();

      const hasBlocker = prereqResult.items.some(
        (item) => !item.available && item.severity === "blocker"
      );
      if (hasBlocker && !prereqResult.claudeInstalled) {
        setPhase("failed");
        setStatusMessage("环境检测发现阻塞项，请查看日志解决后重试");
        return;
      }

      if (prereqResult.claudeInstalled) {
        setPhase("success");
        setStatusMessage(
          `Claude Code 已安装 (${prereqResult.claudeVersion ?? "unknown"})`
        );
        return;
      }

      setStatusMessage("正在安装 Claude Code...");
      const installResult = await install.run();

      if (installResult.status !== "success") {
        setPhase("failed");
        setStatusMessage(`安装失败: ${installResult.summary}`);
        return;
      }

      setStatusMessage("正在验证安装...");
      const verifyResult = await verify.run();

      if (verifyResult.success) {
        setPhase("success");
        setStatusMessage(
          `安装完成! ${verifyResult.versionOutput.trim()}`
        );
      } else {
        setPhase("failed");
        setStatusMessage(
          verifyResult.errorSummary ?? "验证失败，请查看日志"
        );
      }
    } catch {
      setPhase("failed");
      setStatusMessage("安装过程出错，请查看日志");
    }
  }, [prereqs, install, verify]);

  const isRunning = phase === "running";

  return (
    <div className="flex flex-col gap-6 p-6 max-w-3xl mx-auto">
      <div>
        <h2 className="text-xl font-bold text-slate-900 mb-1">
          Claude Code 安装
        </h2>
        <p className="text-sm text-slate-500">
          一键检测环境、安装 Claude Code 并验证
        </p>
      </div>

      {prereqData && (
        <div className="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
          <h3 className="text-sm font-semibold text-slate-800 mb-3">
            环境状态
          </h3>
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-slate-500">平台: </span>
              <span className="font-medium">{prereqData.platform}</span>
            </div>
            <div>
              <span className="text-slate-500">Claude Code: </span>
              <span
                className={`font-medium ${prereqData.claudeInstalled ? "text-emerald-600" : "text-amber-600"}`}
              >
                {prereqData.claudeInstalled
                  ? prereqData.claudeVersion ?? "已安装"
                  : "未安装"}
              </span>
            </div>
            {prereqData.items.map((item) => (
              <div key={item.name}>
                <span className="text-slate-500">{item.name}: </span>
                <span
                  className={`font-medium ${item.available ? "text-emerald-600" : item.severity === "blocker" ? "text-rose-600" : "text-amber-600"}`}
                >
                  {item.available ? "可用" : item.message}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex items-center gap-4">
        <button
          type="button"
          disabled={isRunning}
          onClick={runOneClick}
          className={`px-6 py-2.5 rounded-lg font-medium text-sm transition-all ${
            isRunning
              ? "bg-slate-300 text-slate-500 cursor-not-allowed"
              : "bg-black text-white hover:bg-slate-800 shadow-sm"
          }`}
        >
          {isRunning
            ? "安装中..."
            : phase === "success"
              ? "重新安装"
              : "一键安装"}
        </button>

        {phase === "success" && onInstallComplete && (
          <button
            type="button"
            onClick={onInstallComplete}
            className="px-6 py-2.5 rounded-lg font-medium text-sm bg-emerald-600 text-white hover:bg-emerald-700 shadow-sm transition-all"
          >
            进入监控仪表盘
          </button>
        )}
      </div>

      {statusMessage && (
        <div
          className={`rounded-lg px-4 py-3 text-sm font-medium ${
            phase === "success"
              ? "bg-emerald-50 text-emerald-800 border border-emerald-200"
              : phase === "failed"
                ? "bg-rose-50 text-rose-800 border border-rose-200"
                : "bg-blue-50 text-blue-800 border border-blue-200"
          }`}
        >
          {statusMessage}
        </div>
      )}

      <LogPanel logs={logs} />
    </div>
  );
}
