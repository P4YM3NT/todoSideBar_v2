import { useEffect, useState } from "react";
import { CalendarDays, Clock } from "lucide-react";
import type { TaskFields } from "@/lib/tasks";
import { changeEndDay, clockOf, defaultTimes, timedSchedule } from "@/lib/schedule";
import { DateField } from "@/components/DateField";
import { TimeField } from "@/components/TimeField";
import { Button } from "@/components/ui/button";

type Props = { fields: TaskFields; onChange: (fields: TaskFields) => void; onError: (error: string | null) => void };

export function ScheduleFields({ fields, onChange, onError }: Props) {
  const day = fields.scheduledOn ?? fields.dueOn;
  const [timed, setTimed] = useState(Boolean(fields.startsAt));
  const [from, setFrom] = useState(clockOf(fields.startsAt));
  const [to, setTo] = useState(clockOf(fields.endsAt));
  const [error, setError] = useState<string | null>(null);
  const fail = (value: string | null) => { setError(value); onError(value); };
  useEffect(() => {
    setTimed(Boolean(fields.startsAt));
    setFrom(clockOf(fields.startsAt)); setTo(clockOf(fields.endsAt));
    setError(null); onError(null);
  }, [fields.startsAt, fields.endsAt, fields.endsOn, day, onError]);

  const apply = (change: () => TaskFields) => {
    try { onChange(change()); fail(null); } catch (e) { fail((e as Error).message); }
  };
  const edit = (start: string, end: string) => {
    setFrom(start); setTo(end);
    apply(() => timedSchedule(fields, start, end));
  };

  const ModeIcon = timed ? Clock : CalendarDays;

  // Same metadata row and Button size as Geplant/Fällig. Editing stays in popovers.
  return <>
    <Button variant="outline" size="sm" className="w-auto font-normal" disabled={!day}
      aria-label={timed ? "Auf Ganztag umstellen" : "Uhrzeiten aktivieren"} aria-pressed={!timed}
      title={!day ? "Zuerst Geplant oder Fällig setzen" : timed ? "Auf Ganztag umstellen" : "Uhrzeiten aktivieren"}
      onClick={() => {
        if (timed) { setTimed(false); apply(() => ({ ...fields, startsAt: null, endsAt: null })); }
        else { apply(() => defaultTimes(fields)); }
      }}>
      <ModeIcon className="size-3.5" />{timed ? "Uhrzeit" : "Ganztägig"}
    </Button>
    <TimeField id="task-from" label="Von" value={from} disabled={!day || !timed} invalid={Boolean(error)} onChange={(value) => edit(value, to)} />
    <TimeField id="task-to" label="Bis" value={to} disabled={!day || !timed} invalid={Boolean(error)} onChange={(value) => edit(from, value)} />
    {day && <DateField value={fields.endsOn} placeholder="Enddatum" onChange={(end) => apply(() => {
      const next = changeEndDay({ ...fields, endsAt: null }, end);
      return timed ? timedSchedule(next, from, to) : next;
    })} />}
    {error && <p role="alert" className="basis-full text-xs text-destructive">{error}</p>}
  </>;
}
