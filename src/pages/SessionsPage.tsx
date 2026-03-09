import { useEffect } from "react";
import TranscriptView from "../TranscriptView";
import Pager from "../components/Pager";
import EventItemView from "../components/EventItemView";
import { useSessions } from "../hooks/useSessions";
import { useEvents } from "../hooks/useEvents";
import { useEvaluations } from "../hooks/useEvaluations";
import { useTranscript } from "../hooks/useTranscript";
import { riskClass, formatTimestamp } from "../utils";

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
    eventTypeFilter,
    setEventTypeFilter,
    eventsCollapsed,
    setEventsCollapsed,
    loadEvents,
    gotoEventPage,
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

  useEffect(() => {
    void loadSessions();
  }, []);

  useEffect(() => {
    if (!selectedSessionId) return;
    void loadEvents(selectedSessionId, events.page, events.page_size, eventTypeFilter);
    void loadEvaluations(selectedSessionId, evaluations.page, evaluations.page_size);
  }, [selectedSessionId, eventTypeFilter]);

  useEffect(() => {
    setEventsCollapsed(true);
  }, [selectedSessionId]);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-6 flex-1 min-h-0 items-stretch">
      <aside className="bg-white rounded-xl border border-black/6 p-6 shadow-md lg:overflow-y-auto min-w-0">
        <div className="flex justify-between items-center mb-5">
          <h2 className="text-base font-semibold m-0 tracking-tight">Session 列表</h2>
          <button
            type="button"
            className="bg-gray-50 border border-black/6 text-gray-500 px-3 py-1 text-[13px] rounded-md transition-all duration-150 hover:border-black/15 hover:text-gray-900 shadow-none"
            onClick={() => void loadSessions()}
          >
            刷新
          </button>
        </div>
        {loading ? <p>加载中...</p> : null}
        <div className="flex flex-col gap-5">
          {Object.entries(groupedSessions).map(([projectName, projectSessions]) => {
            const isCollapsed = collapsedProjects[projectName];
            return (
              <div key={projectName} className="flex flex-col gap-3">
                <button
                  type="button"
                  className="text-[13px] font-semibold text-gray-500 m-0 py-1.5 px-1 uppercase tracking-widest bg-none border-none flex items-center gap-2 w-full text-left cursor-pointer transition-colors duration-150 shadow-none hover:text-gray-900 hover:shadow-none"
                  onClick={() => toggleProject(projectName)}
                >
                  <span className="text-[10px] transition-transform duration-150 opacity-70">
                    {isCollapsed ? "▶" : "▼"}
                  </span>
                  {projectName}
                </button>
                {!isCollapsed && (
                  <ul className="session-list m-0 p-0 list-none flex flex-col gap-2 pl-2">
                    {projectSessions.map((session) => (
                      <li key={session.session_id}>
                        <button
                          type="button"
                          className={`w-full text-left flex flex-col gap-1 px-4 py-3 border rounded-lg bg-white transition-all duration-150 shadow-sm cursor-pointer overflow-hidden ${
                            selectedSessionId === session.session_id
                              ? "border-gray-900 bg-gray-50 ring-1 ring-gray-900"
                              : "border-black/6 hover:border-black/15 hover:-translate-y-px hover:shadow-md"
                          }`}
                          onClick={() => setSelectedSessionId(session.session_id)}
                        >
                          <span className="text-[15px] font-semibold text-gray-900 truncate">{session.session_id}</span>
                          {session.agent_type && (
                            <span className="inline-block px-1.5 py-0.5 rounded text-[11px] font-medium w-fit bg-gray-100 text-gray-600 border border-black/6">
                              {session.agent_type}
                            </span>
                          )}
                          <span className="text-[13px] text-gray-500">Last Active: {formatTimestamp(session.last_active_at_ms)}</span>
                          <span className={`font-medium inline-block px-2 py-0.5 rounded-full text-xs mt-1 w-fit border ${riskClass(session.latest_risk_level)}`}>
                            Risk: {session.latest_risk_level}
                          </span>
                          <span className="text-[13px] text-gray-500">Evaluations: {session.evaluation_count}</span>
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            );
          })}
        </div>
      </aside>

      <section className="bg-white rounded-xl border border-black/6 p-6 shadow-md lg:overflow-y-auto min-w-0 overflow-x-hidden">
        <div className="flex justify-between items-center mb-5">
          <div className="mb-0">
            <h2 className="text-base font-semibold m-0 tracking-tight">Session 详情</h2>
          </div>
          <div className="flex items-center gap-3">
            <label htmlFor="eventType" className="text-[13px] text-gray-500 m-0 font-medium">Filter</label>
            <input
              id="eventType"
              className="py-1.5 px-3 text-[13px] rounded-md w-[140px] border border-black/6 bg-gray-50 text-gray-900 transition-all duration-150 focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
              value={eventTypeFilter}
              onChange={(event) => setEventTypeFilter(event.target.value)}
              placeholder="e.g. tool.call"
            />
            {selectedSession ? (
              <button
                type="button"
                className="bg-gray-50 border border-black/6 text-gray-500 px-3 py-1.5 text-[13px] rounded-md transition-all duration-150 hover:border-red-300 hover:text-red-600 hover:bg-red-50 shadow-none disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={archiving}
                onClick={() => void archiveSelectedSession(clearEvents, clearEvaluations)}
              >
                {archiving ? "归档中..." : "归档"}
              </button>
            ) : null}
          </div>
        </div>
        {sessionMessage ? <p className="text-gray-500">{sessionMessage}</p> : null}

        {selectedSession ? (
          <>
            <div className="mb-6">
              <div className="flex items-center gap-2 mb-1">
                <h3 className="text-2xl font-bold tracking-tight m-0">{selectedSession.project_name}</h3>
                {selectedSession.agent_type && (
                  <span className="inline-block px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 border border-black/6">
                    {selectedSession.agent_type}
                  </span>
                )}
              </div>
              <p className="text-gray-500 m-0">{selectedSession.session_id}</p>
            </div>

            <div className="flex flex-col gap-6 mt-6 items-stretch">
              <aside className={`flex flex-col ${transcriptCollapsed ? "gap-0" : "gap-3"}`}>
                <div className="flex justify-between items-center gap-3">
                  <button
                    type="button"
                    className="w-full flex items-center justify-start gap-1.5 p-0 border-none bg-none text-inherit shadow-none cursor-pointer text-left hover:shadow-none"
                    onClick={() => setTranscriptCollapsed((prev) => !prev)}
                    aria-expanded={!transcriptCollapsed}
                  >
                    <h4 className="m-0 flex items-center gap-1.5">
                      <span className="text-[10px] transition-transform duration-150 opacity-70">{transcriptCollapsed ? "▶" : "▼"}</span>
                      Transcript
                    </h4>
                  </button>
                  {transcript?.updated_at_ms ? (
                    <span className="text-xs text-gray-400 font-mono whitespace-nowrap">
                      更新于 {formatTimestamp(transcript.updated_at_ms)}
                    </span>
                  ) : null}
                </div>
                {!transcriptCollapsed ? (
                  <>
                    {transcriptLoading ? <p className="text-gray-500">Transcript 加载中...</p> : null}
                    {!transcriptLoading && transcriptError ? (
                      <p className="text-red-800">{transcriptError}</p>
                    ) : null}
                    {!transcriptLoading &&
                    !transcriptError &&
                    transcript?.last_error_message ? (
                      <p className="text-red-800">
                        最近同步错误：{transcript.last_error_message}
                      </p>
                    ) : null}
                    {!transcriptLoading &&
                    !transcriptError &&
                    (!transcript || transcript.items.length === 0) ? (
                      <p className="text-gray-500">暂无 transcript 内容</p>
                    ) : null}
                    {!transcriptLoading && transcript?.items.length ? (
                      <TranscriptView
                        items={transcript.items}
                        loadingMore={transcriptLoadingMore}
                        hasMore={transcript.has_more}
                        onScroll={handleTranscriptScroll}
                      />
                    ) : null}
                    {transcript?.imported_offset_bytes ? (
                      <p className="text-gray-500 mt-2.5">
                        已导入偏移：{transcript.imported_offset_bytes} bytes
                      </p>
                    ) : null}
                  </>
                ) : null}
              </aside>

              <div className="flex flex-col gap-6 min-w-0">
                <div className="flex flex-col gap-3">
                  <button
                    type="button"
                    className="w-full flex items-center justify-between gap-3 p-0 border-none bg-none text-inherit cursor-pointer shadow-none text-left hover:shadow-none"
                    onClick={() => setEventsCollapsed((prev) => !prev)}
                    aria-expanded={!eventsCollapsed}
                  >
                    <h4 className="m-0 flex items-center gap-1.5">
                      <span className="text-[10px] transition-transform duration-150 opacity-70">{eventsCollapsed ? "▶" : "▼"}</span>
                      事件时间线
                    </h4>
                    <span className="text-gray-500">total: {events.total}</span>
                  </button>
                  {!eventsCollapsed ? (
                    <>
                      <ul className="timeline list-none m-0 p-0 flex flex-col gap-4">
                        {events.items.map((eventItem) => (
                          <EventItemView key={eventItem.event_id} eventItem={eventItem} />
                        ))}
                      </ul>
                      <Pager
                        page={events.page}
                        pageSize={events.page_size}
                        total={events.total}
                        onPageChange={(p) => gotoEventPage(selectedSessionId, p)}
                      />
                    </>
                  ) : null}
                </div>

                <div>
                  <h4>评估结果</h4>
                  <ul className="timeline list-none m-0 p-0 flex flex-col gap-4">
                    {evaluations.items.map((item) => (
                      <li key={item.evaluation_id} className="border border-black/6 rounded-lg p-5 bg-white shadow-sm transition-shadow duration-150 hover:shadow-md">
                        <div className="flex justify-between items-center mb-3 pb-3 border-b border-black/6">
                          <strong className="text-sm font-mono bg-gray-50 px-2 py-1 rounded-md border border-black/6 text-gray-900 font-semibold">{item.status}</strong>
                          <span className="text-xs text-gray-400 font-mono">{formatTimestamp(item.created_at_ms)}</span>
                        </div>
                        <div className="flex flex-col gap-2 mb-4 bg-gray-50 px-4 py-3 rounded-md border border-black/6">
                          <div className="flex items-center gap-2 text-sm">
                            <span className="text-gray-500 font-medium min-w-[80px]">Risk:</span>
                            <span className={`font-semibold inline-flex px-2 py-0.5 rounded text-xs uppercase ${riskClass(item.risk_level)}`}>{item.risk_level}</span>
                            <span className="text-gray-400 text-[13px]">({item.risk_category})</span>
                          </div>
                          <div className="flex items-center gap-2 text-sm">
                            <span className="text-gray-500 font-medium min-w-[80px]">Efficiency:</span>
                            <span className="font-semibold text-gray-900">{item.efficiency_level}</span>
                          </div>
                        </div>
                        {item.suggestion && (
                          <div className="flex flex-col gap-1.5 mt-4">
                            <div className="text-xs font-semibold uppercase tracking-widest text-gray-400">Suggestion</div>
                            <p className="m-0 text-sm text-green-800 leading-relaxed p-3 bg-green-50 border border-green-200 rounded-md">{item.suggestion}</p>
                          </div>
                        )}
                        {item.error_message && (
                          <div className="flex flex-col gap-1.5 mt-4">
                            <div className="text-xs font-semibold uppercase tracking-widest text-gray-400">Error</div>
                            <pre className="m-0 overflow-auto max-h-[400px] bg-red-50 text-red-800 rounded-md p-4 font-mono text-xs border border-red-300 leading-relaxed whitespace-pre-wrap break-all">{item.error_message}</pre>
                          </div>
                        )}
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
            </div>
          </>
        ) : (
          <p>请先选择一个 session。</p>
        )}
      </section>
    </section>
  );
}
