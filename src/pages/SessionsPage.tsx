import { useEffect, useRef } from "react";
import TranscriptView, { extractTranscriptTimeRange } from "../TranscriptView";
import Pager from "../components/Pager";
import { useSessions } from "../hooks/useSessions";
import { useEvents } from "../hooks/useEvents";
import { useEvaluations } from "../hooks/useEvaluations";
import { useTranscript } from "../hooks/useTranscript";
import { riskClass, formatTimestamp } from "../utils";
import { Card, CardHeader, CardTitle, CardAction, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@/components/ui/collapsible";
import { cn } from "@/lib/utils";

export default function SessionsPage() {
  const {
    selectedSessionId,
    setSelectedSessionId,
    selectedSession,
    groupedSessions,
    collapsedProjects,
    loading,
    archiving,
    sessionMessage,
    toggleProject,
    loadSessions,
    archiveSelectedSession,
  } = useSessions();

  const {
    events,
    loadEvents,
    clearEvents,
  } = useEvents();

  const {
    evaluations,
    loadEvaluations,
    gotoEvaluationPage,
    clearEvaluations,
  } = useEvaluations();

  const {
    transcript,
    transcriptLoading,
    transcriptLoadingMore,
    transcriptError,
    transcriptCollapsed,
    setTranscriptCollapsed,
    handleTranscriptScroll,
  } = useTranscript(selectedSessionId);

  const prevTimeRangeRef = useRef<string>("");

  useEffect(() => {
    void loadSessions();
  }, []);

  useEffect(() => {
    if (!selectedSessionId) return;
    void loadEvaluations(selectedSessionId, evaluations.page, evaluations.page_size);
  }, [selectedSessionId]);

  useEffect(() => {
    if (!selectedSessionId || !transcript?.items.length) return;
    const range = extractTranscriptTimeRange(transcript.items);
    const rangeKey = range ? `${range.minMs}-${range.maxMs}` : "";
    if (rangeKey && rangeKey !== prevTimeRangeRef.current) {
      prevTimeRangeRef.current = rangeKey;
      void loadEvents(selectedSessionId, range!.minMs, range!.maxMs);
    }
  }, [selectedSessionId, transcript]);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-6 flex-1 min-h-0 items-stretch p-6">
      <Card className="lg:overflow-y-auto min-w-0">
        <CardHeader>
          <CardTitle>Session 列表</CardTitle>
          <CardAction>
            <Button variant="outline" size="sm" onClick={() => void loadSessions()}>
              刷新
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          {loading ? <p className="text-muted-foreground">加载中...</p> : null}
          <div className="flex flex-col gap-5">
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
                              <span className="text-[15px] font-semibold truncate">{session.session_id}</span>
                              {session.agent_type && (
                                <Badge variant="secondary" className="w-fit text-[11px]">
                                  {session.agent_type}
                                </Badge>
                              )}
                              <span className="text-[13px] text-muted-foreground">Last Active: {formatTimestamp(session.last_active_at_ms)}</span>
                              <span className={cn("font-medium inline-block px-2 py-0.5 rounded-full text-xs mt-1 w-fit border", riskClass(session.latest_risk_level))}>
                                Risk: {session.latest_risk_level}
                              </span>
                              <span className="text-[13px] text-muted-foreground">Evaluations: {session.evaluation_count}</span>
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

      <Card className="lg:overflow-y-auto min-w-0 overflow-x-hidden">
        <CardHeader>
          <CardTitle>Session 详情</CardTitle>
          {selectedSession ? (
            <CardAction>
              <Button
                variant="outline"
                size="sm"
                className="hover:border-destructive/30 hover:text-destructive hover:bg-destructive/10"
                disabled={archiving}
                onClick={() => void archiveSelectedSession(clearEvents, clearEvaluations)}
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
              <div className="mb-6">
                <div className="flex items-center gap-2 mb-1">
                  <h3 className="text-2xl font-bold tracking-tight m-0">{selectedSession.project_name}</h3>
                  {selectedSession.agent_type && (
                    <Badge variant="secondary">
                      {selectedSession.agent_type}
                    </Badge>
                  )}
                </div>
                <p className="text-muted-foreground m-0">{selectedSession.session_id}</p>
              </div>

              <div className="flex flex-col gap-6 mt-6 items-stretch">
                <Collapsible
                  open={!transcriptCollapsed}
                  onOpenChange={(open) => setTranscriptCollapsed(!open)}
                >
                  <div className={cn("flex flex-col", transcriptCollapsed ? "gap-0" : "gap-3")}>
                    <div className="flex justify-between items-center gap-3">
                      <CollapsibleTrigger className="w-full flex items-center justify-start gap-1.5 p-0 border-none bg-transparent text-inherit shadow-none cursor-pointer text-left hover:shadow-none">
                        <h4 className="m-0 flex items-center gap-1.5">
                          <span className="text-[10px] transition-transform duration-150 opacity-70">{transcriptCollapsed ? "▶" : "▼"}</span>
                          Transcript
                        </h4>
                      </CollapsibleTrigger>
                      {transcript?.updated_at_ms ? (
                        <span className="text-xs text-muted-foreground font-mono whitespace-nowrap">
                          更新于 {formatTimestamp(transcript.updated_at_ms)}
                        </span>
                      ) : null}
                    </div>
                    <CollapsibleContent>
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
                          onScroll={handleTranscriptScroll}
                        />
                      ) : null}
                      {transcript?.imported_offset_bytes ? (
                        <p className="text-muted-foreground mt-2.5">
                          已导入偏移：{transcript.imported_offset_bytes} bytes
                        </p>
                      ) : null}
                    </CollapsibleContent>
                  </div>
                </Collapsible>

                <div className="min-w-0">
                  <h4>评估结果</h4>
                  <ul className="timeline list-none m-0 p-0 flex flex-col gap-4">
                    {evaluations.items.map((item) => (
                      <li key={item.evaluation_id} className="list-none">
                        <Card size="sm">
                          <CardContent className="flex flex-col gap-3">
                            <div className="flex justify-between items-center pb-3 border-b">
                              <Badge variant="outline" className="font-mono font-semibold">{item.status}</Badge>
                              <span className="text-xs text-muted-foreground font-mono">{formatTimestamp(item.created_at_ms)}</span>
                            </div>
                            <div className="flex flex-col gap-2 bg-muted px-4 py-3 rounded-md">
                              <div className="flex items-center gap-2 text-sm">
                                <span className="text-muted-foreground font-medium min-w-[80px]">Risk:</span>
                                <span className={cn("font-semibold inline-flex px-2 py-0.5 rounded text-xs uppercase", riskClass(item.risk_level))}>{item.risk_level}</span>
                                <span className="text-muted-foreground text-[13px]">({item.risk_category})</span>
                              </div>
                              <div className="flex items-center gap-2 text-sm">
                                <span className="text-muted-foreground font-medium min-w-[80px]">Efficiency:</span>
                                <span className="font-semibold">{item.efficiency_level}</span>
                              </div>
                            </div>
                            {item.suggestion && (
                              <div className="flex flex-col gap-1.5">
                                <div className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">Suggestion</div>
                                <p className="m-0 text-sm text-emerald-800 leading-relaxed p-3 bg-emerald-50 border border-emerald-200 rounded-md">{item.suggestion}</p>
                              </div>
                            )}
                            {item.error_message && (
                              <div className="flex flex-col gap-1.5">
                                <div className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">Error</div>
                                <pre className="m-0 overflow-auto max-h-[400px] bg-destructive/10 text-destructive rounded-md p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap break-all">{item.error_message}</pre>
                              </div>
                            )}
                          </CardContent>
                        </Card>
                      </li>
                    ))}
                  </ul>
                  <Pager
                    page={evaluations.page}
                    pageSize={evaluations.page_size}
                    total={evaluations.total}
                    onPageChange={(p) => gotoEvaluationPage(selectedSessionId, p)}
                  />
                </div>
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
