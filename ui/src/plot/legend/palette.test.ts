import { describe, expect, it } from "vitest";
import {
  DARK_PALETTE,
  DARK_PLOT_BACKGROUND,
  LIGHT_PALETTE,
  LIGHT_PLOT_BACKGROUND,
  assignSlots,
  contrastRatio,
  seriesStyle,
} from "./palette";

describe("series palette (FR-009i)", () => {
  it.each([
    ["light", LIGHT_PALETTE, LIGHT_PLOT_BACKGROUND],
    ["dark", DARK_PALETTE, DARK_PLOT_BACKGROUND],
  ])("has at least 8 distinct colors with 3:1 contrast in %s", (_, palette, background) => {
    expect(palette.length).toBeGreaterThanOrEqual(8);
    expect(new Set(palette).size).toBe(palette.length);
    for (const color of palette) {
      expect(contrastRatio(color, background)).toBeGreaterThanOrEqual(3);
    }
  });

  it("keeps a column's color while others are added or removed", () => {
    const first = assignSlots(new Map(), ["a", "b", "c"]);
    const second = assignSlots(first, ["a", "c", "d"]);
    expect(second.get("a")).toBe(first.get("a"));
    expect(second.get("c")).toBe(first.get("c"));
    expect(second.get("d")).toBe(first.get("b"));
  });

  it("repeats colors with the alternate style beyond the palette size", () => {
    const n = LIGHT_PALETTE.length;
    expect(seriesStyle(n, false)).toEqual({ color: LIGHT_PALETTE[0], alternate: true });
    expect(seriesStyle(0, false).alternate).toBe(false);
  });
});
