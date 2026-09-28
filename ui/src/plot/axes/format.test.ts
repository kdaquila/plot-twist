import { describe, expect, it } from "vitest";
import { formatDateTime, formatExact, needsScientific, numericTicks, timeTicks } from "./format";

// 2026-09-27T00:00:00Z
const DAY = 1790467200;

describe("numeric labels (FR-009h)", () => {
  it("switch to scientific notation at |v| >= 1e6 or < 1e-3, never for zero", () => {
    expect(needsScientific([999_999])).toBe(false);
    expect(needsScientific([1e6])).toBe(true);
    expect(needsScientific([-2e6])).toBe(true);
    expect(needsScientific([0.001])).toBe(false);
    expect(needsScientific([0.0009])).toBe(true);
    expect(needsScientific([0])).toBe(false);
  });

  it("label large and tiny ranges in scientific notation", () => {
    for (const label of numericTicks(0, 5e7, 800).map((t) => t.label)) {
      expect(label).toMatch(/^(0|-?\d(\.\d+)?e-?\d+)$/);
    }
    expect(numericTicks(0, 4e-5, 800).map((t) => t.label)).toContain("1e-5");
    expect(numericTicks(0, 100, 800).map((t) => t.label)).toContain("50");
  });

  it("give about one tick per 80 px", () => {
    const count = numericTicks(0, 1, 800).length;
    expect(count).toBeGreaterThanOrEqual(6);
    expect(count).toBeLessThanOrEqual(16);
  });
});

describe("time labels (FR-006a, FR-009h)", () => {
  it("show milliseconds for sub-second spans", () => {
    const labels = timeTicks(DAY + 0.1, DAY + 0.6, 800).map((t) => t.label);
    expect(labels.some((l) => /^\.\d{3}$/.test(l))).toBe(true);
  });

  it("show the date at the day boundary within a multi-day span", () => {
    const labels = timeTicks(DAY - 43_200, DAY + 43_200, 800).map((t) => t.label);
    expect(labels).toContain("Sep 27");
    expect(labels).toContain("12:00");
  });

  it("show years for multi-year spans", () => {
    const labels = timeTicks(DAY, DAY + 5 * 365 * 86_400, 800).map((t) => t.label);
    expect(labels).toContain("2028");
  });

  it("format the hover readout in full", () => {
    expect(formatDateTime(DAY + 60.125)).toBe("2026-09-27 00:01:00.125");
    expect(formatDateTime(DAY)).toBe("2026-09-27 00:00:00");
    expect(formatDateTime(DAY + 1.000123)).toBe("2026-09-27 00:00:01.000123");
    expect(formatExact(1015.2500000001)).toBe("1015.2500000001");
    expect(formatExact(1.23456789e-7)).toBe("1.23456789e-7");
  });
});
