import type { TaskFields } from "./tasks";

const pad = (n: number) => String(n).padStart(2, "0");
export function localDay(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function clockOf(instant: string | null): string {
  if (!instant) return "";
  const date = new Date(instant);
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function toInstant(day: string, clock: string): string {
  const date = new Date(`${day}T${clock}:00`);
  // Reject DST gaps instead of silently moving an entered 02:30 to 03:30.
  if (!/^\d{2}:\d{2}$/.test(clock) || !Number.isFinite(date.getTime()) ||
      localDay(date) !== day || clockOf(date.toISOString()) !== clock) {
    throw new Error("Diese Uhrzeit existiert an diesem Datum in deiner Zeitzone nicht.");
  }
  return date.toISOString();
}

export function timedSchedule(fields: TaskFields, from: string, to: string): TaskFields {
  const day = fields.scheduledOn ?? fields.dueOn;
  if (!day) throw new Error("Zuerst ein geplantes oder fälliges Datum auswählen.");
  if (!from || !to) throw new Error("Von und Bis müssen beide angegeben werden.");
  const keepOrParse = (old: string | null, date: string, time: string) =>
    old && localDay(new Date(old)) === date && clockOf(old) === time ? old : toInstant(date, time);
  const startsAt = keepOrParse(fields.startsAt, day, from);
  const endsAt = keepOrParse(fields.endsAt, fields.endsOn ?? day, to);
  if (new Date(endsAt) <= new Date(startsAt)) throw new Error("Bis muss nach Von liegen. Über Mitternacht: Enddatum auf Folgetag setzen.");
  return { ...fields, startsAt, endsAt };
}

export function defaultTimes(fields: TaskFields): TaskFields {
  const day = fields.scheduledOn ?? fields.dueOn;
  if (!day) throw new Error("Zuerst ein Datum auswählen.");
  const startsAt = toInstant(day, "09:00");
  const end = new Date(new Date(startsAt).getTime() + Math.max(1, fields.estimateMinutes ?? 60) * 60_000);
  return { ...fields, startsAt, endsAt: end.toISOString(), endsOn: localDay(end) === day ? null : localDay(end) };
}

/** Move local clocks and civil day span together when the preferred date changes. */
export function shiftSchedule(fields: TaskFields, scheduledOn: string | null, dueOn: string | null): TaskFields {
  const before = fields.scheduledOn ?? fields.dueOn;
  const after = scheduledOn ?? dueOn;
  const next = { ...fields, scheduledOn, dueOn };
  if (!after) return { ...next, startsAt: null, endsAt: null, endsOn: null };
  if (!before || before === after) return next;
  const civilNumber = (day: string) => Date.parse(`${day}T00:00:00Z`);
  const span = fields.endsOn ? (civilNumber(fields.endsOn) - civilNumber(before)) / 86_400_000 : 0;
  const endDay = new Date(civilNumber(after) + span * 86_400_000).toISOString().slice(0, 10);
  next.endsOn = endDay === after ? null : endDay;
  if (fields.startsAt && fields.endsAt) {
    next.startsAt = toInstant(after, clockOf(fields.startsAt));
    next.endsAt = toInstant(endDay, clockOf(fields.endsAt));
  }
  return next;
}

export function changeEndDay(fields: TaskFields, endsOn: string | null): TaskFields {
  const day = fields.scheduledOn ?? fields.dueOn;
  if (!day) throw new Error("Zuerst ein Datum auswählen.");
  const endDay = endsOn ?? day;
  if (endDay < day) throw new Error("Enddatum darf nicht vor dem Startdatum liegen.");
  const next = { ...fields, endsOn: endDay === day ? null : endDay };
  // Preserve original start instant, including an imported repeated DST hour.
  if (fields.endsAt) next.endsAt = toInstant(endDay, clockOf(fields.endsAt));
  return next;
}

export function scheduleLabel(task: Pick<TaskFields, "startsAt" | "endsAt" | "scheduledOn" | "dueOn" | "endsOn">): string | null {
  const day = task.scheduledOn ?? task.dueOn;
  const short = (date: string) => new Date(`${date}T00:00:00`).toLocaleDateString("de-DE", { day: "2-digit", month: "2-digit" });
  if (task.startsAt && task.endsAt) {
    const start = clockOf(task.startsAt), end = clockOf(task.endsAt);
    const prefix = task.scheduledOn && task.dueOn && task.scheduledOn !== task.dueOn ? `${short(task.scheduledOn)} ` : "";
    return `${prefix}${start}–${task.endsOn && task.endsOn !== day ? `${short(task.endsOn)} ` : ""}${end}`;
  }
  return task.endsOn && day && task.endsOn !== day ? `${short(day)}–${short(task.endsOn)} · Ganztägig` : null;
}
