import { useState, useMemo, useEffect, useRef, type UIEvent } from "react";
import type { TranscriptLineItem, EventItem } from "./types";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@/components/ui/collapsible";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { truncate } from "./utils";

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

export type TimeRange = { minMs: number; maxMs: number };

type TranscriptViewProps = {
  items: TranscriptLineItem[];
  events: EventItem[];
  loadingMore: boolean;
  hasMore: boolean;
  skippedLines?: number;
  onScroll: (event: UIEvent<HTMLDivElement>) => void;
  onTimeRangeChange?: (range: TimeRange | null) => void;
};

type MergedTimelineItem =
  | { kind: "transcript"; record: ParsedRecord; timestampMs: number }
  | { kind: "event"; event: EventItem; timestampMs: number };

type ParseLinesResult = {
  records: ParsedRecord[];
  timeRange: { minMs: number; maxMs: number } | null;
};

function parseLines(items: TranscriptLineItem[]): ParseLinesResult {
  let minMs = Infinity;
  let maxMs = -Infinity;

  const records = items.map((item) => {
    const trimmed = item.line_content.trim();
    if (!trimmed) return { line_no: item.line_no, raw: item.line_content, parsed: null };
    try {
      const obj = JSON.parse(trimmed) as TranscriptEntry;
      if (typeof obj !== "object" || obj === null) {
        return { line_no: item.line_no, raw: item.line_content, parsed: null };
      }
      if (obj.timestamp) {
        const ms = new Date(obj.timestamp).getTime();
        if (!Number.isNaN(ms)) {
          if (ms < minMs) minMs = ms;
          if (ms > maxMs) maxMs = ms;
        }
      }
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

  const timeRange = minMs === Infinity || maxMs === -Infinity ? null : { minMs, maxMs };
  return { records, timeRange };
}

function inferInitialTimestamp(records: ParsedRecord[], events: EventItem[]): number {
  for (const r of records) {
    if (r.parsed?.timestamp) {
      const ms = new Date(r.parsed.timestamp).getTime();
      if (!Number.isNaN(ms)) return ms;
    }
  }
  if (events.length > 0) {
    return Math.min(...events.map((e) => e.created_at_ms));
  }
  return 0;
}

function mergeTimeline(records: ParsedRecord[], events: EventItem[]): MergedTimelineItem[] {
  let lastTs = inferInitialTimestamp(records, events);
  const tsRecords: MergedTimelineItem[] = records.map((r) => {
    if (r.parsed?.timestamp) {
      const ms = new Date(r.parsed.timestamp).getTime();
      if (!Number.isNaN(ms)) lastTs = ms;
    }
    return { kind: "transcript", record: r, timestampMs: lastTs };
  });

  const tsEvents: MergedTimelineItem[] = events.map((e) => ({
    kind: "event",
    event: e,
    timestampMs: e.created_at_ms,
  }));

  const result: MergedTimelineItem[] = [];
  let i = 0;
  let j = 0;
  while (i < tsRecords.length && j < tsEvents.length) {
    if (tsRecords[i].timestampMs <= tsEvents[j].timestampMs) {
      result.push(tsRecords[i++]);
    } else {
      result.push(tsEvents[j++]);
    }
  }
  while (i < tsRecords.length) result.push(tsRecords[i++]);
  while (j < tsEvents.length) result.push(tsEvents[j++]);
  return result;
}

export default function TranscriptView({ items, events, loadingMore, hasMore, skippedLines, onScroll, onTimeRangeChange }: TranscriptViewProps) {
  const [ascending, setAscending] = useState(true);
  const { records, timeRange } = useMemo(() => parseLines(items), [items]);
  const merged = useMemo(() => {
    const list = mergeTimeline(records, events);
    return ascending ? list : [...list].reverse();
  }, [records, events, ascending]);

  const prevRangeKeyRef = useRef("");
  useEffect(() => {
    const key = timeRange ? `${timeRange.minMs}-${timeRange.maxMs}` : "";
    if (key !== prevRangeKeyRef.current) {
      prevRangeKeyRef.current = key;
      onTimeRangeChange?.(timeRange);
    }
  }, [timeRange, onTimeRangeChange]);

  return (
    <div className="flex flex-col gap-2 flex-1 min-h-0">
      <div className="flex justify-end shrink-0">
        <Button
          variant="outline"
          size="sm"
          className="text-xs"
          onClick={() => setAscending((v) => !v)}
        >
          {ascending ? "正序 ↑" : "倒序 ↓"}
        </Button>
      </div>
      <div
        className="flex-1 min-h-0 overflow-y-auto rounded-md border bg-muted p-4 flex flex-col gap-1"
        onScroll={onScroll}
      >
        {loadingMore && (
          <p className="text-muted-foreground m-0 py-1 text-xs text-center">加载更多中...</p>
        )}
        {merged.map((item) =>
          item.kind === "transcript" ? (
            <RecordRow key={`t-${item.record.line_no}`} record={item.record} />
          ) : (
            <EventBubble key={`e-${item.event.event_id}`} event={item.event} />
          ),
        )}
        {!hasMore && (
          <p className="text-muted-foreground m-0 py-1 text-xs text-center">已到最早内容</p>
        )}
        {hasMore && !loadingMore && (
          <p className="text-muted-foreground m-0 py-1 text-xs text-center">向下滚动加载更早内容</p>
        )}
        {(skippedLines ?? 0) > 0 && (
          <p className="text-muted-foreground/60 m-0 py-1 text-xs text-center">
            已跳过 {skippedLines} 行无效数据
          </p>
        )}
      </div>
    </div>
  );
}

function RecordRow({ record }: { record: ParsedRecord }) {
  if (!record.parsed) {
    const display = record.raw.endsWith("\n") ? record.raw.slice(0, -1) : record.raw;
    if (!display.trim()) return null;
    return (
      <div className="font-mono text-xs text-muted-foreground whitespace-pre-wrap break-words py-0.5">
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
  return <span className="text-[10px] text-muted-foreground font-mono shrink-0">{str}</span>;
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
      <div className="max-w-[80%] bg-primary text-primary-foreground rounded-2xl rounded-tr-md px-4 py-2.5 text-sm whitespace-pre-wrap break-words shadow-sm">
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
      <div className="size-6 rounded-full bg-foreground text-background flex items-center justify-center text-[10px] font-bold shrink-0 mt-0.5">
        AI
      </div>
      <div className="max-w-[85%] flex flex-col gap-1.5">
        {model && (
          <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground font-mono">{model}</span>
            {stopReason && <span className="text-[10px] text-muted-foreground">· {stopReason}</span>}
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
        <div className="bg-card border rounded-2xl rounded-tl-md px-4 py-2.5 text-sm whitespace-pre-wrap break-words shadow-sm leading-relaxed">
          {block.text}
        </div>
      );
    case "thinking":
      return <ThinkingBlock text={block.thinking} />;
    case "tool_use":
      return <ToolUseBlock name={block.name} input={block.input} id={block.id} />;
    case "tool_result":
      return (
        <div className="bg-muted border rounded-lg px-3 py-2 text-xs font-mono text-muted-foreground whitespace-pre-wrap break-words max-h-[200px] overflow-auto">
          <span className="text-muted-foreground text-[10px] uppercase tracking-wider font-semibold">Result</span>
          <div className="mt-1">{typeof block.content === "string" ? block.content : JSON.stringify(block.content)}</div>
        </div>
      );
    default:
      return null;
  }
}

function ThinkingBlock({ text }: { text: string }) {
  const [open, setOpen] = useState(false);
  const preview = text.length > 120 ? `${text.slice(0, 120)}...` : text;

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="border border-dashed border-purple-200 bg-purple-50/50 rounded-lg px-3 py-2 text-xs">
        <CollapsibleTrigger className="flex items-center gap-1.5 text-purple-500 font-medium bg-transparent border-none p-0 cursor-pointer text-xs shadow-none hover:text-purple-700">
          <span className="text-[10px]">{open ? "▼" : "▶"}</span>
          Thinking
        </CollapsibleTrigger>
        <div className="mt-1 text-muted-foreground whitespace-pre-wrap break-words leading-relaxed">
          {open ? text : preview}
        </div>
      </div>
    </Collapsible>
  );
}

function ToolUseBlock({ name, input, id }: { name: string; input: Record<string, unknown>; id: string }) {
  const [open, setOpen] = useState(false);

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
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="border bg-amber-50/60 rounded-lg px-3 py-2 text-xs">
        <CollapsibleTrigger className="flex items-center gap-1.5 font-medium bg-transparent border-none p-0 cursor-pointer text-xs text-amber-700 shadow-none hover:text-amber-900">
          <span className="text-[10px]">{open ? "▼" : "▶"}</span>
          <span className="font-mono font-semibold">{name}</span>
          {summary && <span className="text-muted-foreground font-normal truncate max-w-[400px]">{summary}</span>}
        </CollapsibleTrigger>
        <CollapsibleContent>
          <pre className="mt-2 m-0 overflow-auto max-h-[300px] bg-card rounded-md p-3 font-mono text-[11px] border whitespace-pre-wrap break-all leading-relaxed">
            {JSON.stringify(input, null, 2)}
          </pre>
        </CollapsibleContent>
        <span className="text-[9px] text-muted-foreground font-mono">{id}</span>
      </div>
    </Collapsible>
  );
}

function SystemMessage({ entry }: { entry: TranscriptEntry }) {
  const [open, setOpen] = useState(false);
  const subtype = entry.subtype ?? "";
  const level = entry.level ?? "";

  const levelColor = level === "suggestion"
    ? "bg-emerald-50 border-emerald-200 text-emerald-700"
    : level === "warning"
      ? "bg-yellow-50 border-yellow-200 text-yellow-700"
      : "bg-muted border text-muted-foreground";

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="flex flex-col items-center my-1">
        <CollapsibleTrigger
          className={cn(
            "inline-flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-medium cursor-pointer bg-transparent shadow-none",
            levelColor
          )}
        >
          <span className="text-[9px]">{open ? "▼" : "▶"}</span>
          ⚙ {subtype || "system"}
          {entry.hookCount != null && <span className="opacity-60">({entry.hookCount} hooks)</span>}
          <Timestamp value={entry.timestamp} />
        </CollapsibleTrigger>
        <CollapsibleContent>
          <pre className="mt-1 text-[10px] font-mono text-muted-foreground bg-card border rounded-md p-2 max-h-[200px] overflow-auto whitespace-pre-wrap break-all w-full max-w-[600px]">
            {JSON.stringify(entry, null, 2)}
          </pre>
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}

function ProgressMessage({ entry }: { entry: TranscriptEntry }) {
  const hookEvent = entry.data?.hookEvent ?? "";
  const hookName = entry.data?.hookName ?? "";

  return (
    <div className="flex justify-center my-0.5">
      <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-muted border text-[10px] text-muted-foreground font-mono">
        ↻ {hookEvent}{hookName && hookName !== hookEvent ? ` · ${hookName}` : ""}
        <Timestamp value={entry.timestamp} />
      </span>
    </div>
  );
}

function GenericMessage({ entry }: { entry: TranscriptEntry }) {
  const [open, setOpen] = useState(false);
  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="flex flex-col items-center my-0.5">
        <CollapsibleTrigger className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-muted border text-[10px] text-muted-foreground font-mono cursor-pointer shadow-none">
          <span className="text-[9px]">{open ? "▼" : "▶"}</span>
          {entry.type}
          <Timestamp value={entry.timestamp} />
        </CollapsibleTrigger>
        <CollapsibleContent>
          <pre className="mt-1 text-[10px] font-mono text-muted-foreground bg-card border rounded-md p-2 max-h-[200px] overflow-auto whitespace-pre-wrap break-all w-full max-w-[600px]">
            {JSON.stringify(entry, null, 2)}
          </pre>
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}

function EventBubble({ event }: { event: EventItem }) {
  const [open, setOpen] = useState(false);
  const d = new Date(event.created_at_ms);
  const timeStr = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="flex flex-col items-center my-1">
        <CollapsibleTrigger
          className={cn(
            "inline-flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-medium cursor-pointer bg-transparent shadow-none",
            "bg-sky-50 border-sky-200 text-sky-700",
          )}
        >
          <span className="text-[9px]">{open ? "▼" : "▶"}</span>
          ⚡ {event.event_type}
          <span className="text-[10px] text-sky-500 font-mono shrink-0">{timeStr}</span>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <pre className="mt-1 text-[10px] font-mono text-muted-foreground bg-card border rounded-md p-2 max-h-[300px] overflow-auto whitespace-pre-wrap break-all w-full max-w-[600px]">
            {JSON.stringify(event.payload, null, 2)}
          </pre>
        </CollapsibleContent>
      </div>
    </Collapsible>
  );
}
