import type { LogEvent } from "../types";

interface LogPanelProps {
  logs: LogEvent[];
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString();
}

const ANSI_ESCAPE_PATTERN =
  // CSI + OSC 序列，覆盖常见颜色和终端控制输出
  /(?:\x1B\[[0-?]*[ -/]*[@-~])|(?:\x1B\][^\x07\x1B]*(?:\x07|\x1B\\))/g;
const CONTROL_CHAR_PATTERN = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g;

function normalizeCarriageReturn(input: string) {
  // 进度条通常使用 \r 覆写同一行，这里仅保留覆写后的最终内容
  return input
    .split("\n")
    .map((line) => line.split("\r").pop() ?? "")
    .join("\n");
}

function sanitizeForDisplay(input: string) {
  return normalizeCarriageReturn(input)
    .replace(ANSI_ESCAPE_PATTERN, "")
    .replace(CONTROL_CHAR_PATTERN, "");
}

export function LogPanel({ logs }: LogPanelProps) {
  const allText = logs
    .map((item) => {
      const rawMessage = item.raw && item.raw.length > 0 ? item.raw : item.message;
      return `[${formatTime(item.timestamp)}] [${item.level}] ${rawMessage}`;
    })
    .join("\n");

  return (
    <section className="min-w-0 rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-800">实时日志</h3>
        <button
          type="button"
          className="rounded-md border border-slate-300 px-2 py-1 text-xs text-slate-600 hover:bg-slate-50"
          onClick={() => navigator.clipboard.writeText(allText)}
        >
          复制日志
        </button>
      </div>
      <div className="min-w-0 max-h-72 overflow-auto rounded-md border border-slate-200 bg-slate-50 p-3 font-mono text-xs text-slate-700">
        {logs.length === 0 ? (
          <p className="text-slate-500">暂无日志，执行检测/安装/验证后会显示输出。</p>
        ) : (
          logs.map((item, idx) => (
            <p
              key={`${item.timestamp}-${idx}`}
              className="whitespace-pre-wrap break-all py-0.5"
              title={item.raw ?? undefined}
            >
              <span className="mr-2 text-slate-400">[{formatTime(item.timestamp)}]</span>
              <span
                className={
                  item.level === "error"
                    ? "text-rose-600"
                    : item.level === "warn"
                      ? "text-amber-600"
                      : "text-emerald-600"
                }
              >
                {sanitizeForDisplay(item.message)}
              </span>
            </p>
          ))
        )}
      </div>
    </section>
  );
}
