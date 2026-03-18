import { useEffect, useCallback } from "react";
import TranscriptView from "../TranscriptView";
import type { TimeRange } from "../TranscriptView";
import { useSessions, type DateRangePreset } from "../hooks/useSessions";
import { useEvents } from "../hooks/useEvents";
import { useTranscript } from "../hooks/useTranscript";
import { formatTimestamp } from "../utils";
import { Card, CardHeader, CardTitle, CardAction, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@/components/ui/collapsible";
import { cn } from "@/lib/utils";

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

const SOURCE_OPTIONS: { value: string | undefined; label: string }[] = [
  { value: undefined, label: "全部" },
  { value: "claude-code", label: "Claude Code" },
  { value: "cursor", label: "Cursor" },
];

const DATE_OPTIONS: { value: DateRangePreset; label: string }[] = [
  { value: "7d", label: "近 7 天" },
  { value: "30d", label: "近 30 天" },
  { value: "all", label: "全部" },
];

export default function SessionsPage() {
  const {
    selectedSessionId,
    setSelectedSessionId,
    selectedSession,
    groupedSessions,
    collapsedProjects,
    loading,
    archiving,
    discovering,
    sessionMessage,
    sourceFilter,
    setSourceFilter,
    datePreset,
    setDatePreset,
    toggleProject,
    loadSessions,
    archiveSelectedSession,
    runDiscover,
  } = useSessions();

  const {
    events,
    loadEvents,
    clearEvents,
  } = useEvents();

  const {
    transcript,
    transcriptLoading,
    transcriptLoadingMore,
    transcriptError,
    skippedLines,
    handleTranscriptScroll,
  } = useTranscript(selectedSessionId);

  useEffect(() => {
    void loadSessions();
  }, []);

  useEffect(() => {
    clearEvents();
  }, [selectedSessionId, clearEvents]);

  const handleTimeRangeChange = useCallback((range: TimeRange | null) => {
    if (!selectedSessionId || !range) return;
    void loadEvents(selectedSessionId, range.minMs, range.maxMs);
  }, [selectedSessionId, loadEvents]);

  const handleSourceChange = (value: string | undefined) => {
    setSourceFilter(value);
    setTimeout(() => void loadSessions(), 0);
  };

  const handleDateChange = (value: DateRangePreset) => {
    setDatePreset(value);
    setTimeout(() => void loadSessions(), 0);
  };

  const sessionCount = Object.values(groupedSessions).reduce((sum, g) => sum + g.length, 0);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-6 flex-1 min-h-0 items-stretch p-6 overflow-hidden">
      <Card className="overflow-y-auto min-w-0">
        <CardHeader>
          <CardTitle>Session 列表</CardTitle>
          <CardAction>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={discovering}
                onClick={() => void runDiscover()}
              >
                {discovering ? "扫描中..." : "发现历史"}
              </Button>
              <Button variant="outline" size="sm" onClick={() => void loadSessions()}>
                刷新
              </Button>
            </div>
          </CardAction>
        </CardHeader>
        <CardContent>
          {/* Filter bar */}
          <div className="flex flex-col gap-2 mb-4">
            <div className="flex gap-1 flex-wrap">
              {SOURCE_OPTIONS.map((opt) => (
                <Button
                  key={opt.label}
                  variant={sourceFilter === opt.value ? "default" : "outline"}
                  size="sm"
                  className="text-xs h-7 px-2.5"
                  onClick={() => handleSourceChange(opt.value)}
                >
                  {opt.label}
                </Button>
              ))}
            </div>
            <div className="flex gap-1 flex-wrap">
              {DATE_OPTIONS.map((opt) => (
                <Button
                  key={opt.value}
                  variant={datePreset === opt.value ? "default" : "outline"}
                  size="sm"
                  className="text-xs h-7 px-2.5"
                  onClick={() => handleDateChange(opt.value)}
                >
                  {opt.label}
                </Button>
              ))}
            </div>
            <span className="text-[11px] text-muted-foreground">
              {loading ? "加载中..." : `${sessionCount} 个会话`}
            </span>
          </div>

          <div className="flex flex-col gap-5">
            {!loading && sessionCount === 0 && (
              <p className="text-sm text-muted-foreground">无匹配会话</p>
            )}
            {Object.entries(groupedSessions).map(([projectName, projectSessions]) => {
              const isCollapsed = collapsedProjects[projectName];
              return (
                <Collapsible
                  key={projectName}
                  open={!isCollapsed}
                  onOpenChange={() => toggleProject(projectName)}
                >
                  <div className="flex flex-col gap-3">
                    <CollapsibleTrigger className="text-[13px] font-semibold text-muted-foreground m-0 py-1.5 px-1 uppercase tracking-widest bg-transparent border-none flex items-center gap-2 w-full text-left cursor-pointer transition-colors shadow-none hover:text-foreground">
                      <span className="text-[10px] transition-transform duration-150 opacity-70">
                        {isCollapsed ? "▶" : "▼"}
                      </span>
                      {projectName}
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                      <ul className="session-list m-0 p-0 list-none flex flex-col gap-2 pl-2">
                        {projectSessions.map((session) => (
                          <li key={session.session_id}>
                            <button
                              type="button"
                              className={cn(
                                "w-full text-left flex flex-col gap-1 px-4 py-3 border rounded-lg bg-card transition-all duration-150 shadow-sm cursor-pointer overflow-hidden",
                                selectedSessionId === session.session_id
                                  ? "border-foreground ring-1 ring-foreground bg-muted"
                                  : "border-border hover:border-foreground/15 hover:-translate-y-px hover:shadow-md"
                              )}
                              onClick={() => setSelectedSessionId(session.session_id)}
                            >
                              <span className="text-[13px] font-semibold truncate leading-snug">
                                {session.first_prompt || session.session_id.slice(0, 12)}
                              </span>
                              {session.summary ? (
                                <span className="text-[12px] text-muted-foreground line-clamp-2 leading-snug">{session.summary}</span>
                              ) : null}
                              <div className="flex items-center gap-1.5 flex-wrap mt-0.5">
                                {session.agent_type && (
                                  <Badge
                                    variant="secondary"
                                    className={cn(
                                      "text-[10px] px-1.5 py-0",
                                      session.agent_type === "cursor" && "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
                                      session.agent_type === "claude-code" && "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
                                    )}
                                  >
                                    {session.agent_type}
                                  </Badge>
                                )}
                                {(session.source === "discovery" || session.source === "cursor-discovery") && (
                                  <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                                    历史
                                  </Badge>
                                )}
                                {session.duration_minutes > 0 && (
                                  <span className="text-[11px] text-muted-foreground">{session.duration_minutes}min</span>
                                )}
                                {(session.input_tokens > 0 || session.output_tokens > 0) && (
                                  <span className="text-[11px] text-muted-foreground font-mono">
                                    {formatTokens(session.input_tokens)}/{formatTokens(session.output_tokens)}
                                  </span>
                                )}
                              </div>
                              <span className="text-[12px] text-muted-foreground">{formatTimestamp(session.last_active_at_ms)}</span>
                            </button>
                          </li>
                        ))}
                      </ul>
                    </CollapsibleContent>
                  </div>
                </Collapsible>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card className="overflow-y-auto min-w-0 overflow-x-hidden">
        <CardHeader>
          <CardTitle>Session 详情</CardTitle>
          {selectedSession ? (
            <CardAction>
              <Button
                variant="outline"
                size="sm"
                className="hover:border-destructive/30 hover:text-destructive hover:bg-destructive/10"
                disabled={archiving}
                onClick={() => void archiveSelectedSession(clearEvents)}
              >
                {archiving ? "归档中..." : "归档"}
              </Button>
            </CardAction>
          ) : null}
        </CardHeader>
        <CardContent>
          {sessionMessage ? <p className="text-muted-foreground">{sessionMessage}</p> : null}

          {selectedSession ? (
            <>
              <div className="mb-6 flex flex-col gap-3">
                <div className="flex items-center justify-between gap-2">
                  <p className="text-muted-foreground m-0 text-sm font-mono">{selectedSession.session_id}</p>
                  {transcript?.updated_at_ms ? (
                    <span className="text-xs text-muted-foreground font-mono whitespace-nowrap">
                      更新于 {formatTimestamp(transcript.updated_at_ms)}
                    </span>
                  ) : null}
                </div>

                {selectedSession.goal && (
                  <div className="flex flex-col gap-1 bg-muted px-4 py-3 rounded-md">
                    <span className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">目标</span>
                    <p className="m-0 text-sm leading-relaxed">{selectedSession.goal}</p>
                  </div>
                )}

                {selectedSession.outcome && (
                  <div className="flex items-center gap-2 text-sm">
                    <span className="text-muted-foreground font-medium">结果:</span>
                    <Badge variant={selectedSession.outcome === "fully_achieved" ? "default" : "outline"}>
                      {selectedSession.outcome}
                    </Badge>
                  </div>
                )}

                {(selectedSession.duration_minutes > 0 || selectedSession.input_tokens > 0) && (
                  <div className="flex items-center gap-4 text-sm text-muted-foreground">
                    {selectedSession.duration_minutes > 0 && (
                      <span>时长: {selectedSession.duration_minutes} 分钟</span>
                    )}
                    {selectedSession.input_tokens > 0 && (
                      <span>输入: {formatTokens(selectedSession.input_tokens)} tokens</span>
                    )}
                    {selectedSession.output_tokens > 0 && (
                      <span>输出: {formatTokens(selectedSession.output_tokens)} tokens</span>
                    )}
                  </div>
                )}

                {selectedSession.summary && (
                  <div className="flex flex-col gap-1">
                    <span className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">摘要</span>
                    <p className="m-0 text-sm leading-relaxed text-muted-foreground">{selectedSession.summary}</p>
                  </div>
                )}
              </div>

              <div className="flex flex-col gap-3 mt-4 flex-1 min-h-0">
                {transcriptLoading ? <p className="text-muted-foreground">Transcript 加载中...</p> : null}
                {!transcriptLoading && transcriptError ? (
                  <p className="text-destructive">{transcriptError}</p>
                ) : null}
                {!transcriptLoading &&
                !transcriptError &&
                transcript?.last_error_message ? (
                  <p className="text-destructive">
                    最近同步错误：{transcript.last_error_message}
                  </p>
                ) : null}
                {!transcriptLoading &&
                !transcriptError &&
                (!transcript || transcript.items.length === 0) ? (
                  <p className="text-muted-foreground">暂无 transcript 内容</p>
                ) : null}
                {!transcriptLoading && transcript?.items.length ? (
                  <TranscriptView
                    items={transcript.items}
                    events={events}
                    loadingMore={transcriptLoadingMore}
                    hasMore={transcript.has_more}
                    skippedLines={skippedLines}
                    onScroll={handleTranscriptScroll}
                    onTimeRangeChange={handleTimeRangeChange}
                  />
                ) : null}
              </div>
            </>
          ) : (
            <p className="text-muted-foreground">请先选择一个 session。</p>
          )}
        </CardContent>
      </Card>
    </section>
  );
}
