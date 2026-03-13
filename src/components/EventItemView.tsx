import type { EventItem } from "../types";
import { formatTimestamp } from "../utils";
import { Card, CardContent } from "@/components/ui/card";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { useState } from "react";

export default function EventItemView({ eventItem }: { eventItem: EventItem }) {
  const [open, setOpen] = useState(false);

  return (
    <li className="list-none">
      <Card size="sm">
        <Collapsible open={open} onOpenChange={setOpen}>
          <CardContent className="flex flex-col gap-3">
            <CollapsibleTrigger className="flex w-full cursor-pointer items-center justify-between border-none bg-transparent p-0 text-left shadow-none hover:shadow-none">
              <Badge variant="outline" className="font-mono font-semibold">
                <span className="text-[10px] opacity-70">{open ? "▼" : "▶"}</span>
                {eventItem.event_type}
              </Badge>
              <span className="text-xs text-muted-foreground font-mono">{formatTimestamp(eventItem.created_at_ms)}</span>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="flex flex-col gap-2">
                <div className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">Payload</div>
                <pre className="m-0 overflow-auto max-h-[400px] rounded-md bg-muted p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap break-all">
                  {JSON.stringify(eventItem.payload, null, 2)}
                </pre>
              </div>
            </CollapsibleContent>
          </CardContent>
        </Collapsible>
      </Card>
    </li>
  );
}
