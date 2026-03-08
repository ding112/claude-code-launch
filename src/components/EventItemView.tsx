import { useState } from "react";
import type { EventItem } from "../types";
import { formatTimestamp } from "../utils";

export default function EventItemView({ eventItem }: { eventItem: EventItem }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <li className="border border-black/6 rounded-lg p-5 bg-white shadow-sm transition-shadow duration-150 hover:shadow-md">
      <div
        className="flex justify-between items-center mb-4 pb-3 border-b border-black/6 cursor-pointer"
        onClick={() => setExpanded((prev) => !prev)}
      >
        <strong className="text-sm font-mono bg-gray-50 px-2 py-1 rounded-md border border-black/6 text-gray-900 font-semibold">
          <span className="text-[10px] transition-transform duration-150 opacity-70">{expanded ? "▼" : "▶"}</span>
          {" "}{eventItem.event_type}
        </strong>
        <span className="text-xs text-gray-400 font-mono">{formatTimestamp(eventItem.created_at_ms)}</span>
      </div>
      {expanded ? (
        <div className="flex flex-col gap-2">
          <div className="text-xs font-semibold uppercase tracking-widest text-gray-400">Payload</div>
          <pre className="m-0 overflow-auto max-h-[400px] bg-slate-50 text-gray-900 rounded-md p-4 font-mono text-xs border border-black/6 leading-relaxed whitespace-pre-wrap break-all">{JSON.stringify(eventItem.payload, null, 2)}</pre>
        </div>
      ) : null}
    </li>
  );
}
