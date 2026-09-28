// Series colors (FR-009i): Okabe–Ito hues, ordered so the first series are the most
// distinguishable under common color-vision deficiencies, with light and dark variants that
// each keep ≥ 3:1 contrast against the plot background.

export const LIGHT_PALETTE = [
  "#0072B2", // blue
  "#D55E00", // vermillion
  "#007A5A", // bluish green
  "#B04A8C", // reddish purple
  "#A86400", // orange
  "#6A5ACD", // violet
  "#4D4D4D", // gray
  "#2F7FAF", // sky blue
] as const;

export const DARK_PALETTE = [
  "#56B4E9",
  "#F07F3C",
  "#2FC49B",
  "#E08DC0",
  "#F0B429",
  "#B3A6FF",
  "#E6E6E6",
  "#9FD4F5",
] as const;

export const LIGHT_PLOT_BACKGROUND = "#ffffff";
export const DARK_PLOT_BACKGROUND = "#16191f";
/** Gridline colors; keep in sync with `--grid` in `app/theme.css`. */
export const LIGHT_GRID = "#eceef2";
export const DARK_GRID = "#252a33";

export interface SeriesStyle {
  color: string;
  /** Beyond the palette size, colors repeat with dashed lines / hollow markers. */
  alternate: boolean;
}

export function seriesStyle(slot: number, dark: boolean): SeriesStyle {
  const palette = dark ? DARK_PALETTE : LIGHT_PALETTE;
  const color = palette[slot % palette.length] ?? palette[0];
  return { color, alternate: Math.floor(slot / palette.length) % 2 === 1 };
}

/**
 * Keeps each column's color slot stable as columns are added or removed: existing columns
 * keep their slot; new ones take the lowest free slot.
 */
export function assignSlots(previous: ReadonlyMap<string, number>, columns: readonly string[]) {
  const slots = new Map<string, number>();
  for (const column of columns) {
    const slot = previous.get(column);
    if (slot !== undefined) slots.set(column, slot);
  }
  const used = new Set(slots.values());
  let next = 0;
  for (const column of columns) {
    if (slots.has(column)) continue;
    while (used.has(next)) next++;
    slots.set(column, next);
    used.add(next);
  }
  return slots;
}

/** WCAG contrast ratio between two `#rrggbb` colors. */
export function contrastRatio(a: string, b: string): number {
  const luminance = (hex: string) => {
    const channels = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
    const [r = 0, g = 0, bl = 0] = channels.map((v) =>
      v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4,
    );
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

/** `#rrggbb` → [r, g, b] in 0..1. */
export function rgb(hex: string): [number, number, number] {
  return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255) as [number, number, number];
}
