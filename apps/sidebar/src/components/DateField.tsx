import { CalendarDays, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

/** `YYYY-MM-DD` ohne Zeitzonenverschiebung - `toISOString()` waere hier falsch. */
const toIso = (d: Date) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate(),
  ).padStart(2, "0")}`;

type Props = {
  value: string | null;
  placeholder: string;
  /** Fuer "Faellig" ein anderes Symbol als fuer "Geplant". */
  icon?: React.ComponentType<{ className?: string }>;
  onChange: (value: string | null) => void;
};

export function DateField({
  value,
  placeholder,
  icon: Icon = CalendarDays,
  onChange,
}: Props) {
  const selected = value ? new Date(`${value}T00:00:00`) : undefined;

  return (
    <div className="flex items-center gap-1">
      <Popover>
        <PopoverTrigger
          render={<Button variant="outline" size="sm" className="w-auto justify-start font-normal" />}
        >
          <Icon className="size-3.5" />
          {selected
            ? selected.toLocaleDateString("de-DE", {
                day: "2-digit",
                month: "2-digit",
                year: "numeric",
              })
            : placeholder}
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={selected}
            onSelect={(d) => onChange(d ? toIso(d) : null)}
            autoFocus
          />
        </PopoverContent>
      </Popover>

      {value && (
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={`${placeholder} entfernen`}
          onClick={() => onChange(null)}
          className="text-muted-foreground"
        >
          <X className="size-3" />
        </Button>
      )}
    </div>
  );
}
