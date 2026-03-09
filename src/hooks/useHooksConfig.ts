import { useState } from "react";
import type { HooksData } from "../types";
import { fetchHooks, saveHooksApi, initHooksApi } from "../api";

export function useHooksConfig() {
  const [hooksData, setHooksData] = useState<HooksData>({ events: {} });
  const [hooksLoading, setHooksLoading] = useState(false);
  const [hooksSaveMessage, setHooksSaveMessage] = useState<string>("");
  const [hooksExpandedEvents, setHooksExpandedEvents] = useState<Record<string, boolean>>({});

  const loadHooks = async () => {
    setHooksLoading(true);
    try {
      const data = await fetchHooks();
      if (data) setHooksData(data);
    } finally {
      setHooksLoading(false);
    }
  };

  const saveHooks = async () => {
    setHooksSaveMessage("");
    const ok = await saveHooksApi(hooksData);
    setHooksSaveMessage(ok ? "Hooks 已保存并生效。" : "Hooks 保存失败，请检查配置。");
  };

  const initHooks = async () => {
    setHooksSaveMessage("");
    const data = await initHooksApi();
    if (!data) {
      setHooksSaveMessage("Hooks 初始化失败。");
      return;
    }
    setHooksData({ events: data.events });
    setHooksSaveMessage(`初始化完成，新增 ${data.added_count} 条 hook。`);
  };

  const deleteHookItem = (eventName: string, blockIndex: number, hookIndex: number) => {
    setHooksData((prev) => {
      const events = { ...prev.events };
      const blocks = [...(events[eventName] ?? [])];
      if (!blocks[blockIndex]) return prev;
      const block = { ...blocks[blockIndex], hooks: [...blocks[blockIndex].hooks] };
      block.hooks.splice(hookIndex, 1);
      blocks[blockIndex] = block;
      if (block.hooks.length === 0) {
        blocks.splice(blockIndex, 1);
      }
      if (blocks.length === 0) {
        delete events[eventName];
      } else {
        events[eventName] = blocks;
      }
      return { events };
    });
  };

  const addHookEvent = (eventName: string) => {
    setHooksData((prev) => {
      if (prev.events[eventName]) return prev;
      return {
        events: {
          ...prev.events,
          [eventName]: [{ matcher: "*", hooks: [] }],
        },
      };
    });
    setHooksExpandedEvents((prev) => ({ ...prev, [eventName]: true }));
  };

  const addHookCommand = (eventName: string, blockIndex: number, command: string) => {
    if (!command.trim()) return;
    setHooksData((prev) => {
      const events = { ...prev.events };
      const blocks = [...(events[eventName] ?? [])];
      if (!blocks[blockIndex]) return prev;
      const block = { ...blocks[blockIndex], hooks: [...blocks[blockIndex].hooks] };
      block.hooks.push({ type: "command", command: command.trim() });
      blocks[blockIndex] = block;
      events[eventName] = blocks;
      return { events };
    });
  };

  return {
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
  };
}
