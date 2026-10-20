import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

process.env.TZ = "Europe/Berlin";
const source = await readFile(new URL("../src/lib/schedule.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext } });
const { clockOf, timedSchedule, shiftSchedule, toInstant, defaultTimes, changeEndDay, scheduleLabel } =
  await import(`data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`);
const fields = (patch = {}) => ({ scheduledOn: "2026-09-28", dueOn: "2026-10-02", startsAt: null, endsAt: null, endsOn: null, estimateMinutes: null, ...patch });

test("absolute times survive overnight date changes and prefer planning over deadline", () => {
  const original = timedSchedule(fields({ endsOn: "2026-09-29" }), "23:30", "01:00");
  const moved = shiftSchedule(original, "2026-10-24", original.dueOn);
  assert.equal(moved.endsOn, "2026-10-25");
  assert.equal(clockOf(moved.startsAt), "23:30"); assert.equal(clockOf(moved.endsAt), "01:00");
  assert.deepEqual(shiftSchedule(original, original.scheduledOn, "2026-12-01"), { ...original, dueOn: "2026-12-01" });
  const fallback = shiftSchedule(original, null, "2026-10-02");
  assert.equal(fallback.endsOn, "2026-10-03"); assert.equal(clockOf(fallback.startsAt), "23:30");
  assert.deepEqual(shiftSchedule(original, null, null), { ...original, scheduledOn: null, dueOn: null, startsAt: null, endsAt: null, endsOn: null });
});

test("DST gap rejected; spring interval uses elapsed time; fold retains imported instant", () => {
  assert.throws(() => toInstant("2026-03-29", "02:30"), /existiert/);
  const spring = timedSchedule(fields({ scheduledOn: "2026-03-29" }), "01:30", "03:30");
  assert.equal(Date.parse(spring.endsAt) - Date.parse(spring.startsAt), 60 * 60_000);
  const folded = fields({ scheduledOn: "2026-10-25", startsAt: "2026-10-25T00:30:00Z", endsAt: "2026-10-25T01:30:00Z" });
  assert.deepEqual(timedSchedule(folded, "02:30", "02:30"), folded);
});

test("date movement rejects a clock landing in DST gap", () => {
  const original = timedSchedule(fields({ scheduledOn: "2026-03-28" }), "02:30", "03:30");
  assert.throws(() => shiftSchedule(original, "2026-03-29", original.dueOn), /existiert/);
});

test("incomplete or backwards intervals cannot be saved", () => {
  assert.throws(() => timedSchedule(fields(), "09:00", ""), /beide/);
  assert.throws(() => timedSchedule(fields(), "09:00", "08:00"), /nach Von/);
  assert.throws(() => changeEndDay(fields(), "2026-09-27"), /vor dem/);
  assert.throws(() => defaultTimes(fields({ scheduledOn: null, dueOn: null })), /Datum/);
});

test("defaults follow estimate; long intervals get explicit end day and clear labels", () => {
  const schedule = defaultTimes(fields({ estimateMinutes: 18 * 60 }));
  assert.equal(clockOf(schedule.startsAt), "09:00"); assert.equal(clockOf(schedule.endsAt), "03:00");
  assert.equal(schedule.endsOn, "2026-09-29");
  assert.match(scheduleLabel(schedule), /28\.09.*09:00.*29\.09.*03:00/);
  assert.match(scheduleLabel(fields({ endsOn: "2026-09-30" })), /Ganztägig/);
  assert.equal(scheduleLabel(fields()), null);
});
