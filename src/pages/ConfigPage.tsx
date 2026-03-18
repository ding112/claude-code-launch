import { useEffect } from "react";
import { useConfigs, type SourceFilter } from "../hooks/useConfigs";
import { useConfigContent } from "../hooks/useConfigContent";
import type { ConfigItem } from "../types";
import { Card, CardHeader, CardTitle, CardAction, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { SourceBadge } from "@/components/SourceBadge";
import { cn } from "@/lib/utils";

const SOURCE_FILTERS: { value: SourceFilter; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "claude", label: "Claude" },
  { value: "cursor", label: "Cursor" },
];

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

export default function ConfigPage() {
  const {
    loading,
    sourceFilter,
    changeFilter,
    selectedConfigId,
    setSelectedConfigId,
    loadConfigs,
    groupedConfigs,
    configs,
  } = useConfigs();

  const selectedConfig = configs.find((c) => c.id === selectedConfigId) ?? null;
  const { content, loading: contentLoading, error: contentError } = useConfigContent(selectedConfigId);

  useEffect(() => {
    void loadConfigs();
  }, []);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[340px_1fr] gap-6 flex-1 min-h-0 items-stretch p-6 overflow-hidden">
      {/* Left: Config list */}
      <Card className="overflow-y-auto min-w-0">
        <CardHeader>
          <CardTitle>配置项</CardTitle>
          <CardAction>
            <Button variant="outline" size="sm" onClick={() => void loadConfigs()}>
              刷新
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          {/* Source filter */}
          <div className="flex gap-1 mb-4">
            {SOURCE_FILTERS.map((f) => (
              <Button
                key={f.value}
                variant={sourceFilter === f.value ? "default" : "outline"}
                size="sm"
                className="text-xs"
                onClick={() => changeFilter(f.value)}
              >
                {f.label}
              </Button>
            ))}
            <span className="ml-auto text-xs text-muted-foreground self-center">
              {configs.length} 项
            </span>
          </div>

          {loading ? <p className="text-muted-foreground text-sm">扫描中...</p> : null}

          {!loading && configs.length === 0 ? (
            <p className="text-muted-foreground text-sm">未发现配置项</p>
          ) : null}

          <div className="flex flex-col gap-4">
            {groupedConfigs.map((group) => (
              <div key={group.label}>
                <p className="text-[13px] font-semibold text-muted-foreground uppercase tracking-widest mb-2 px-1">
                  {group.label}
                </p>
                <ul className="m-0 p-0 list-none flex flex-col gap-1.5">
                  {group.items.map((item) => (
                    <ConfigListItem
                      key={item.id}
                      item={item}
                      selected={selectedConfigId === item.id}
                      onSelect={() => setSelectedConfigId(item.id)}
                    />
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Right: Content detail */}
      <Card className="overflow-hidden min-w-0 flex flex-col">
        <CardHeader>
          <CardTitle>配置内容</CardTitle>
        </CardHeader>
        <CardContent className="flex-1 min-h-0 flex flex-col">
          {!selectedConfig ? (
            <p className="text-muted-foreground text-sm">请选择一个配置项查看内容。</p>
          ) : null}

          {selectedConfig && selectedConfig.status === "missing" ? (
            <div className="flex flex-col gap-2">
              <p className="text-sm font-medium">{selectedConfig.name}</p>
              <p className="text-xs text-muted-foreground font-mono break-all">{selectedConfig.file_path}</p>
              <p className="text-sm text-muted-foreground">文件不存在或为空。</p>
            </div>
          ) : null}

          {selectedConfig && selectedConfig.status === "active" ? (
            <div className="flex flex-col gap-3 flex-1 min-h-0">
              <div className="flex flex-col gap-1 shrink-0">
                <div className="flex items-center gap-2">
                  <p className="text-sm font-medium m-0">{selectedConfig.name}</p>
                  <SourceBadge source={selectedConfig.source} />
                  <CategoryBadge category={selectedConfig.category} />
                </div>
                <p className="text-xs text-muted-foreground font-mono break-all m-0">{selectedConfig.file_path}</p>
                {selectedConfig.size_bytes != null && (
                  <p className="text-xs text-muted-foreground m-0">{formatBytes(selectedConfig.size_bytes)}</p>
                )}
              </div>
              <Separator />
              <div className="flex-1 min-h-0">
                {contentLoading ? (
                  <p className="text-muted-foreground text-sm">加载中...</p>
                ) : contentError ? (
                  <p className="text-destructive text-sm">{contentError}</p>
                ) : content ? (
                  <ScrollArea className="h-full">
                    <pre className="text-xs font-mono whitespace-pre-wrap break-words p-4 bg-muted rounded-md m-0 leading-relaxed">
                      {content.content_type === "json"
                        ? tryFormatJson(content.content)
                        : content.content}
                    </pre>
                  </ScrollArea>
                ) : null}
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </section>
  );
}

function ConfigListItem({
  item,
  selected,
  onSelect,
}: {
  item: ConfigItem;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <li>
      <button
        type="button"
        className={cn(
          "w-full text-left flex flex-col gap-0.5 px-3 py-2 border rounded-lg bg-card transition-all duration-150 shadow-sm cursor-pointer overflow-hidden",
          selected
            ? "border-foreground ring-1 ring-foreground bg-muted"
            : "border-border hover:border-foreground/15 hover:-translate-y-px hover:shadow-md",
        )}
        onClick={onSelect}
      >
        <div className="flex items-center gap-1.5">
          <span
            className={cn(
              "size-2 rounded-full shrink-0",
              item.status === "active" ? "bg-green-500" : "bg-muted-foreground/30",
            )}
          />
          <span className="text-[13px] font-medium truncate">{item.name}</span>
        </div>
        <div className="flex items-center gap-1.5 flex-wrap pl-3.5">
          <SourceBadge source={item.source} />
          <CategoryBadge category={item.category} />
          <StatusBadge status={item.status} />
        </div>
      </button>
    </li>
  );
}

function CategoryBadge({ category }: { category: string }) {
  return (
    <Badge variant="outline" className="text-[10px] px-1.5 py-0">
      {category}
    </Badge>
  );
}

function StatusBadge({ status }: { status: string }) {
  return (
    <Badge
      variant="secondary"
      className={cn(
        "text-[10px] px-1.5 py-0",
        status === "active" && "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300",
        status === "missing" && "bg-muted text-muted-foreground",
      )}
    >
      {status}
    </Badge>
  );
}

function tryFormatJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}
