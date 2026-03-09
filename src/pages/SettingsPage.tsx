import { useEffect } from "react";
import AddCommandInput from "../components/AddCommandInput";
import { useEvalSettings } from "../hooks/useEvalSettings";
import { useHooksConfig } from "../hooks/useHooksConfig";
import { KNOWN_EVENTS } from "../constants";

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

  useEffect(() => {
    void loadSettings();
    void loadHooks();
  }, []);

  return (
    <section className="grid grid-cols-1 lg:grid-cols-[400px_1fr] gap-6 flex-1 min-h-0 items-stretch">
      <aside className="bg-white rounded-xl border border-black/6 p-6 shadow-md lg:overflow-y-auto min-w-0">
        <h2 className="text-base font-semibold m-0 mb-5 tracking-tight">评估设置</h2>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          Provider
          <select
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            value={settings.provider}
            onChange={(event) =>
              setSettings((prev) => ({ ...prev, provider: event.target.value }))
            }
          >
            <option value="openai">OpenAI</option>
            <option value="anthropic">Anthropic</option>
            <option value="ollama">Ollama</option>
          </select>
        </label>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          Model
          <input
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            value={settings.model}
            onChange={(event) =>
              setSettings((prev) => ({ ...prev, model: event.target.value }))
            }
          />
        </label>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          Base URL
          <input
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            value={settings.base_url}
            onChange={(event) =>
              setSettings((prev) => ({ ...prev, base_url: event.target.value }))
            }
            placeholder="https://api.openai.com/v1"
          />
        </label>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          API Key
          <input
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            type="password"
            value={settings.api_key ?? ""}
            onChange={(event) =>
              setSettings((prev) => ({ ...prev, api_key: event.target.value }))
            }
            placeholder="可选（Ollama 可留空）"
          />
        </label>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          Timeout (ms)
          <input
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            type="number"
            min={500}
            max={120000}
            value={settings.timeout_ms}
            onChange={(event) =>
              setSettings((prev) => ({
                ...prev,
                timeout_ms: Number(event.target.value || "8000"),
              }))
            }
          />
        </label>
        <label className="flex flex-col gap-2 mb-5 text-sm font-medium text-gray-900">
          Sampling Rate
          <input
            className="rounded-md border border-black/6 px-3 py-2.5 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 shadow-[inset_0_1px_2px_rgba(0,0,0,0.02)] focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
            type="number"
            min={1}
            value={settings.sampling_rate}
            onChange={(event) =>
              setSettings((prev) => ({
                ...prev,
                sampling_rate: Number(event.target.value || "1"),
              }))
            }
          />
        </label>
        <label className="flex flex-row items-center gap-2 mb-5 text-sm font-medium text-gray-900 select-none">
          <input
            className="w-4 h-4 accent-black"
            type="checkbox"
            checked={settings.enabled}
            onChange={(event) =>
              setSettings((prev) => ({ ...prev, enabled: event.target.checked }))
            }
          />
          启用评估
        </label>
        <button
          type="button"
          className="rounded-md border border-black/6 px-4 py-2.5 text-sm font-sans font-medium text-white bg-black transition-all duration-150 cursor-pointer shadow-sm hover:-translate-y-px hover:shadow-md active:translate-y-0 disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-gray-50 disabled:text-gray-500"
          onClick={() => void saveSettings()}
        >
          保存设置
        </button>
        {saveMessage ? <p className="text-gray-500">{saveMessage}</p> : null}
      </aside>

      <section className="bg-white rounded-xl border border-black/6 p-6 shadow-md lg:overflow-y-auto min-w-0">
        <div className="flex justify-between items-center mb-5">
          <h2 className="text-base font-semibold m-0 tracking-tight">Hooks 配置</h2>
          <div className="flex items-center gap-2">
            <button
              type="button"
              className="bg-gray-50 border border-black/6 text-gray-500 px-3 py-1 text-[13px] rounded-md transition-all duration-150 hover:border-black/15 hover:text-gray-900 shadow-none"
              onClick={() => void loadHooks()}
            >
              刷新
            </button>
            <button
              type="button"
              className="rounded-md border border-black/6 px-3 py-1 text-[13px] font-medium text-white bg-black transition-all duration-150 cursor-pointer shadow-sm hover:-translate-y-px hover:shadow-md active:translate-y-0"
              onClick={() => void initHooks()}
            >
              一键初始化
            </button>
          </div>
        </div>
        <p className="text-xs text-gray-400 m-0 mb-4">管理 ~/.claude/settings.json 中的 hooks 事件配置</p>

        {hooksLoading ? <p className="text-gray-500">加载中...</p> : null}

        {!hooksLoading && (
          <>
            <div className="flex flex-col gap-3 mb-5">
              {Object.entries(hooksData.events).map(([eventName, blocks]) => {
                const isExpanded = hooksExpandedEvents[eventName] ?? false;
                return (
                  <div key={eventName} className="border border-black/6 rounded-lg overflow-hidden">
                    <button
                      type="button"
                      className="w-full flex items-center justify-between gap-2 px-4 py-3 bg-gray-50 border-none text-left cursor-pointer text-sm font-semibold text-gray-900 hover:bg-gray-100 transition-colors duration-150 shadow-none hover:shadow-none"
                      onClick={() =>
                        setHooksExpandedEvents((prev) => ({
                          ...prev,
                          [eventName]: !isExpanded,
                        }))
                      }
                    >
                      <span className="flex items-center gap-1.5">
                        <span className="text-[10px] opacity-70">{isExpanded ? "▼" : "▶"}</span>
                        {eventName}
                      </span>
                      <span className="text-xs text-gray-400 font-normal">
                        {blocks.reduce((sum, b) => sum + b.hooks.length, 0)} hook(s)
                      </span>
                    </button>
                    {isExpanded && (
                      <div className="px-4 py-3 flex flex-col gap-3">
                        {blocks.map((block, blockIndex) => (
                          <div key={blockIndex} className="flex flex-col gap-2">
                            <span className="text-xs text-gray-400">
                              matcher: <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-600">{block.matcher}</code>
                            </span>
                            {block.hooks.map((hookItem, hookIndex) => (
                              <div
                                key={hookIndex}
                                className="flex items-center gap-2 bg-gray-50 px-3 py-2 rounded-md border border-black/6"
                              >
                                <code className="flex-1 text-xs text-gray-700 break-all">{hookItem.command}</code>
                                <button
                                  type="button"
                                  className="bg-none border-none text-gray-400 hover:text-red-500 cursor-pointer text-sm px-1 shadow-none hover:shadow-none"
                                  title="删除"
                                  onClick={() => deleteHookItem(eventName, blockIndex, hookIndex)}
                                >
                                  ✕
                                </button>
                              </div>
                            ))}
                            <AddCommandInput
                              onAdd={(cmd) => addHookCommand(eventName, blockIndex, cmd)}
                            />
                          </div>
                        ))}
                        {blocks.length === 0 && (
                          <p className="text-xs text-gray-400 m-0">暂无 hook 条目</p>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
              {Object.keys(hooksData.events).length === 0 && (
                <p className="text-gray-400 text-sm">暂无 hooks 配置，可点击「一键初始化」快速添加。</p>
              )}
            </div>

            <div className="flex flex-col gap-3 mb-5 pt-4 border-t border-black/6">
              <h4 className="text-sm font-semibold m-0 text-gray-700">添加事件</h4>
              <div className="flex items-center gap-2">
                <select
                  id="addEventSelect"
                  className="rounded-md border border-black/6 px-3 py-2 text-sm font-sans text-gray-900 bg-gray-50 transition-all duration-150 focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5 flex-1"
                  defaultValue=""
                >
                  <option value="" disabled>选择事件类型...</option>
                  {KNOWN_EVENTS.filter((e) => !hooksData.events[e]).map((eventName) => (
                    <option key={eventName} value={eventName}>{eventName}</option>
                  ))}
                </select>
                <button
                  type="button"
                  className="rounded-md border border-black/6 px-3 py-2 text-sm font-sans font-medium text-gray-700 bg-white transition-all duration-150 cursor-pointer hover:bg-gray-50 shadow-none"
                  onClick={() => {
                    const select = document.getElementById("addEventSelect") as HTMLSelectElement | null;
                    if (select?.value) {
                      addHookEvent(select.value);
                      select.value = "";
                    }
                  }}
                >
                  添加
                </button>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <button
                type="button"
                className="rounded-md border border-black/6 px-4 py-2.5 text-sm font-sans font-medium text-white bg-black transition-all duration-150 cursor-pointer shadow-sm hover:-translate-y-px hover:shadow-md active:translate-y-0"
                onClick={() => void saveHooks()}
              >
                保存 Hooks
              </button>
              {hooksSaveMessage ? <span className="text-sm text-gray-500">{hooksSaveMessage}</span> : null}
            </div>
          </>
        )}
      </section>
    </section>
  );
}
