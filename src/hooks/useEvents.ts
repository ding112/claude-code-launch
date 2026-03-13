import { useState, useCallback } from "react";
import type { EventItem } from "../types";
import { fetchEvents } from "../api";

export function useEvents() {
  const [events, setEvents] = useState<EventItem[]>([]);

  const loadEvents = useCallback(async (
    sessionId: string,
    fromMs?: number,
    toMs?: number,
  ) => {
    const data = await fetchEvents(sessionId, { fromMs, toMs });
    setEvents(data.items);
  }, []);

  const clearEvents = () => {
    setEvents([]);
  };

  return {
    events,
    loadEvents,
    clearEvents,
  };
}
