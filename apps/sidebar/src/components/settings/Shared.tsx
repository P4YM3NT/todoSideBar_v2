import { ChevronLeft } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Row({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      <span className="text-sm font-medium">{label}</span>
      {hint && <p className="text-xs leading-snug text-muted-foreground">{hint}</p>}
      {children}
    </div>
  );
}

export function SubPage({
  title,
  onBack,
  children,
}: {
  title: string;
  onBack: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full flex-col gap-4">
      <header className="flex items-center gap-1">
        <Button variant="ghost" size="icon-sm" onClick={onBack} aria-label="Zurück">
          <ChevronLeft className="size-4" />
        </Button>
        <h1 className="text-sm font-medium">{title}</h1>
      </header>
      {/* overflow-x explizit aus: mit nur `overflow-y-auto` stuft CSS die
          andere Achse auf `auto` hoch und zeigt eine Scrollleiste. */}
      <div className="space-y-4 overflow-y-auto overflow-x-hidden">{children}</div>
    </div>
  );
}
