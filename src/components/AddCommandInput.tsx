import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export default function AddCommandInput({ onAdd }: { onAdd: (command: string) => void }) {
  const [value, setValue] = useState("");

  const submit = () => {
    if (value.trim()) {
      onAdd(value);
      setValue("");
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Input
        className="flex-1 font-mono text-xs"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder="输入 command..."
        onKeyDown={(e) => {
          if (e.key === "Enter") submit();
        }}
      />
      <Button variant="outline" size="xs" onClick={submit}>
        +
      </Button>
    </div>
  );
}
