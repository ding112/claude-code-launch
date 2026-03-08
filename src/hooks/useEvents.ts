import { useState } from "react";
import type { EventResponse } from "../types";
import { fetchEvents } from "../api";

export function useEvents() {
  const [events, setEvents] = useState<EventResponse>({
    items: [],
    total: 0,
    page: 1,
    page_size: 20,
  });
  const [eventTypeFilter, setEventTypeFilter] = useState<string>("");
  const [eventsCollapsed, setEventsCollapsed] = useState(true);

  const loadEvents = async (
    sessionId: string,
    page: number,
    pageSize: number,
    eventType: string,
  ) => {
    const data = await fetchEvents(sessionId, page, pageSize, eventType);
    setEvents(data);
  };

  const gotoEventPage = (sessionId: string, nextPage: number) => {
    if (!sessionId) return;
    void loadEvents(sessionId, nextPage, events.page_size, eventTypeFilter);
  };

  const clearEvents = () => {
    setEvents((prev) => ({ ...prev, items: [], total: 0, page: 1 }));
  };

  return {
    events,
    eventTypeFilter,
    setEventTypeFilter,
    eventsCollapsed,
    setEventsCollapsed,
    loadEvents,
    gotoEventPage,
    clearEvents,
  };
}
