import { useState } from "react";
import { Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

const numbers = (count: number) => Array.from({ length: count }, (_, i) => String(i).padStart(2, "0"));
const hours = numbers(24);
const minutes = numbers(60);

type Props = {
  id: string;
  label: string;
  value: string;
  disabled?: boolean;
  invalid?: boolean;
  onChange: (value: string) => void;
};

/** Compact metadata chip; direct input and the picker live inside its popover. */
export function TimeField({ id, label, value, disabled, invalid, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [hour, setHour] = useState("09");
  const [minute, setMinute] = useState("00");
  const [draft, setDraft] = useState("09:00");

  return <Popover open={open} onOpenChange={(next) => {
        if (next) {
          const parts = (value || "09:00").split(":");
          setHour(parts[0]); setMinute(parts[1]);
          setDraft(value || "09:00");
        }
        setOpen(next);
      }}>
        <PopoverTrigger render={<Button id={id} variant="outline" size="sm" className="w-auto font-normal" disabled={disabled}
          aria-invalid={invalid} aria-label={`${label}: ${value || "Uhrzeit auswählen"}`} />}>
          <Clock className="size-3.5" />
          {label}{value && <span className="tabular-nums">{value}</span>}
        </PopoverTrigger>
        <PopoverContent align="start" className="w-48 space-y-3 p-3">
          <p className="text-sm font-medium">{label}: Uhrzeit</p>
          <Input id={`${id}-input`} type="time" step={60} value={draft} aria-label={`${label}: Uhrzeit eingeben`}
            className="h-8" onChange={(event) => {
              const next = event.target.value;
              setDraft(next);
              if (/^\d{2}:\d{2}$/.test(next)) {
                const [h, m] = next.split(":"); setHour(h); setMinute(m);
              }
            }} />
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor={`${id}-hour`} className="text-xs">Stunden</Label>
              <Select value={hour} onValueChange={(next) => {
                const h = next ?? "09"; setHour(h); setDraft(`${h}:${minute}`);
              }}>
                <SelectTrigger id={`${id}-hour`} className="h-8 w-full"><SelectValue /></SelectTrigger>
                <SelectContent>{hours.map((number) => <SelectItem key={number} value={number}>{number}</SelectItem>)}</SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor={`${id}-minute`} className="text-xs">Minuten</Label>
              <Select value={minute} onValueChange={(next) => {
                const m = next ?? "00"; setMinute(m); setDraft(`${hour}:${m}`);
              }}>
                <SelectTrigger id={`${id}-minute`} className="h-8 w-full"><SelectValue /></SelectTrigger>
                <SelectContent>{minutes.map((number) => <SelectItem key={number} value={number}>{number}</SelectItem>)}</SelectContent>
              </Select>
            </div>
          </div>
          <Button size="sm" className="w-full" disabled={!draft} onClick={() => { onChange(draft); setOpen(false); }}>Übernehmen</Button>
        </PopoverContent>
      </Popover>;
}
