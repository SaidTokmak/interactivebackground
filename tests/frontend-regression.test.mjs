import test from "node:test";
import assert from "node:assert/strict";
import { calculateLayout, hasWidgetCollision, monitorLayoutViewport, widgetSizeLimits } from "../.regression-dist/widgetLayout.js";
import { clockHandAngles, defaultClockSettings, formatClock, formatClockDate } from "../.regression-dist/clockFormat.js";

const widget = (overrides = {}) => ({
  id: 1,
  monitorId: null,
  kind: "clock",
  x: 0.1,
  y: 0.1,
  width: 0.2,
  height: 0.2,
  locked: false,
  snapToGrid: true,
  visible: true,
  sortOrder: 0,
  clockSettings: null,
  ...overrides,
});

test("preview dönüşümü çözünürlük, DPI ve negatif ikinci monitör koordinatından bağımsızdır", () => {
  const cases = [
    [{ width: 1920, height: 1080, scaleFactor: 1 }, { width: 1920, height: 1080 }],
    [{ width: 2560, height: 1440, scaleFactor: 1.25 }, { width: 2048, height: 1152 }],
    [{ width: 3840, height: 2160, scaleFactor: 2 }, { width: 1920, height: 1080 }],
    [{ x: -3440, y: -280, width: 3440, height: 1440, scaleFactor: 1.5 }, { width: 2293.3333333333335, height: 960 }],
  ];
  for (const [monitor, expected] of cases) {
    const actual = monitorLayoutViewport({ id: "test", name: "test", x: 0, y: 0, isPrimary: false, ...monitor });
    assert.deepEqual(actual, expected);
  }
});

test("grid hareketi hassas adımlara oturur ve yüzey sınırını aşmaz", () => {
  const moved = calculateLayout(widget(), "move", 0.037, -0.3, { width: 1920, height: 1080 }, 0.01);
  assert.equal(moved.x, 0.14);
  assert.equal(moved.y, 0.015);
  const free = calculateLayout(widget(), "move", 0.037, 0.043, { width: 1920, height: 1080 }, 0);
  assert.equal(free.x, 0.137);
  assert.equal(free.y, 0.143);
});

test("resize monitör eşdeğerindeki minimum boyutu korur", () => {
  const viewport = { width: 1920, height: 1080 };
  const limits = widgetSizeLimits("clock", viewport);
  const resized = calculateLayout(widget(), "nw", 0.4, 0.4, viewport, 0);
  assert.ok(Math.abs(resized.width - limits.minWidth) < 0.000001);
  assert.ok(Math.abs(resized.height - limits.minHeight) < 0.000001);
});

test("görünür widget çarpışmaları algılanır, gizli widget engel olmaz", () => {
  const candidate = widget();
  assert.equal(hasWidgetCollision(candidate, [widget({ id: 2, x: 0.295 })]), true);
  assert.equal(hasWidgetCollision(candidate, [widget({ id: 2, x: 0.295, visible: false })]), false);
  assert.equal(hasWidgetCollision(candidate, [widget({ id: 2, x: 0.7 })]), false);
});

test("saat 12/24 formatı, tarih görünürlüğü, saat dilimi ve analog açıları uygular", () => {
  const instant = new Date("2026-07-17T13:05:30.000Z");
  const settings = { ...defaultClockSettings(), timeZone: "UTC", showSeconds: false, hourFormat: "hour24" };
  assert.match(formatClock(instant, "en", settings), /13:05/);
  assert.doesNotMatch(formatClock(instant, "en", settings), /30/);
  assert.equal(formatClockDate(instant, "en", { ...settings, showDate: false, showWeekday: false }), "");
  assert.deepEqual(clockHandAngles(instant, "UTC"), { hour: 32.5, minute: 33, second: 180 });
  assert.match(formatClock(instant, "en", { ...settings, hourFormat: "hour12" }), /01:05/);
});
