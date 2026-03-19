import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function SourceBadge({ source }: { source: string }) {
  return (
    <Badge
      variant="secondary"
      className={cn(
        "text-[10px] px-1.5 py-0",
        source === "claude" && "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
        source === "cursor" && "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
        (source === "claude-code" || source === "discovery") && "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
      )}
    >
      {source}
    </Badge>
  );
}