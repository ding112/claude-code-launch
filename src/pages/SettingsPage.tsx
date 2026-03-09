import { useEffect, useState } from "react";
import AddCommandInput from "../components/AddCommandInput";
import { useEvalSettings } from "../hooks/useEvalSettings";
import { useHooksConfig } from "../hooks/useHooksConfig";
import { KNOWN_EVENTS } from "../constants";
import { Card, CardHeader, CardTitle, CardAction, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectGroup, SelectItem } from "@/components/ui/select";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@/components/ui/collapsible";
import { Separator } from "@/components/ui/separator";

export default function SettingsPage() {
  const {
    settings,
    setSettings,
    saveMessage,
    loadSettings,
    saveSettings,
  } = useEvalSettings();

  const {
    hooksData,
    hooksLoading,
    hooksSaveMessage,
    hooksExpandedEvents,
    setHooksExpandedEvents,
    loadHooks,
    saveHooks,
    initHooks,
    deleteHookItem,
    addHookEvent,
    addHookCommand,
  } = useHooksConfig();

  const [addEventValue, setAddEventValue] = useState("");

  useEffect(() => {
    void loadSettings();
    void loadHooks();
  }, []);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[400px_1fr] gap-6 flex-1 min-h-0 items-stretch p-6">
      <Card className="lg:overflow-y-auto min-w-0">
        <CardHeader>
          <CardTitle>评估设置</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-5">
          <label className="flex flex-col gap-2 text-sm font-medium">
            Provider
            <Select
              value={settings.provider}
              onValueChange={(val) => {
                if (val) setSettings((prev) => ({ ...prev, provider: val }));
              }}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem value="openai">OpenAI</SelectItem>
                  <SelectItem value="anthropic">Anthropic</SelectItem>
                  <SelectItem value="ollama">Ollama</SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
          </label>
          <label className="flex flex-col gap-2 text-sm font-medium">
            Model
            <Input
              value={settings.model}
              onChange={(e) =>
                setSettings((prev) => ({ ...prev, model: e.target.value }))
              }
            />
          </label>
          <label className="flex flex-col gap-2 text-sm font-medium">
            Base URL
            <Input
              value={settings.base_url}
              onChange={(e) =>
                setSettings((prev) => ({ ...prev, base_url: e.target.value }))
              }
              placeholder="https://api.openai.com/v1"
            />
          </label>
          <label className="flex flex-col gap-2 text-sm font-medium">
            API Key
            <Input
              type="password"
              value={settings.api_key ?? ""}
              onChange={(e) =>
                setSettings((prev) => ({ ...prev, api_key: e.target.value }))
              }
              placeholder="可选（Ollama 可留空）"
            />
          </label>
          <label className="flex flex-col gap-2 text-sm font-medium">
            Timeout (ms)
            <Input
              type="number"
              min={500}
              max={120000}
              value={settings.timeout_ms}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  timeout_ms: Number(e.target.value || "8000"),
                }))
              }
            />
          </label>
          <label className="flex flex-col gap-2 text-sm font-medium">
            Sampling Rate
            <Input
              type="number"
              min={1}
              value={settings.sampling_rate}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  sampling_rate: Number(e.target.value || "1"),
                }))
              }
            />
          </label>
          <label className="flex items-center gap-2 text-sm font-medium select-none">
            <Checkbox
              checked={settings.enabled}
              onCheckedChange={(checked) =>
                setSettings((prev) => ({ ...prev, enabled: Boolean(checked) }))
              }
            />
            启用评估
          </label>
          <Button onClick={() => void saveSettings()}>保存设置</Button>
          {saveMessage ? <p className="text-muted-foreground text-sm">{saveMessage}</p> : null}
        </CardContent>
      </Card>

      <Card className="lg:overflow-y-auto min-w-0">
        <CardHeader>
          <CardTitle>Hooks 配置</CardTitle>
          <CardAction className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => void loadHooks()}>
              刷新
            </Button>
            <Button size="sm" onClick={() => void initHooks()}>
              一键初始化
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <p className="text-xs text-muted-foreground m-0">管理 ~/.claude/settings.json 中的 hooks 事件配置</p>

          {hooksLoading ? <p className="text-muted-foreground">加载中...</p> : null}

          {!hooksLoading && (
            <>
              <div className="flex flex-col gap-3">
                {Object.entries(hooksData.events).map(([eventName, blocks]) => {
                  const isExpanded = hooksExpandedEvents[eventName] ?? false;
                  return (
                    <Collapsible
                      key={eventName}
                      open={isExpanded}
                      onOpenChange={(open) =>
                        setHooksExpandedEvents((prev) => ({
                          ...prev,
                          [eventName]: open,
                        }))
                      }
                    >
                      <div className="rounded-lg border overflow-hidden">
                        <CollapsibleTrigger className="w-full flex items-center justify-between gap-2 px-4 py-3 bg-muted border-none text-left cursor-pointer text-sm font-semibold hover:bg-muted/80 transition-colors shadow-none">
                          <span className="flex items-center gap-1.5">
                            <span className="text-[10px] opacity-70">{isExpanded ? "▼" : "▶"}</span>
                            {eventName}
                          </span>
                          <span className="text-xs text-muted-foreground font-normal">
                            {blocks.reduce((sum, b) => sum + b.hooks.length, 0)} hook(s)
                          </span>
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                          <div className="px-4 py-3 flex flex-col gap-3">
                            {blocks.map((block, blockIndex) => (
                              <div key={blockIndex} className="flex flex-col gap-2">
                                <span className="text-xs text-muted-foreground">
                                  matcher: <code className="bg-muted px-1 py-0.5 rounded">{block.matcher}</code>
                                </span>
                                {block.hooks.map((hookItem, hookIndex) => (
                                  <div
                                    key={hookIndex}
                                    className="flex items-center gap-2 bg-muted px-3 py-2 rounded-md"
                                  >
                                    <code className="flex-1 text-xs break-all">{hookItem.command}</code>
                                    <Button
                                      variant="ghost"
                                      size="icon-xs"
                                      className="text-muted-foreground hover:text-destructive"
                                      title="删除"
                                      onClick={() => deleteHookItem(eventName, blockIndex, hookIndex)}
                                    >
                                      ✕
                                    </Button>
                                  </div>
                                ))}
                                <AddCommandInput
                                  onAdd={(cmd) => addHookCommand(eventName, blockIndex, cmd)}
                                />
                              </div>
                            ))}
                            {blocks.length === 0 && (
                              <p className="text-xs text-muted-foreground m-0">暂无 hook 条目</p>
                            )}
                          </div>
                        </CollapsibleContent>
                      </div>
                    </Collapsible>
                  );
                })}
                {Object.keys(hooksData.events).length === 0 && (
                  <p className="text-muted-foreground text-sm">暂无 hooks 配置，可点击「一键初始化」快速添加。</p>
                )}
              </div>

              <Separator />

              <div className="flex flex-col gap-3">
                <h4 className="text-sm font-semibold m-0">添加事件</h4>
                <div className="flex items-center gap-2">
                  <Select value={addEventValue} onValueChange={(val) => setAddEventValue(val ?? "")}>
                    <SelectTrigger className="flex-1">
                      <SelectValue placeholder="选择事件类型..." />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        {KNOWN_EVENTS.filter((e) => !hooksData.events[e]).map((eventName) => (
                          <SelectItem key={eventName} value={eventName}>{eventName}</SelectItem>
                        ))}
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                  <Button
                    variant="outline"
                    onClick={() => {
                      if (addEventValue) {
                        addHookEvent(addEventValue);
                        setAddEventValue("");
                      }
                    }}
                  >
                    添加
                  </Button>
                </div>
              </div>

              <div className="flex items-center gap-3">
                <Button onClick={() => void saveHooks()}>保存 Hooks</Button>
                {hooksSaveMessage ? <span className="text-sm text-muted-foreground">{hooksSaveMessage}</span> : null}
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </section>
  );
}
