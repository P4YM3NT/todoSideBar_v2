import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import ts from "typescript";

// Transpilieren statt Node-Type-Stripping: läuft auch mit dem unterstützten Node 20.
const source = await readFile(new URL("../src/lib/manualOrder.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
});
const {
  applyManualPositions,
  loadManualPositions,
  moveTask,
  positionsAfterMove,
  saveManualPositions,
} = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`);

const tasks = (...ids) => ids.map((id) => ({ id }));
const ids = (items) => items.map((item) => item.id);

test("nur gezogenes Todo überschreibt Sortierung, auch nach Wechsel der Sortierrichtung", () => {
  const base = tasks("a", "b", "c", "d");
  const moved = moveTask(base, "d", 1);
  const positions = positionsAfterMove(moved, {}, "d");
  assert.deepEqual(positions, { d: 1 });
  assert.deepEqual(ids(applyManualPositions(base, positions)), ["a", "d", "b", "c"]);
  assert.deepEqual(ids(applyManualPositions([...base].reverse(), positions)), ["c", "d", "b", "a"]);
  assert.deepEqual(ids(base), ["a", "b", "c", "d"]);
});

test("mehrfaches Verschieben zieht bestehende feste Positionen nach", () => {
  const base = tasks("a", "b", "c", "d", "e");
  let positions = {};
  for (const [id, index] of [["e", 1], ["a", 3], ["d", 0], ["e", 4]]) {
    const current = applyManualPositions(base, positions);
    const expected = moveTask(current, id, index);
    positions = positionsAfterMove(expected, positions, id);
    assert.deepEqual(ids(applyManualPositions(base, positions)), ids(expected));
  }
  assert.equal(Object.hasOwn(positions, "b"), false);
  assert.equal(Object.hasOwn(positions, "c"), false);
});

test("lange Folge von Bewegungen behält Zielposition und jedes Todo genau einmal", () => {
  const base = tasks(...Array.from({ length: 12 }, (_, i) => String(i)));
  let positions = {};
  for (let i = 0; i < 120; i++) {
    const current = applyManualPositions(base, positions);
    const id = base[(i * 7) % base.length].id;
    const index = (i * 5 + 3) % base.length;
    const expected = moveTask(current, id, index);
    positions = positionsAfterMove(expected, positions, id);
    const actual = applyManualPositions(base, positions);
    assert.deepEqual(ids(actual), ids(expected));
    assert.equal(new Set(ids(actual)).size, base.length);
  }
});

test("Filter versteckt feste Todos, beim Zurückkehren behalten sie ihre Position", () => {
  const positions = { b: 0, e: 4 };
  assert.deepEqual(ids(applyManualPositions(tasks("a", "c", "d"), positions)), ["a", "c", "d"]);
  assert.deepEqual(ids(applyManualPositions(tasks("a", "b", "c", "d", "e"), positions)), ["b", "a", "c", "d", "e"]);
});

test("verkürzte Liste und kollidierende Positionen verlieren kein Todo", () => {
  const positions = { b: 4, d: 4, e: 10 };
  assert.deepEqual(ids(applyManualPositions(tasks("b", "d", "e"), positions)), ["b", "d", "e"]);
  assert.deepEqual(ids(applyManualPositions(tasks("a", "b", "d", "e"), positions)), ["a", "b", "d", "e"]);
  assert.deepEqual(applyManualPositions([], positions), []);
});

test("neues Todo folgt automatischer Sortierung, feste Position bleibt", () => {
  const base = tasks("new", "a", "b", "c", "d");
  assert.deepEqual(ids(applyManualPositions(base, { d: 1 })), ["new", "d", "a", "b", "c"]);
});

test("einzelner Reset entfernt nur gewählte Ausnahme, gesamter Reset stellt Sortierung her", () => {
  const base = tasks("a", "b", "c", "d");
  const positions = { c: 0, d: 1 };
  delete positions.c;
  assert.deepEqual(ids(applyManualPositions(base, positions)), ["a", "d", "b", "c"]);
  assert.equal(applyManualPositions(base, {}), base);
});

test("Positionen nach Filter-Bewegung bewahren unsichtbare Ausnahmen", () => {
  const next = positionsAfterMove(tasks("c", "a"), { hidden: 5 }, "c");
  assert.deepEqual(next, { hidden: 5, c: 0 });
});

test("oberer und unterer Rand sowie unbekanntes Todo", () => {
  const base = tasks("a", "b", "c");
  assert.deepEqual(ids(moveTask(base, "c", -4)), ["c", "a", "b"]);
  assert.deepEqual(ids(moveTask(base, "a", 99)), ["b", "c", "a"]);
  assert.equal(moveTask(base, "missing", 1), base);
});

test("Neuladen und Reset speichern pro Ansicht unabhängig", () => {
  const storage = new Map();
  globalThis.localStorage = {
    getItem: (key) => storage.get(key) ?? null,
    setItem: (key, value) => storage.set(key, value),
    removeItem: (key) => storage.delete(key),
  };
  assert.equal(saveManualPositions("today", { a: 2 }), true);
  assert.equal(saveManualPositions("project:one", { b: 0 }), true);
  assert.deepEqual(loadManualPositions("today"), { a: 2 });
  assert.deepEqual(loadManualPositions("project:one"), { b: 0 });
  assert.deepEqual(loadManualPositions("inbox"), {});
  assert.equal(saveManualPositions("today", {}), true);
  assert.deepEqual(loadManualPositions("today"), {});
  assert.deepEqual(loadManualPositions("project:one"), { b: 0 });
});

test("ungültige Speicherwerte werden ignoriert", () => {
  for (const [raw, expected] of [
    ["{broken", {}], ["null", {}], ["[]", {}], ["42", {}],
    ['{"good":2,"negative":-1,"fraction":1.5,"string":"1","":0}', { good: 2 }],
  ]) {
    globalThis.localStorage = { getItem: () => raw };
    assert.deepEqual(loadManualPositions("today"), expected);
  }
});

test("gesperrter Speicher blockiert weder Laden noch Reset", () => {
  globalThis.localStorage = {
    getItem: () => { throw new Error("blocked"); },
    setItem: () => { throw new Error("full"); },
    removeItem: () => { throw new Error("blocked"); },
  };
  assert.deepEqual(loadManualPositions("today"), {});
  assert.equal(saveManualPositions("today", { a: 0 }), false);
  assert.equal(saveManualPositions("today", {}), false);
});
