export type Tab = "sessions" | "settings" | "setup";

const TABS: { id: Tab; label: string }[] = [
  { id: "sessions", label: "Sessions" },
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
    <div className="flex gap-1 bg-white p-1 rounded-lg border border-black/6 shadow-sm">
      {TABS.map((t) => (
        <button
          key={t.id}
          className={`border-none bg-transparent text-gray-500 font-medium px-4 py-1.5 rounded-md transition-all duration-150 ${tab === t.id ? "!bg-black !text-white shadow-sm" : "hover:bg-gray-100 hover:text-gray-900"}`}
          type="button"
          onClick={() => onTabChange(t.id)}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}
