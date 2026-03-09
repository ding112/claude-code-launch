import { useState } from "react";

export default function AddCommandInput({ onAdd }: { onAdd: (command: string) => void }) {
  const [value, setValue] = useState("");

  return (
    <div className="flex items-center gap-2">
      <input
        className="flex-1 rounded-md border border-black/6 px-2 py-1.5 text-xs font-mono text-gray-700 bg-white transition-all duration-150 focus:outline-none focus:border-gray-400 focus:ring-2 focus:ring-black/5"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder="输入 command..."
        onKeyDown={(e) => {
          if (e.key === "Enter" && value.trim()) {
            onAdd(value);
            setValue("");
          }
        }}
      />
      <button
        type="button"
        className="rounded-md border border-black/6 px-2 py-1.5 text-xs font-sans text-gray-600 bg-white transition-all duration-150 cursor-pointer hover:bg-gray-50 shadow-none"
        onClick={() => {
          if (value.trim()) {
            onAdd(value);
            setValue("");
          }
        }}
      >
        +
      </button>
    </div>
  );
}
