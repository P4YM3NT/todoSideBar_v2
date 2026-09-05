import { useState } from "react";

import { pretty, toAccelerator } from "@/lib/hotkey";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type Props = {
  value: string;
  onChange: (accelerator: string) => Promise<void>;
};

export function HotkeyRecorder({ value, onChange }: Props) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="flex flex-col gap-1">
      <Button
        variant="outline"
        size="sm"
        onClick={() => {
          setError(null);
          setRecording(true);
        }}
        onBlur={() => setRecording(false)}
        onKeyDown={async (e) => {
          if (!recording) return;
          e.preventDefault();
          if (e.key === "Escape") {
            setRecording(false);
            return;
          }
          const accelerator = toAccelerator(e);
          if (!accelerator) return;
          try {
            await onChange(accelerator);
            setRecording(false);
          } catch (err) {
            setError(String(err));
          }
        }}
        className={cn(
          "w-full justify-start font-mono",
          recording && "ring-3 ring-ring/50",
        )}
      >
        {recording ? "Tasten drücken…" : pretty(value)}
      </Button>
      {error && <span className="text-xs text-destructive">{error}</span>}
    </div>
  );
}
