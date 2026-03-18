import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

export type Tab = "dashboard" | "sessions" | "config" | "stats" | "settings" | "setup";

const TABS: { id: Tab; label: string }[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "sessions", label: "Sessions" },
  { id: "config", label: "Config" },
  { id: "stats", label: "Stats" },
  { id: "settings", label: "Settings" },
  { id: "setup", label: "Setup" },
];

export default function TabSwitcher({
  tab,
  onTabChange,
}: {
  tab: Tab;
  onTabChange: (tab: Tab) => void;
}) {
  return (
    <Tabs value={tab} onValueChange={(v) => onTabChange(v as Tab)}>
      <TabsList>
        {TABS.map((t) => (
          <TabsTrigger key={t.id} value={t.id}>
            {t.label}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
