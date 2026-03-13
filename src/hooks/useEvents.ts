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
    try {
      const data = await fetchEvents(sessionId, { fromMs, toMs });
      setEvents(data.items);
    } catch (error) {
      console.error("Failed to load events:", error);
    }
  }, []);

  const clearEvents = useCallback(() => {
    setEvents([]);
  }, []);

  return {
    events,
    loadEvents,
    clearEvents,
  };
}
