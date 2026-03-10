import type { LogEvent } from "../types";
import { Card, CardHeader, CardTitle, CardAction, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { useMemo } from "react";

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString();
}

const ANSI_ESCAPE_PATTERN =
  /(?:\x1B\[[0-?]*[ -/]*[@-~])|(?:\x1B\][^\x07\x1B]*(?:\x07|\x1B\\))/g;
const CONTROL_CHAR_PATTERN = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g;

function normalizeCarriageReturn(input: string) {
  return input
    .split("\n")
    .map((line) => line.split("\r").pop() ?? "")
    .join("\n");
}

function sanitizeForDisplay(input: string) {
  return normalizeCarriageReturn(input)
    .replace(ANSI_ESCAPE_PATTERN, "")
    .replace(CONTROL_CHAR_PATTERN, "");
}

export function LogPanel({ logs }: { logs: LogEvent[] }) {
  const allText = useMemo(
    () => logs
      .map((item) => {
        const rawMessage = item.raw && item.raw.length > 0 ? item.raw : item.message;
        return `[${formatTime(item.timestamp)}] [${item.level}] ${rawMessage}`;
      })
      .join("\n"),
    [logs]
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle>实时日志</CardTitle>
        <CardAction>
          <Button
            variant="outline"
            size="xs"
            onClick={() => navigator.clipboard.writeText(allText)}
          >
            复制日志
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent>
        <ScrollArea className="max-h-72 rounded-md border bg-muted p-3 font-mono text-xs">
          {logs.length === 0 ? (
            <p className="text-muted-foreground">暂无日志，执行检测/安装/验证后会显示输出。</p>
          ) : (
            logs.map((item, idx) => (
              <p
                key={`${item.timestamp}-${idx}`}
                className="whitespace-pre-wrap break-all py-0.5"
                title={item.raw ?? undefined}
              >
                <span className="mr-2 text-muted-foreground">[{formatTime(item.timestamp)}]</span>
                <span
                  className={
                    item.level === "error"
                      ? "text-destructive"
                      : item.level === "warn"
                        ? "text-amber-600"
                        : "text-emerald-600"
                  }
                >
                  {sanitizeForDisplay(item.message)}
                </span>
              </p>
            ))
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
