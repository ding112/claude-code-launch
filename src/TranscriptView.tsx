import { useState, useMemo, type UIEvent } from "react";
import type { TranscriptLineItem } from "./types";

type ParsedRecord = {
  line_no: number;
  raw: string;
  parsed: TranscriptEntry | null;
};

type TranscriptEntry = {
  type: string;
  subtype?: string;
  timestamp?: string;
  uuid?: string;
  parentUuid?: string | null;
  isSidechain?: boolean;
  message?: MessagePayload;
  data?: ProgressData;
  hookCount?: number;
  hookInfos?: { command: string; durationMs: number }[];
  level?: string;
  stopReason?: string;
  hasOutput?: boolean;
  [key: string]: unknown;
};

type MessagePayload = {
  role?: string;
  model?: string;
  content?: string | ContentBlock[];
  stop_reason?: string;
  usage?: Record<string, unknown>;
};

type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; thinking: string }
  | { type: "tool_use"; name: string; id: string; input: Record<string, unknown> }
  | { type: "tool_result"; content: string; tool_use_id: string };

type ProgressData = {
  type?: string;
  hookEvent?: string;
  hookName?: string;
  command?: string;
};

type TranscriptViewProps = {
  items: TranscriptLineItem[];
  loadingMore: boolean;
  hasMore: boolean;
  onScroll: (event: UIEvent<HTMLDivElement>) => void;
};

function parseLines(items: TranscriptLineItem[]): ParsedRecord[] {
  return items.map((item) => {
    const trimmed = item.line_content.trim();
    if (!trimmed) return { line_no: item.line_no, raw: item.line_content, parsed: null };
    try {
      const obj = JSON.parse(trimmed) as TranscriptEntry;
      if (typeof obj !== "object" || obj === null) {
        return { line_no: item.line_no, raw: item.line_content, parsed: null };
      }
      // Claude Code format uses `type`, Cursor format uses `role` at top level
      if (!obj.type && typeof (obj as Record<string, unknown>).role === "string") {
        obj.type = (obj as Record<string, unknown>).role as string;
      }
      if (typeof obj.type === "string") {
        return { line_no: item.line_no, raw: item.line_content, parsed: obj };
      }
      return { line_no: item.line_no, raw: item.line_content, parsed: null };
    } catch {
      return { line_no: item.line_no, raw: item.line_content, parsed: null };
    }
  });
}

export default function TranscriptView({ items, loadingMore, hasMore, onScroll }: TranscriptViewProps) {
  const records = useMemo(() => parseLines(items), [items]);

  return (
    <div
      className="m-0 max-h-[700px] overflow-auto bg-slate-50 border border-black/6 rounded-md p-4 flex flex-col gap-1"
      onScroll={onScroll}
    >
      {loadingMore && (
        <p className="text-gray-400 m-0 py-1 text-xs text-center">加载更多中...</p>
      )}
      {records.map((rec) => (
        <RecordRow key={rec.line_no} record={rec} />
      ))}
      {!hasMore && (
        <p className="text-gray-400 m-0 py-1 text-xs text-center">已到最早内容</p>
      )}
      {hasMore && !loadingMore && (
        <p className="text-gray-400 m-0 py-1 text-xs text-center">向下滚动加载更早内容</p>
      )}
    </div>
  );
}

function RecordRow({ record }: { record: ParsedRecord }) {
  if (!record.parsed) {
    const display = record.raw.endsWith("\n") ? record.raw.slice(0, -1) : record.raw;
    if (!display.trim()) return null;
    return (
      <div className="font-mono text-xs text-gray-500 whitespace-pre-wrap break-words py-0.5">
        {display}
      </div>
    );
  }

  const entry = record.parsed;
  switch (entry.type) {
    case "user":
      return <UserMessage entry={entry} />;
    case "assistant":
      return <AssistantMessage entry={entry} />;
    case "system":
      return <SystemMessage entry={entry} />;
    case "progress":
      return <ProgressMessage entry={entry} />;
    case "file-history-snapshot":
      return null;
    default:
      return <GenericMessage entry={entry} />;
  }
}

function Timestamp({ value }: { value?: string }) {
  if (!value) return null;
  const d = new Date(value);
  const str = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;
  return <span className="text-[10px] text-gray-400 font-mono shrink-0">{str}</span>;
}

function extractUserText(entry: TranscriptEntry): string {
  const content = entry.message?.content;
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    const parts: string[] = [];
    for (const block of content) {
      if (block.type === "text") parts.push(block.text);
      else if (block.type === "tool_result") parts.push(`[Tool Result: ${block.tool_use_id}]`);
    }
    return parts.join("\n") || "[tool result]";
  }
  return "[empty]";
}

function UserMessage({ entry }: { entry: TranscriptEntry }) {
  const text = extractUserText(entry);
  return (
    <div className="flex justify-end items-start gap-2 my-1">
      <Timestamp value={entry.timestamp} />
      <div className="max-w-[80%] bg-blue-600 text-white rounded-2xl rounded-tr-md px-4 py-2.5 text-sm whitespace-pre-wrap break-words shadow-sm">
        {text}
      </div>
    </div>
  );
}

function AssistantMessage({ entry }: { entry: TranscriptEntry }) {
  const content = entry.message?.content;
  const blocks: ContentBlock[] = Array.isArray(content)
    ? (content as ContentBlock[])
    : typeof content === "string"
      ? [{ type: "text" as const, text: content }]
      : [];

  const model = entry.message?.model;
  const stopReason = entry.message?.stop_reason;

  return (
    <div className="flex justify-start items-start gap-2 my-1">
      <div className="w-6 h-6 rounded-full bg-gray-800 text-white flex items-center justify-center text-[10px] font-bold shrink-0 mt-0.5">
        AI
      </div>
      <div className="max-w-[85%] flex flex-col gap-1.5">
        {model && (
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-gray-400 font-mono">{model}</span>
            {stopReason && <span className="text-[10px] text-gray-400">· {stopReason}</span>}
            <Timestamp value={entry.timestamp} />
          </div>
        )}
        {blocks.map((block, i) => (
          <AssistantBlock key={i} block={block} />
        ))}
      </div>
    </div>
  );
}

function AssistantBlock({ block }: { block: ContentBlock }) {
  switch (block.type) {
    case "text":
      return (
        <div className="bg-white border border-black/6 rounded-2xl rounded-tl-md px-4 py-2.5 text-sm whitespace-pre-wrap break-words shadow-sm text-gray-900 leading-relaxed">
          {block.text}
        </div>
      );
    case "thinking":
      return <ThinkingBlock text={block.thinking} />;
    case "tool_use":
      return <ToolUseBlock name={block.name} input={block.input} id={block.id} />;
    case "tool_result":
      return (
        <div className="bg-gray-50 border border-black/6 rounded-lg px-3 py-2 text-xs font-mono text-gray-600 whitespace-pre-wrap break-words max-h-[200px] overflow-auto">
          <span className="text-gray-400 text-[10px] uppercase tracking-wider font-semibold">Result</span>
          <div className="mt-1">{typeof block.content === "string" ? block.content : JSON.stringify(block.content)}</div>
        </div>
      );
    default:
      return null;
  }
}

function ThinkingBlock({ text }: { text: string }) {
  const [expanded, setExpanded] = useState(false);
  const preview = text.length > 120 ? `${text.slice(0, 120)}...` : text;

  return (
    <div className="border border-dashed border-purple-200 bg-purple-50/50 rounded-lg px-3 py-2 text-xs">
      <button
        type="button"
        className="flex items-center gap-1.5 text-purple-500 font-medium bg-transparent border-none p-0 cursor-pointer text-xs shadow-none hover:text-purple-700 hover:shadow-none"
        onClick={() => setExpanded((p) => !p)}
      >
        <span className="text-[10px]">{expanded ? "▼" : "▶"}</span>
        Thinking
      </button>
      <div className="mt-1 text-gray-600 whitespace-pre-wrap break-words leading-relaxed">
        {expanded ? text : preview}
      </div>
    </div>
  );
}

function ToolUseBlock({ name, input, id }: { name: string; input: Record<string, unknown>; id: string }) {
  const [expanded, setExpanded] = useState(false);

  const summary = useMemo(() => {
    if (name === "Read" || name === "Write") return String(input.path ?? input.file_path ?? "");
    if (name === "Bash" || name === "Shell") return truncate(String(input.command ?? ""), 80);
    if (name === "Grep" || name === "Search") return truncate(String(input.pattern ?? input.query ?? ""), 80);
    if (name === "Glob") return truncate(String(input.glob_pattern ?? input.pattern ?? ""), 80);
    const keys = Object.keys(input);
    if (keys.length === 0) return "";
    const first = String(input[keys[0]] ?? "");
    return truncate(first, 60);
  }, [name, input]);

  return (
    <div className="border border-black/8 bg-amber-50/60 rounded-lg px-3 py-2 text-xs">
      <button
        type="button"
        className="flex items-center gap-1.5 font-medium bg-transparent border-none p-0 cursor-pointer text-xs text-amber-700 shadow-none hover:text-amber-900 hover:shadow-none"
        onClick={() => setExpanded((p) => !p)}
      >
        <span className="text-[10px]">{expanded ? "▼" : "▶"}</span>
        <span className="font-mono font-semibold">{name}</span>
        {summary && <span className="text-gray-500 font-normal truncate max-w-[400px]">{summary}</span>}
      </button>
      {expanded && (
        <pre className="mt-2 m-0 overflow-auto max-h-[300px] bg-white rounded-md p-3 font-mono text-[11px] text-gray-700 border border-black/6 whitespace-pre-wrap break-all leading-relaxed">
          {JSON.stringify(input, null, 2)}
        </pre>
      )}
      <span className="text-[9px] text-gray-400 font-mono">{id}</span>
    </div>
  );
}

function SystemMessage({ entry }: { entry: TranscriptEntry }) {
  const [expanded, setExpanded] = useState(false);
  const subtype = entry.subtype ?? "";
  const level = entry.level ?? "";

  const levelColor = level === "suggestion"
    ? "bg-green-50 border-green-200 text-green-700"
    : level === "warning"
      ? "bg-yellow-50 border-yellow-200 text-yellow-700"
      : "bg-gray-100 border-black/6 text-gray-600";

  return (
    <div className="flex flex-col items-center my-1">
      <button
        type="button"
        className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-medium cursor-pointer bg-transparent shadow-none hover:shadow-none ${levelColor}`}
        onClick={() => setExpanded((p) => !p)}
      >
        <span className="text-[9px]">{expanded ? "▼" : "▶"}</span>
        ⚙ {subtype || "system"}
        {entry.hookCount != null && <span className="opacity-60">({entry.hookCount} hooks)</span>}
        <Timestamp value={entry.timestamp} />
      </button>
      {expanded && (
        <pre className="mt-1 text-[10px] font-mono text-gray-500 bg-white border border-black/6 rounded-md p-2 max-h-[200px] overflow-auto whitespace-pre-wrap break-all w-full max-w-[600px]">
          {JSON.stringify(entry, null, 2)}
        </pre>
      )}
    </div>
  );
}

function ProgressMessage({ entry }: { entry: TranscriptEntry }) {
  const hookEvent = entry.data?.hookEvent ?? "";
  const hookName = entry.data?.hookName ?? "";

  return (
    <div className="flex justify-center my-0.5">
      <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-gray-100 border border-black/4 text-[10px] text-gray-500 font-mono">
        ↻ {hookEvent}{hookName && hookName !== hookEvent ? ` · ${hookName}` : ""}
        <Timestamp value={entry.timestamp} />
      </span>
    </div>
  );
}

function GenericMessage({ entry }: { entry: TranscriptEntry }) {
  const [expanded, setExpanded] = useState(false);
  return (
    <div className="flex flex-col items-center my-0.5">
      <button
        type="button"
        className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-gray-50 border border-black/6 text-[10px] text-gray-500 font-mono cursor-pointer shadow-none hover:shadow-none"
        onClick={() => setExpanded((p) => !p)}
      >
        <span className="text-[9px]">{expanded ? "▼" : "▶"}</span>
        {entry.type}
        <Timestamp value={entry.timestamp} />
      </button>
      {expanded && (
        <pre className="mt-1 text-[10px] font-mono text-gray-500 bg-white border border-black/6 rounded-md p-2 max-h-[200px] overflow-auto whitespace-pre-wrap break-all w-full max-w-[600px]">
          {JSON.stringify(entry, null, 2)}
        </pre>
      )}
    </div>
  );
}

function truncate(str: string, max: number): string {
  return str.length > max ? `${str.slice(0, max)}…` : str;
}
