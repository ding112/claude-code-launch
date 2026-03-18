import { useEffect } from "react";
import {
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Area,
  ComposedChart,
} from "recharts";
import { useDashboard } from "../hooks/useDashboard";
import type { Tab } from "../components/TabSwitcher";
import type { SessionItem } from "../types";
import { formatTimestamp } from "../utils";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
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

function SessionCard({ session }: { session: SessionItem }) {
  return (
    <div className="flex flex-col gap-1 px-4 py-3 border rounded-lg bg-card transition-all duration-150 shadow-sm">
      <span className="text-[13px] font-semibold truncate leading-snug">
        {session.first_prompt || session.session_id.slice(0, 12)}
      </span>
      {session.summary ? (
        <span className="text-[12px] text-muted-foreground line-clamp-1 leading-snug">
          {session.summary}
        </span>
      ) : null}
      <div className="flex items-center gap-1.5 flex-wrap mt-0.5">
        {session.agent_type && (
          <Badge
            variant="secondary"
            className={cn(
              "text-[10px] px-1.5 py-0",
              session.agent_type === "cursor" &&
                "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300",
              session.agent_type === "claude-code" &&
                "bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300",
            )}
          >
            {session.agent_type}
          </Badge>
        )}
        <Badge variant="outline" className="text-[10px] px-1.5 py-0">
          {session.project_name}
        </Badge>
        {(session.input_tokens > 0 || session.output_tokens > 0) && (
          <span className="text-[11px] text-muted-foreground font-mono">
            {formatTokens(session.input_tokens)}/{formatTokens(session.output_tokens)}
          </span>
        )}
        <span className="text-[11px] text-muted-foreground ml-auto">
          {formatTimestamp(session.last_active_at_ms)}
        </span>
      </div>
    </div>
  );
}

export default function DashboardPage({
  onNavigate,
}: {
  onNavigate: (tab: Tab) => void;
}) {
  const { loading, stats, recentSessions, dailyActivity, configSummary, refresh } =
    useDashboard();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (loading && !stats) {
    return (
      <section className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <div className="mx-auto mb-4 size-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
          <p className="text-sm text-muted-foreground">加载仪表盘数据...</p>
        </div>
      </section>
    );
  }

  const chartData = dailyActivity.map((d) => ({
    date: formatDate(d.date),
    sessions: d.session_count,
    tokens: d.input_tokens + d.output_tokens,
  }));

  const configHealthPct =
    stats && stats.configHealth.total > 0
      ? Math.round((stats.configHealth.active / stats.configHealth.total) * 100)
      : 0;

  return (
    <section className="flex-1 overflow-y-auto p-6 space-y-6">
      {/* KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <KpiCard
          title="今日会话"
          value={stats?.todaySessions ?? 0}
          subtitle="sessions today"
        />
        <KpiCard
          title="总 Token 用量"
          value={formatTokens(stats?.totalTokens ?? 0)}
          subtitle="input + output"
        />
        <KpiCard
          title="活跃项目"
          value={stats?.activeProjects ?? 0}
          subtitle="active projects"
        />
        <KpiCard
          title="配置健康度"
          value={`${stats?.configHealth.active ?? 0}/${stats?.configHealth.total ?? 0}`}
          subtitle={`${configHealthPct}% active`}
        />
      </div>

      {/* Activity Chart */}
      {chartData.length > 1 && (
        <Card>
          <CardHeader>
            <CardTitle>近 30 天活动趋势</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={260}>
              <ComposedChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis
                  dataKey="date"
                  tick={{ fontSize: 12 }}
                  className="fill-muted-foreground"
                />
                <YAxis
                  yAxisId="left"
                  tick={{ fontSize: 12 }}
                  className="fill-muted-foreground"
                  allowDecimals={false}
                />
                <YAxis
                  yAxisId="right"
                  orientation="right"
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
                    name === "tokens" ? formatTokens(Number(value)) : value,
                    name === "sessions" ? "会话数" : "Token 用量",
                  ]}
                />
                <Line
                  yAxisId="left"
                  type="monotone"
                  dataKey="sessions"
                  stroke="hsl(var(--foreground))"
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                />
                <Area
                  yAxisId="right"
                  type="monotone"
                  dataKey="tokens"
                  fill="hsl(var(--foreground) / 0.08)"
                  stroke="hsl(var(--foreground) / 0.3)"
                  strokeWidth={1}
                />
              </ComposedChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      )}

      {/* Bottom section: Recent Sessions + Config Summary */}
      <div className="grid grid-cols-1 lg:grid-cols-[1fr_340px] gap-6">
        {/* Recent Sessions */}
        <Card>
          <CardHeader className="flex-row items-center justify-between">
            <CardTitle>最近会话</CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onNavigate("sessions")}
            >
              查看全部
            </Button>
          </CardHeader>
          <CardContent>
            {recentSessions.length === 0 ? (
              <p className="text-sm text-muted-foreground">暂无会话数据</p>
            ) : (
              <div className="flex flex-col gap-2">
                {recentSessions.map((session) => (
                  <SessionCard key={session.session_id} session={session} />
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Config Summary */}
        <Card>
          <CardHeader className="flex-row items-center justify-between">
            <CardTitle>配置状态</CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onNavigate("config")}
            >
              查看详情
            </Button>
          </CardHeader>
          <CardContent>
            {configSummary ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">已发现项目</span>
                  <span className="text-sm font-semibold">{configSummary.projectCount}</span>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">总配置项</span>
                    <span className="font-semibold">{configSummary.total}</span>
                  </div>
                  <div className="w-full h-2 bg-muted rounded-full overflow-hidden">
                    <div
                      className="h-full bg-foreground rounded-full transition-all"
                      style={{
                        width: configSummary.total > 0
                          ? `${(configSummary.active / configSummary.total) * 100}%`
                          : "0%",
                      }}
                    />
                  </div>
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span>{configSummary.active} active</span>
                    <span>{configSummary.missing} missing</span>
                  </div>
                </div>

                <div className="border-t pt-3 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Badge
                        variant="secondary"
                        className="text-[10px] px-1.5 py-0 bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300"
                      >
                        Claude
                      </Badge>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {configSummary.bySource.claude.active} active / {configSummary.bySource.claude.missing} missing
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Badge
                        variant="secondary"
                        className="text-[10px] px-1.5 py-0 bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300"
                      >
                        Cursor
                      </Badge>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {configSummary.bySource.cursor.active} active / {configSummary.bySource.cursor.missing} missing
                    </span>
                  </div>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">加载中...</p>
            )}
          </CardContent>
        </Card>
      </div>
    </section>
  );
}
