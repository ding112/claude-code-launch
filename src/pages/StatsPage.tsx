import { useEffect, useMemo, useState } from "react";
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { useTokenStats, type TimeRange } from "../hooks/useTokenStats";
import type { TokenSessionStat } from "../types";
import { formatTimestamp } from "../utils";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatDate(dateStr: string): string {
  const d = new Date(dateStr + "T00:00:00");
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

function KpiCard({
  title,
  value,
  subtitle,
}: {
  title: string;
  value: string | number;
  subtitle?: string;
}) {
  return (
    <Card>
      <CardContent className="pt-6">
        <p className="text-sm font-medium text-muted-foreground">{title}</p>
        <p className="text-3xl font-bold tracking-tight mt-1">{value}</p>
        {subtitle && (
          <p className="text-xs text-muted-foreground mt-1">{subtitle}</p>
        )}
      </CardContent>
    </Card>
  );
}

const TIME_RANGES: { id: TimeRange; label: string }[] = [
  { id: "7d", label: "7 天" },
  { id: "30d", label: "30 天" },
  { id: "90d", label: "90 天" },
  { id: "all", label: "全部" },
];

type ChartMode = "bar" | "line";
type SortField = "total_tokens" | "input_tokens" | "output_tokens" | "last_active_at_ms";

function tokenDisplay(n: number): string {
  return n > 0 ? formatTokens(n) : "—";
}

export default function StatsPage() {
  const {
    data,
    loading,
    timeRange,
    setTimeRange,
    sourceFilter,
    setSourceFilter,
    refresh,
  } = useTokenStats();

  const [chartMode, setChartMode] = useState<ChartMode>("bar");
  const [sortField, setSortField] = useState<SortField>("total_tokens");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const chartData = useMemo(
    () =>
      (data?.daily ?? []).map((d) => ({
        date: formatDate(d.date),
        input_tokens: d.input_tokens,
        output_tokens: d.output_tokens,
        session_count: d.session_count,
      })),
    [data],
  );

  const sortedSessions = useMemo(() => {
    if (!data) return [];
    return [...data.sessions].sort((a, b) => {
      let aVal: number;
      let bVal: number;
      switch (sortField) {
        case "total_tokens":
          aVal = a.input_tokens + a.output_tokens;
          bVal = b.input_tokens + b.output_tokens;
          break;
        case "input_tokens":
          aVal = a.input_tokens;
          bVal = b.input_tokens;
          break;
        case "output_tokens":
          aVal = a.output_tokens;
          bVal = b.output_tokens;
          break;
        default:
          aVal = a.last_active_at_ms;
          bVal = b.last_active_at_ms;
          break;
      }
      return sortDir === "desc" ? bVal - aVal : aVal - bVal;
    });
  }, [data, sortField, sortDir]);

  function handleSort(field: SortField) {
    if (sortField === field) {
      setSortDir((d) => (d === "desc" ? "asc" : "desc"));
    } else {
      setSortField(field);
      setSortDir("desc");
    }
  }

  if (loading && !data) {
    return (
      <section className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-4 size-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
          <p className="text-sm text-muted-foreground">加载统计数据...</p>
        </div>
      </section>
    );
  }

  return (
    <section className="flex-1 overflow-y-auto p-6 space-y-6">
      {/* Controls */}
      <div className="flex items-center justify-between flex-wrap gap-3">
        <h2 className="text-lg font-semibold tracking-tight">Token 用量统计</h2>
        <div className="flex items-center gap-3 flex-wrap">
          <div className="flex items-center gap-1">
            {TIME_RANGES.map((r) => (
              <Button
                key={r.id}
                variant={timeRange === r.id ? "default" : "outline"}
                size="sm"
                className="text-xs h-7 px-2.5"
                onClick={() => setTimeRange(r.id)}
              >
                {r.label}
              </Button>
            ))}
          </div>

          <Select
            value={sourceFilter ?? "all"}
            onValueChange={(val) =>
              setSourceFilter(val === "all" || !val ? undefined : val)
            }
          >
            <SelectTrigger size="sm">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="all">全部来源</SelectItem>
                <SelectItem value="claude-code">Claude Code</SelectItem>
                <SelectItem value="cursor">Cursor</SelectItem>
                <SelectItem value="event">Event</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>

          <div className="flex items-center gap-1">
            <Button
              variant={chartMode === "bar" ? "default" : "outline"}
              size="sm"
              className="text-xs h-7 px-2.5"
              onClick={() => setChartMode("bar")}
            >
              柱状图
            </Button>
            <Button
              variant={chartMode === "line" ? "default" : "outline"}
              size="sm"
              className="text-xs h-7 px-2.5"
              onClick={() => setChartMode("line")}
            >
              折线图
            </Button>
          </div>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="Input Tokens"
          value={formatTokens(data?.total_input_tokens ?? 0)}
          subtitle="总输入"
        />
        <KpiCard
          title="Output Tokens"
          value={formatTokens(data?.total_output_tokens ?? 0)}
          subtitle="总输出"
        />
        <KpiCard
          title="总 Token"
          value={formatTokens(
            (data?.total_input_tokens ?? 0) + (data?.total_output_tokens ?? 0),
          )}
          subtitle={`${data?.total_sessions ?? 0} 个会话`}
        />
        <KpiCard
          title="平均每会话"
          value={formatTokens(data?.avg_tokens_per_session ?? 0)}
          subtitle="input + output"
        />
      </div>

      {/* Chart */}
      {chartData.length > 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>Token 用量趋势（按天）</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              {chartMode === "bar" ? (
                <BarChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                  <XAxis
                    dataKey="date"
                    tick={{ fontSize: 12 }}
                    className="fill-muted-foreground"
                  />
                  <YAxis
                    tick={{ fontSize: 12 }}
                    className="fill-muted-foreground"
                    tickFormatter={(v: number) => formatTokens(v)}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "8px",
                      fontSize: 12,
                    }}
                    formatter={(value, name) => [
                      formatTokens(Number(value)),
                      name === "input_tokens" ? "Input" : "Output",
                    ]}
                  />
                  <Legend
                    formatter={(value: string) =>
                      value === "input_tokens" ? "Input Tokens" : "Output Tokens"
                    }
                  />
                  <Bar
                    dataKey="input_tokens"
                    stackId="tokens"
                    fill="hsl(var(--foreground))"
                    radius={[0, 0, 0, 0]}
                  />
                  <Bar
                    dataKey="output_tokens"
                    stackId="tokens"
                    fill="hsl(var(--foreground) / 0.35)"
                    radius={[4, 4, 0, 0]}
                  />
                </BarChart>
              ) : (
                <LineChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                  <XAxis
                    dataKey="date"
                    tick={{ fontSize: 12 }}
                    className="fill-muted-foreground"
                  />
                  <YAxis
                    tick={{ fontSize: 12 }}
                    className="fill-muted-foreground"
                    tickFormatter={(v: number) => formatTokens(v)}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "8px",
                      fontSize: 12,
                    }}
                    formatter={(value, name) => [
                      formatTokens(Number(value)),
                      name === "input_tokens" ? "Input" : "Output",
                    ]}
                  />
                  <Legend
                    formatter={(value: string) =>
                      value === "input_tokens" ? "Input Tokens" : "Output Tokens"
                    }
                  />
                  <Line
                    type="monotone"
                    dataKey="input_tokens"
                    stroke="hsl(var(--foreground))"
                    strokeWidth={2}
                    dot={false}
                    activeDot={{ r: 4 }}
                  />
                  <Line
                    type="monotone"
                    dataKey="output_tokens"
                    stroke="hsl(var(--foreground) / 0.4)"
                    strokeWidth={2}
                    dot={false}
                    activeDot={{ r: 4 }}
                  />
                </LineChart>
              )}
            </ResponsiveContainer>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-sm text-muted-foreground">
              所选时间范围内暂无数据
            </p>
          </CardContent>
        </Card>
      )}

      {/* Session Token Detail Table */}
      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>会话 Token 明细</CardTitle>
          <span className="text-xs text-muted-foreground">
            {sortedSessions.length} 个会话
          </span>
        </CardHeader>
        <CardContent className="p-0">
          {sortedSessions.length === 0 ? (
            <p className="text-sm text-muted-foreground px-6 py-8 text-center">
              暂无会话数据
            </p>
          ) : (
            <ScrollArea className="max-h-[420px]">
              <table className="w-full text-sm">
                <thead className="sticky top-0 bg-card border-b">
                  <tr className="text-left text-muted-foreground text-xs">
                    <th className="px-4 py-2.5 font-medium">会话</th>
                    <th className="px-4 py-2.5 font-medium">来源</th>
                    <SortHeader
                      label="Input"
                      field="input_tokens"
                      current={sortField}
                      dir={sortDir}
                      onSort={handleSort}
                    />
                    <SortHeader
                      label="Output"
                      field="output_tokens"
                      current={sortField}
                      dir={sortDir}
                      onSort={handleSort}
                    />
                    <SortHeader
                      label="总计"
                      field="total_tokens"
                      current={sortField}
                      dir={sortDir}
                      onSort={handleSort}
                    />
                    <SortHeader
                      label="时间"
                      field="last_active_at_ms"
                      current={sortField}
                      dir={sortDir}
                      onSort={handleSort}
                    />
                  </tr>
                </thead>
                <tbody>
                  {sortedSessions.map((s) => (
                    <SessionRow key={s.session_id} session={s} />
                  ))}
                </tbody>
              </table>
            </ScrollArea>
          )}
        </CardContent>
      </Card>
    </section>
  );
}

function SortHeader({
  label,
  field,
  current,
  dir,
  onSort,
}: {
  label: string;
  field: SortField;
  current: SortField;
  dir: "asc" | "desc";
  onSort: (f: SortField) => void;
}) {
  const active = current === field;
  return (
    <th
      className="px-4 py-2.5 font-medium cursor-pointer select-none hover:text-foreground transition-colors whitespace-nowrap"
      onClick={() => onSort(field)}
    >
      {label}
      {active && (
        <span className="ml-1 text-[10px]">{dir === "desc" ? "↓" : "↑"}</span>
      )}
    </th>
  );
}

function SessionRow({ session }: { session: TokenSessionStat }) {
  const total = session.input_tokens + session.output_tokens;
  return (
    <tr className="border-b last:border-b-0 hover:bg-muted/40 transition-colors">
      <td className="px-4 py-2.5 max-w-[260px]">
        <div className="truncate text-[13px] font-medium leading-snug">
          {session.first_prompt || session.session_id.slice(0, 12)}
        </div>
        <div className="text-[11px] text-muted-foreground truncate">
          {session.project_name}
        </div>
      </td>
      <td className="px-4 py-2.5">
        <Badge
          variant="secondary"
          className={cn(
            "text-[10px] px-1.5 py-0",
            session.source === "cursor" &&
              "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
            (session.source === "claude-code" || session.source === "discovery") &&
              "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
          )}
        >
          {session.agent_type || session.source}
        </Badge>
      </td>
      <td className="px-4 py-2.5 font-mono text-xs text-right">
        {tokenDisplay(session.input_tokens)}
      </td>
      <td className="px-4 py-2.5 font-mono text-xs text-right">
        {tokenDisplay(session.output_tokens)}
      </td>
      <td className="px-4 py-2.5 font-mono text-xs text-right font-semibold">
        {tokenDisplay(total)}
      </td>
      <td className="px-4 py-2.5 text-xs text-muted-foreground whitespace-nowrap">
        {formatTimestamp(session.last_active_at_ms)}
      </td>
    </tr>
  );
}
