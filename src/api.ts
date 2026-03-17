import { API_BASE, TRANSCRIPT_PAGE_SIZE } from "./constants";
import type {
  SessionItem,
  DiscoverResult,
  EventResponse,
  EvaluationResponse,
  EvalSettings,
  HooksData,
  HooksInitResult,
  ScoredCommitsResponse,
  AiTrackingStats,
} from "./types";

async function assertOk(response: Response): Promise<Response> {
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new Error(`API error ${response.status}: ${text}`);
  }
  return response;
}

export async function fetchSessions(): Promise<SessionItem[]> {
  const response = await assertOk(await fetch(`${API_BASE}/sessions`));
  return (await response.json()) as SessionItem[];
}

export async function fetchEvents(
  sessionId: string,
  opts?: {
    page?: number;
    pageSize?: number;
    eventType?: string;
    fromMs?: number;
    toMs?: number;
  },
): Promise<EventResponse> {
  const { page = 1, pageSize = 200, eventType, fromMs, toMs } = opts ?? {};
  const params = new URLSearchParams({
    session_id: sessionId,
    page: String(page),
    page_size: String(pageSize),
  });
  if (eventType?.trim()) {
    params.set("event_type", eventType.trim());
  }
  if (fromMs != null) params.set("from_ms", String(fromMs));
  if (toMs != null) params.set("to_ms", String(toMs));
  const response = await assertOk(await fetch(`${API_BASE}/events?${params.toString()}`));
  return (await response.json()) as EventResponse;
}

export async function fetchEvaluations(
  sessionId: string,
  page: number,
  pageSize: number,
): Promise<EvaluationResponse> {
  const params = new URLSearchParams({
    session_id: sessionId,
    page: String(page),
    page_size: String(pageSize),
  });
  const response = await assertOk(await fetch(`${API_BASE}/evaluations?${params.toString()}`));
  return (await response.json()) as EvaluationResponse;
}

export async function fetchTranscript(
  sessionId: string,
  beforeLineNo?: number,
  signal?: AbortSignal,
): Promise<Response> {
  const params = new URLSearchParams({
    session_id: sessionId,
    page_size: String(TRANSCRIPT_PAGE_SIZE),
  });
  if (beforeLineNo !== undefined) {
    params.set("before_line_no", String(beforeLineNo));
  }
  return fetch(`${API_BASE}/transcripts?${params.toString()}`, { signal });
}

export async function fetchSettings(): Promise<EvalSettings> {
  const response = await assertOk(await fetch(`${API_BASE}/settings`));
  return (await response.json()) as EvalSettings;
}

export async function saveSettingsApi(settings: EvalSettings): Promise<boolean> {
  const response = await fetch(`${API_BASE}/settings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
  return response.ok;
}

export async function fetchHooks(): Promise<HooksData | null> {
  const response = await fetch(`${API_BASE}/hooks`);
  if (!response.ok) return null;
  return (await response.json()) as HooksData;
}

export async function saveHooksApi(data: HooksData): Promise<boolean> {
  const response = await fetch(`${API_BASE}/hooks`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  return response.ok;
}

export async function initHooksApi(): Promise<HooksInitResult | null> {
  const response = await fetch(`${API_BASE}/hooks/init`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });
  if (!response.ok) return null;
  return (await response.json()) as HooksInitResult;
}

export async function discoverSessions(): Promise<DiscoverResult> {
  const response = await assertOk(
    await fetch(`${API_BASE}/sessions/discover`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    }),
  );
  return (await response.json()) as DiscoverResult;
}

export async function archiveSession(sessionId: string): Promise<boolean> {
  const response = await fetch(`${API_BASE}/sessions/archive`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ session_id: sessionId }),
  });
  return response.ok;
}

export async function fetchAiTrackingCommits(
  page = 1,
  pageSize = 50,
): Promise<ScoredCommitsResponse> {
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  const response = await assertOk(
    await fetch(`${API_BASE}/cursor/ai-tracking/commits?${params.toString()}`),
  );
  return (await response.json()) as ScoredCommitsResponse;
}

export async function fetchAiTrackingStats(): Promise<AiTrackingStats> {
  const response = await assertOk(
    await fetch(`${API_BASE}/cursor/ai-tracking/stats`),
  );
  return (await response.json()) as AiTrackingStats;
}
